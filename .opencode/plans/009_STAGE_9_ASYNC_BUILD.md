# Stage 9: Async Build Pipeline + WebSocket Log Streaming

## Overview

Stage 9 adds asynchronous building and deployment. Currently, `fugue deploy` blocks the CLI while `npm install && npm run build` runs (up to 5 minutes for large projects). The new build pipeline runs builds in background tasks, streams logs via WebSocket, and tracks build state in PostgreSQL.

This also enables the web Dashboard (Stage 7) to show real-time build progress.

## Architecture

```
POST /api/v1/apps/:id/deploy
  │
  ├── 1. Create Build record (status: pending)
  ├── 2. Enqueue BuildTask to MPSC channel
  └── 3. Return { build_id, status: "pending" } immediately

BuildWorker (tokio task, consumes from channel):
  │
  ├── 4. Update Build (status: running)
  ├── 5. npm install (stream stdout/stderr → PostgreSQL log field + WebSocket)
  ├── 6. npm run build (stream stdout/stderr → log)
  ├── 7. generate_workerd_artifacts()
  ├── 8. Regenerate dispatch config.capnp + dispatch.mjs
  ├── 9. Reload workerd (--watch or restart)
  ├── 10. Update App (status: running)
  └── 11. Update Build (status: success)

If any step fails:
  ├── Update Build (status: failed, error: message)
  └── Update App (status: error)

WebSocket (/api/v1/apps/:id/builds/:build_id/logs):
  │
  └── Client connects → stream build log lines in real-time
```

## Database Changes

```sql
-- Extend builds table with more detail
ALTER TABLE builds ADD COLUMN IF NOT EXISTS
  framework TEXT;

ALTER TABLE builds ADD COLUMN IF NOT EXISTS
  npm_install_log TEXT;

ALTER TABLE builds ADD COLUMN IF NOT EXISTS
  build_log TEXT;

ALTER TABLE builds ADD COLUMN IF NOT EXISTS
  artifacts_log TEXT;
```

## New Files

```
src/
├── build/
│   ├── mod.rs               -- Build pipeline module
│   ├── queue.rs             -- MPSC channel + BuildWorker task
│   ├── runner.rs            -- Shell command execution (npm install, npm run build, esbuild)
│   └── artifacts.rs         -- Workerd artifact generation (moved from runtime/workerd.rs)
└── ws/
    ├── mod.rs               -- WebSocket module
    └── build_logs.rs        -- Build log streaming handler
```

## Build Queue Design

```rust
pub struct BuildTask {
    pub app_id: Uuid,
    pub build_id: Uuid,
    pub source_path: PathBuf,    // Path to app source in data/apps/<id>/source/
    pub framework: Framework,
}

pub enum BuildMessage {
    Task(BuildTask),
    Shutdown,
}

pub struct BuildQueue {
    sender: mpsc::Sender<BuildMessage>,
    handler: JoinHandle<()>,
}

impl BuildQueue {
    pub fn new(db: PgPool, process_manager: Arc<ProcessManager>) -> Self {
        let (tx, rx) = mpsc::channel::<BuildMessage>(16);
        let handler = tokio::spawn(BuildWorker::run(rx, db, process_manager));
        Self { sender: tx, handler }
    }

    pub async fn enqueue(&self, task: BuildTask) -> Result<()> {
        self.sender.send(BuildMessage::Task(task)).await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.sender.send(BuildMessage::Shutdown).await?;
        self.handler.await?;
        Ok(())
    }
}
```

## Build Worker

```rust
async fn run(
    mut rx: mpsc::Receiver<BuildMessage>,
    db: PgPool,
    process_manager: Arc<ProcessManager>,
) {
    while let Some(msg) = rx.recv().await {
        match msg {
            BuildMessage::Task(task) => {
                if let Err(e) = execute_build(&db, &process_manager, task).await {
                    tracing::error!("Build failed: {}", e);
                }
            }
            BuildMessage::Shutdown => break,
        }
    }
}

async fn execute_build(
    db: &PgPool,
    pm: &ProcessManager,
    task: BuildTask,
) -> Result<()> {
    // 1. Update build status to "running"
    update_build_status(db, &task.build_id, "running").await?;

    // 2. Run npm install
    let install_result = run_npm_install(&task.source_path).await?;
    append_build_log(db, &task.build_id, &install_result.output).await?;

    if !install_result.success {
        update_build_status(db, &task.build_id, "failed").await?;
        update_app_status(db, &task.app_id, "error").await?;
        return Ok(());
    }

    // 3. Run framework build
    let build_result = run_framework_build(&task.source_path, &task.framework).await?;
    append_build_log(db, &task.build_id, &build_result.output).await?;

    if !build_result.success {
        update_build_status(db, &task.build_id, "failed").await?;
        update_app_status(db, &task.app_id, "error").await?;
        return Ok(());
    }

    // 4. Generate workerd artifacts (esbuild bundle, static assets, entry.mjs, capnp)
    let workerd_dir = config::workerd_dir();
    let artifacts_result = generate_workerd_artifacts(
        &task.app_id, &task.source_path, &task.framework, &workerd_dir,
    )?;
    append_build_log(db, &task.build_id, "Workerd artifacts generated").await?;

    // 5. Regenerate dispatch config + reload workerd
    let all_apps = get_all_deployed_apps(db).await?;
    generate_dispatch_config(&all_apps, &workerd_dir)?;
    pm.reload_config().await?;

    // 6. Update statuses
    update_build_status(db, &task.build_id, "success").await?;
    update_app_status(db, &task.app_id, "running").await?;

    Ok(())
}
```

