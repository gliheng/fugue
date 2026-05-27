# Stage 9: Async Build Pipeline + Independent Builder Process

## Overview

Stage 9 adds asynchronous building and deployment via an independent `fugue-builder` process. Currently, `fugue deploy` blocks the CLI while `npm install && npm run build` runs (up to 5 minutes for large projects). The new architecture decouples build execution into a separate binary that communicates with the main daemon via NATS.

Key benefits:
- Build isolation (crashes don't affect the API server)
- Independent scaling (run multiple builders)
- Distributed builds across machines possible
- Cleaner separation of concerns

## Architecture

```
┌─────────────────┐         NATS          ┌─────────────────┐
│   fugue daemon   │◄────────────────────►│  fugue-builder   │
│   (API server)   │                       │  (build worker)  │
└─────────────────┘                       └─────────────────┘
        │                                          │
        ▼                                          ▼
   PostgreSQL                                  Filesystem
   (build state)                          (source + artifacts)
```

### Communication Flow

```
POST /api/v1/apps/:id/deploy
  │
  ├── 1. Create Build record in DB (status: pending)
  ├── 2. Publish BuildTask to NATS subject: "fugue.build.requests"
  └── 3. Return { build_id, status: "pending" } immediately

fugue-builder (subscribed to "fugue.build.requests" with queue group):
  │
  ├── 4. Receive BuildTask
  ├── 5. Update Build (status: running) via NATS → daemon
  ├── 6. npm install (stream logs via NATS)
  ├── 7. npm run build (stream logs via NATS)
  ├── 8. generate_workerd_artifacts()
  └── 9. Publish BuildResult to "fugue.build.results.{build_id}"

fugue daemon (subscribed to "fugue.build.results.>"):
  │
  ├── 10. Update Build (status: success/failed)
  ├── 11. If success: regenerate dispatch config + reload workerd
  └── 12. Update App (status: running/error)

WebSocket (/api/v1/apps/:id/builds/:build_id/logs):
  │
  └── Client connects → stream build log lines from DB in real-time
```

## NATS Subjects

| Subject | Publisher | Subscriber | Payload |
|---------|-----------|------------|---------|
| `fugue.build.requests` | daemon | builder (queue group) | `BuildTask` |
| `fugue.build.logs.{build_id}` | builder | daemon | `BuildLog` |
| `fugue.build.results.{build_id}` | builder | daemon | `BuildResult` |

## Project Structure (Cargo Workspace)

Convert from single package to workspace:

```
fugue/
├── Cargo.toml           # [workspace] definition
├── crates/
│   ├── fugue/           # Main daemon binary (existing code)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs
│   │       └── ... (existing modules moved here)
│   ├── fugue-builder/   # New builder binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── nats.rs       # NATS connection + pub/sub
│   │       ├── runner.rs     # Build execution (npm install, npm run build)
│   │       └── artifacts.rs  # Workerd artifact generation
│   └── fugue-common/    # Shared library
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── error.rs      # FugueError (extracted)
│           ├── config.rs     # PlatformConfig (extracted)
│           ├── models.rs     # BuildTask, BuildResult, Framework, etc.
│           ├── package.rs    # PackageManager (deduplicated)
│           └── fs.rs         # calculate_dir_size, find_esbuild
├── migrations/          # Stays at workspace root
├── docs/
└── examples/
```

## Shared Types (fugue-common/src/models.rs)

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTask {
    pub build_id: Uuid,
    pub app_id: Uuid,
    pub app_slug: String,
    pub source_path: PathBuf,
    pub framework: Framework,
    pub skip_install: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Framework {
    Worker,
    NuxtJs,
    ReactRouter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub build_id: Uuid,
    pub app_id: Uuid,
    pub success: bool,
    pub output_size: u64,
    pub build_time_ms: u128,
    pub error: Option<String>,
    pub artifacts_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildLog {
    pub build_id: Uuid,
    pub line: String,
    pub stream: LogStream,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogStream {
    Stdout,
    Stderr,
    System,
}
```

## Embedded NATS Server

The `fugue start` command launches an embedded NATS server:

```rust
// In src/commands/mod.rs - start_platform()
use embedded_nats::Server;

let nats_server = Server::builder()
    .bind("127.0.0.1:4222")
    .start()
    .await?;

info!("Embedded NATS server started on 127.0.0.1:4222");

let nats_client = async_nats::connect("nats://127.0.0.1:4222").await?;
```

## Builder Main Loop (fugue-builder/src/main.rs)

```rust
use async_nats::Client;
use fugue_common::models::{BuildTask, BuildResult, BuildLog, LogStream};
use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::init();

    let nats_url = std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());

    let client = async_nats::connect(&nats_url).await?;
    info!("Connected to NATS at {}", nats_url);

    // Subscribe with queue group for load balancing
    let mut subscriber = client
        .queue_subscribe("fugue.build.requests", "fugue-builders")
        .await?;
    info!("Listening on fugue.build.requests [queue: fugue-builders]");

    while let Some(message) = subscriber.next().await {
        let task: BuildTask = serde_json::from_slice(&message.payload)?;
        info!("Received build task: {}", task.build_id);

        let client = client.clone();
        tokio::spawn(async move {
            if let Err(e) = execute_build(&client, task).await {
                error!("Build execution failed: {}", e);
            }
        });
    }

    Ok(())
}

async fn execute_build(client: &Client, task: BuildTask) -> anyhow::Result<()> {
    publish_log(client, &task.build_id, "Build started", LogStream::System).await;

    let result = match task.framework {
        Framework::Worker => build_worker(&task).await,
        Framework::NuxtJs => build_nuxtjs(&task).await,
        Framework::ReactRouter => build_reactrouter(&task).await,
    };

    let build_result = match result {
        Ok((output_size, build_time_ms)) => {
            publish_log(client, &task.build_id, "Build succeeded", LogStream::System).await;
            BuildResult {
                build_id: task.build_id,
                app_id: task.app_id,
                success: true,
                output_size,
                build_time_ms,
                error: None,
                artifacts_path: Some(get_artifacts_path(&task)),
            }
        }
        Err(e) => {
            publish_log(client, &task.build_id, &format!("Build failed: {}", e), LogStream::Stderr).await;
            BuildResult {
                build_id: task.build_id,
                app_id: task.app_id,
                success: false,
                output_size: 0,
                build_time_ms: 0,
                error: Some(e.to_string()),
                artifacts_path: None,
            }
        }
    };

    let subject = format!("fugue.build.results.{}", task.build_id);
    client.publish(subject, serde_json::to_vec(&build_result)?.into()).await?;

    Ok(())
}

async fn publish_log(client: &Client, build_id: &Uuid, line: &str, stream: LogStream) {
    let log = BuildLog { build_id: *build_id, line: line.to_string(), stream };
    let subject = format!("fugue.build.logs.{}", build_id);
    client.publish(subject, serde_json::to_vec(&log).unwrap().into()).await.unwrap();
}
```

## Daemon Integration

### Deploy handler (fugue/src/api/deploy.rs)

```rust
use async_nats::Client as NatsClient;

pub struct AppState {
    pub db: PgPool,
    pub process: Arc<RwLock<ProcessManager>>,
    pub config: PlatformConfig,
    pub nats: NatsClient,  // NEW
}

async fn deploy_app(
    State(state): State<AppState>,
    Path(app_id): Path<Uuid>,
) -> Result<Json<DeployResponse>, ApiError> {
    let build = create_build(&state.db, &app_id).await?;

    let task = BuildTask {
        build_id: build.id,
        app_id,
        app_slug: app.slug.clone(),
        source_path: get_source_path(&app_id),
        framework: app.framework.clone(),
        skip_install: false,
    };

    state.nats
        .publish("fugue.build.requests", serde_json::to_vec(&task)?.into())
        .await?;

    Ok(Json(DeployResponse {
        build_id: build.id,
        status: "pending".to_string(),
    }))
}
```

### Background task: listen for build results

```rust
async fn listen_build_results(nats: NatsClient, db: PgPool, process: Arc<RwLock<ProcessManager>>) {
    let mut subscriber = nats.subscribe("fugue.build.results.>").await.unwrap();

    while let Some(msg) = subscriber.next().await {
        let result: BuildResult = serde_json::from_slice(&msg.payload).unwrap();

        update_build_status(&db, &result.build_id, if result.success { "success" } else { "failed" })
            .await.unwrap();

        if result.success {
            let apps = get_all_deployed_apps(&db).await.unwrap();
            generate_dispatch_config(&apps, &workerd_dir()).unwrap();
            process.write().await.reload_config().await.unwrap();
            update_app_status(&db, &result.app_id, "running").await.unwrap();
        } else {
            update_app_status(&db, &result.app_id, "error").await.unwrap();
        }
    }
}
```

### Background task: persist logs to DB

```rust
async fn persist_build_logs(nats: NatsClient, db: PgPool) {
    let mut subscriber = nats.subscribe("fugue.build.logs.>").await.unwrap();

    while let Some(msg) = subscriber.next().await {
        let log: BuildLog = serde_json::from_slice(&msg.payload).unwrap();

        sqlx::query("UPDATE builds SET log = COALESCE(log, '') || $1 WHERE id = $2")
            .bind(format!("{}\n", log.line))
            .bind(log.build_id)
            .execute(&db)
            .await
            .unwrap();
    }
}
```

## WebSocket Log Streaming

```rust
// GET /api/v1/apps/:id/builds/:build_id/logs

async fn build_logs_ws(
    ws: WebSocketUpgrade,
    Path((app_id, build_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        stream_build_logs(socket, state.db.clone(), build_id)
    })
}

async fn stream_build_logs(mut socket: WebSocket, db: PgPool, build_id: Uuid) {
    let mut last_offset = 0;

    loop {
        let build = get_build(&db, &build_id).await.unwrap();

        if let Some(log) = &build.log {
            let lines: Vec<&str> = log.lines().collect();
            for line in lines.iter().skip(last_offset) {
                if socket.send(Message::Text(line.to_string())).await.is_err() {
                    return;
                }
            }
            last_offset = lines.len();
        }

        if build.status == "success" || build.status == "failed" {
            let _ = socket
                .send(Message::Text(format!("BUILD_{}", build.status.to_uppercase())))
                .await;
            return;
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
```

## Database Changes

```sql
-- Ensure log column exists
ALTER TABLE builds ADD COLUMN IF NOT EXISTS log TEXT;

-- Ensure framework column exists
ALTER TABLE builds ADD COLUMN IF NOT EXISTS framework TEXT;
```

## API Endpoints

```
POST   /api/v1/apps/:id/deploy
  Body: {} OR { source: { files: {...} } }
  Response: { build_id, status: "pending" }

POST   /api/v1/apps/:id/redeploy
  Body: {}
  Response: { build_id, status: "pending" }

GET    /api/v1/apps/:id/builds
  Response: [{ id, status, framework, created_at, finished_at }]

GET    /api/v1/apps/:id/builds/:build_id
  Response: { id, status, log, error, framework, created_at, finished_at }

GET    /api/v1/apps/:id/builds/:build_id/logs  (WebSocket)
  Streams: build log lines in real-time
  Close: "BUILD_SUCCESS" or "BUILD_FAILED"
```

## New Dependencies

### fugue-builder/Cargo.toml
```toml
[dependencies]
fugue-common = { path = "../fugue-common" }
async-nats = "0.35"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1", features = ["serde"] }
anyhow = "1"
```

### fugue/Cargo.toml (additions)
```toml
async-nats = "0.35"
embedded-nats = "0.5"
```

## Migration Steps

### Phase 1: Extract Shared Code
1. Create workspace structure (`Cargo.toml` with `[workspace]`)
2. Create `crates/fugue-common` with shared types
3. Move `PackageManager`, `calculate_dir_size`, `find_esbuild` to `fugue-common/src/package.rs` and `fugue-common/src/fs.rs`
4. Move `FugueError` to `fugue-common/src/error.rs`
5. Move `PlatformConfig` to `fugue-common/src/config.rs`
6. Define `BuildTask`, `BuildResult`, `BuildLog`, `Framework` in `fugue-common/src/models.rs`
7. Move existing `fugue` code to `crates/fugue`
8. Update imports to use `fugue-common`

### Phase 2: Create Builder Binary
1. Create `crates/fugue-builder`
2. Move build logic from `src/worker/builder.rs`, `src/nuxtjs/builder.rs`, `src/reactrouter/builder.rs` to `fugue-builder/src/runner.rs`
3. Move artifact generation from `src/runtime/workerd.rs` to `fugue-builder/src/artifacts.rs`
4. Implement NATS subscription loop in `fugue-builder/src/main.rs`
5. Implement log publishing

### Phase 3: Integrate NATS into Daemon
1. Add `async-nats` and `embedded-nats` dependencies
2. Start embedded NATS server in `start_platform()`
3. Add `NatsClient` to `AppState`
4. Modify `deploy_app()` to publish to NATS
5. Add `listen_build_results()` background task
6. Add `persist_build_logs()` background task
6. Add WebSocket route for build log streaming

### Phase 4: Cleanup
1. Remove old builder modules from `fugue` crate (keep detection modules)
2. Remove `tokio::spawn` build logic from `deploy.rs`
3. Update tests
4. Update documentation

## Configuration

### fugue daemon config.toml
```toml
[nats]
embedded = true              # Start embedded NATS server (default: true)
port = 4222                  # NATS port (default: 4222)
```

### fugue-builder environment variables
```bash
NATS_URL=nats://localhost:4222    # NATS server URL (default)
RUST_LOG=info                     # Log level
```

## Running

```bash
# Terminal 1: Start fugue daemon (includes embedded NATS)
fugue start

# Terminal 2: Start builder process
fugue-builder

# Run multiple builders for load balancing
fugue-builder  # Instance 1
fugue-builder  # Instance 2

# Or connect to remote NATS
NATS_URL=nats://remote:4222 fugue-builder
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
# → 实时输出构建日志

# 5. Check status
curl http://localhost:3000/api/v1/apps/<id>/status
# → { "status": "running", "url": "http://test-app.fugue.local:3000" }
```

## Supersedes

- Blocking `deploy_command()` in `src/commands/mod.rs` is replaced by async build pipeline
- `tokio::spawn` in-process builds replaced by NATS pub/sub to independent builder
- MPSC channel approach replaced by NATS (simpler, supports distributed builds)
- Duplicated `PackageManager`, `BuildResult`, `calculate_dir_size` extracted to shared crate

## Dependencies

- Stage 6 (Platform API + PostgreSQL + Dynamic Dispatch) must be complete