## WebSocket Log Streaming

```rust
// In server.rs, add WebSocket route:
// GET /api/v1/apps/:id/builds/:build_id/logs

async fn build_logs_ws(
    ws: WebSocketUpgrade,
    Path((app_id, build_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        stream_build_logs(socket, state.db.clone(), app_id, build_id)
    })
}

async fn stream_build_logs(
    mut socket: WebSocket,
    db: PgPool,
    app_id: Uuid,
    build_id: Uuid,
) {
    // Poll the builds table for new log entries
    // Send each new line to the WebSocket client
    let mut last_offset = 0i64;

    loop {
        let build = get_build(&db, &build_id).await.unwrap();

        // Send new log lines since last_offset
        if let Some(log) = &build.log {
            let lines: Vec<&str> = log.lines().collect();
            for line in lines.iter().skip(last_offset as usize) {
                if socket
                    .send(Message::Text(line.to_string()))
                    .await
                    .is_err()
                {
                    return;
                }
            }
            last_offset = lines.len() as i64;
        }

        // If build is done, send completion message and close
        if build.status == "success" || build.status == "failed" {
            let _ = socket
                .send(Message::Text(format!(
                    "BUILD_{}",
                    build.status.to_uppercase()
                )))
                .await;
            return;
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
```

## API Endpoints (New)

```
POST   /api/v1/apps/:id/deploy
  Body: {} OR { source: { files: {...} } }
  Response: { build_id, deployment_id, status: "pending" }

POST   /api/v1/apps/:id/redeploy
  Body: {}
  Response: { build_id, deployment_id, status: "pending" }

GET    /api/v1/apps/:id/builds
  Response: [{ id, status, framework, created_at, finished_at }]

GET    /api/v1/apps/:id/builds/:build_id
  Response: { id, status, log, error, framework, created_at, finished_at }

GET    /api/v1/apps/:id/builds/:build_id/logs  (WebSocket)
  Streams: build log lines in real-time
  Close: "BUILD_SUCCESS" or "BUILD_FAILED"
```

## Concurrency Limits

To avoid overwhelming the build server:

```rust
pub struct BuildConfig {
    pub max_concurrent_builds: usize,      // Default: 2
    pub build_timeout_secs: u64,             // Default: 300 (5 min)
    pub npm_install_timeout_secs: u64,       // Default: 120 (2 min)
}
```

The queue is MPSC with a bounded channel. If the channel is full, the API returns `429 Too Many Requests`.

## Modified Files

```
src/api/deploy.rs             -- Deploy handler enqueues BuildTask
src/api/apps.rs               -- List builds endpoint
src/daemon/server.rs          -- Add WebSocket route
src/daemon/state.rs           -- Add BuildQueue to AppState
src/process/config_gen.rs     -- Dispatch config generation (called by build worker)
src/runtime/workerd.rs        -- Extract artifact generation to build/artifacts.rs
src/main.rs                   -- Initialize BuildQueue on startup
```

## Testing

```bash
# 1. Create an app
curl -X POST http://localhost:3000/api/v1/apps \
  -d '{"name": "Test App", "framework": "nuxtjs"}'

# 2. Upload source
curl -X POST http://localhost:3000/api/v1/apps/<id>/source \
  -F "archive=@my-app.zip"

# 3. Deploy (returns immediately)
curl -X POST http://localhost:3000/api/v1/apps/<id>/deploy
# → { "build_id": "...", "status": "pending" }

# 4. Stream build logs via WebSocket
wscat -c ws://localhost:3000/api/v1/apps/<id>/builds/<build_id>/logs
# →实时输出构建日志

# 5. Check status
curl http://localhost:3000/api/v1/apps/<id>/status
# → { "status": "running", "url": "http://test-app.fugue.local:3000" }
```

## Supersedes

- Blocking `deploy_command()` in `src/commands/mod.rs` is replaced by async build pipeline
- `npm install` and `npm run build` are now managed by the build worker

## Dependencies

- Stage 6 (Platform API + PostgreSQL + Dynamic Dispatch) must be complete