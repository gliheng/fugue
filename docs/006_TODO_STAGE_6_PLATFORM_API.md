# Stage 6: Platform API + PostgreSQL + Dynamic Dispatch

## Overview

Stage 6 transforms Fugue from a CLI-only tool into a production web platform. Three fundamental changes:

1. **Dynamic Dispatch**: All apps run in a **single workerd process** with a dispatch worker routing by subdomain, instead of one workerd process per app on separate ports.
2. **PostgreSQL**: Replace filesystem-based metadata storage with PostgreSQL for reliable, concurrent data management.
3. **Platform API**: Redesign the HTTP API for web platform use (create app → upload code → build → deploy → access via subdomain).

The architecture remains a CLI + daemon two-process model (unchanged from Stage 1-5). The daemon process gains new responsibilities: serving the management API, reverse proxying to workerd, and coordinating the shared workerd process. The CLI remains a thin HTTP client to the daemon. No architectural change to the binary structure.

## Architecture

```
Browser
  │
  ├── my-app.fugue.local ──┐
  ├── another.fugue.local ──┤
  │                         │
  ▼                         ▼
Fugue Platform (daemon process, port 3000)
  │
  ├── Axum HTTP Server
  │     ├── Host-based routing
  │     │     ├── "*.fugue.local" → Proxy to single workerd (port 8080)
  │     │     └── Reserved subdomains handled internally
  │     └── /api/v1/* → Management API
  │
  ├── Single workerd process (port 8080)
  │     └── Dynamic dispatch architecture:
  │           ┌─────────────────────────────────────────┐
  │           │  Dispatch Worker (service="main")        │
  │           │  - Parse Host header → extract subdomain │
  │           │  - Route to app entry worker             │
  │           │  - Fallback: Platform API (dashboard)    │
  │           └─────┬──────────┬──────────┬──────────────┘
  │                 │          │          │
  │           ┌─────┘    ┌─────┘    ┌─────┘
  │           ▼          ▼          ▼
  │      app-my-blog  app-my-api  app-dashboard
  │      entry worker  (single)   entry worker
  │           │                       │
  │      ┌────┴────┐            ┌────┴────┐
  │      │         │            │         │
  │   ssr-my-blog static-my-blog ssr-dashboard static-dashboard
  │   (bundle.mjs) (assets.mjs)  (bundle.mjs) (assets.mjs)
  │
  ├── Build Engine (async task queue)
  │     ├── npm install + build (per framework)
  │     ├── esbuild bundle + embed
  │     └── Regenerate dispatch capnp config + reload workerd
  │
  ├── PostgreSQL
  │     ├── apps
  │     ├── builds
  │     ├── deployments
  │     └── ai_generations (Stage 8)
  │
  └── Process Manager
        ├── Single workerd lifecycle (with --watch for config reload)
        ├── Health checking
        └── Crash recovery
```

## Key Change: Dynamic Dispatch

### Previous Architecture (Stage 4-5)

Each app gets its own workerd process on a separate port:

```
my-blog ──→ workerd process on port 8081
my-api  ──→ workerd process on port 8082
```

Problems:
- Port pool management (max ~100 apps)
- Memory overhead (one V8 isolate per process)
- Complex reverse proxy routing
- No app isolation within shared runtime

### New Architecture: Single workerd Process with Dispatch

All apps share one workerd process. A **dispatch worker** routes based on the Host header:

```capnp
using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [
    (name = "main", worker = .dispatchWorker),

    # App: my-blog (React Router)
    (name = "app-my-blog", worker = .myBlogEntryWorker),
    (name = "ssr-my-blog", worker = .myBlogSsrWorker),
    (name = "static-my-blog", worker = .myBlogStaticWorker),

    # App: my-api (Single File)
    (name = "app-my-api", worker = .myApiWorker),
  ],
  sockets = [
    ( name = "http",
      address = "*:8080",
      http = (),
      service = "main"
    ),
  ],
);

# ─── Dispatch Worker ────────────────────────────────────────────

const dispatchWorker :Workerd.Worker = (
  modules = [
    (name = "dispatch.mjs", esModule = embed "dispatch.mjs"),
  ],
  compatibilityDate = "2026-04-21",
  compatibilityFlags = ["nodejs_compat"],
  bindings = [
    (name = "APP_MY_BLOG", service = "app-my-blog"),
    (name = "APP_MY_API", service = "app-my-api"),
  ],
);

# ─── App: my-blog (React Router) ────────────────────────────────

const myBlogEntryWorker :Workerd.Worker = (
  modules = [
    (name = "my-blog/entry.mjs", esModule = embed "my-blog/entry.mjs"),
    (name = "my-blog/static-assets.mjs", esModule = embed "my-blog/static-assets.mjs"),
  ],
  compatibilityDate = "2026-04-21",
  compatibilityFlags = ["nodejs_compat"],
  bindings = [
    (name = "SSR", service = "ssr-my-blog"),
    (name = "STATIC", service = "static-my-blog"),
  ],
);

const myBlogSsrWorker :Workerd.Worker = (
  modules = [
    (name = "my-blog/bundle.mjs", esModule = embed "my-blog/bundle.mjs"),
  ],
  compatibilityDate = "2026-04-21",
  compatibilityFlags = ["nodejs_compat"],
  bindings = [
    (name = "ASSETS", service = "static-my-blog"),
  ],
);

const myBlogStaticWorker :Workerd.Worker = (
  modules = [
    (name = "my-blog/static-assets.mjs", esModule = embed "my-blog/static-assets.mjs"),
  ],
  compatibilityDate = "2026-04-21",
);

# ─── App: my-api (Single File) ──────────────────────────────────

const myApiWorker :Workerd.Worker = (
  modules = [
    (name = "my-api/worker.js", esModule = embed "my-api/worker.js"),
  ],
  compatibilityDate = "2024-01-01",
);
```

### Dispatch Worker (dispatch.mjs)

```javascript
// Auto-generated by fugue — do not edit
// Routes requests by subdomain to the appropriate app worker.

const routes = {
  "my-blog": "APP_MY_BLOG",
  "my-api": "APP_MY_API",
};

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const hostname = url.hostname;
    const subdomain = hostname.split(".")[0];

    // Route by subdomain
    const bindingName = routes[subdomain];
    if (bindingName && env[bindingName]) {
      return env[bindingName].fetch(request);
    }

    // Unknown subdomain
    return new Response("Not Found", { status: 404 });
  },
};
```

### Why Not Cloudflare's Dynamic Dispatch?

Cloudflare has a `DispatchNamespace` API that allows runtime-dynamic routing. However:

1. **`DispatchNamespace` is a Cloudflare-specific feature** not available in the open-source `workerd` binary.
2. Static service bindings in capnp config are the only supported routing mechanism in `workerd`.
3. The `env.BINDING_NAME.fetch(request)` pattern (static service bindings) works perfectly and is what workerd supports natively.

So our dispatch approach uses **static service bindings** with a **dynamically-generated config** that regenerates whenever apps are added/removed/updated.

### Service Naming Convention

Each app's services follow the pattern `{{role}}-{{slug}}`:

| Role | Pattern | Example |
|------|---------|---------|
| Dispatch | `main` (single) | `main` |
| App entry | `app-{{slug}}` | `app-my-blog` |
| SSR | `ssr-{{slug}}` | `ssr-my-blog` |
| Static | `static-{{slug}}` | `static-my-blog` |
| Single-file | `app-{{slug}}` | `app-my-api` |

Binding names follow the pattern `APP_{{SLUG_UPPER}}`:
- `my-blog` → `APP_MY_BLOG`
- `my-api` → `APP_MY_API`

### Reload Strategy

When the set of apps changes (deploy, delete, update):

1. **Regenerate** `dispatch.mjs` and `config.capnp` with all current apps
2. **Reload** workerd using `--watch` flag (workerd monitors config file changes)
   - Or graceful restart: start new workerd → drain traffic → stop old
3. **No downtime** for existing apps (static assets and SSR handlers don't change)

### Port Simplification

With single-worker architecture:
- **Workerd listens on port 8080** (single port, always the same)
- **Platform API listens on port 3000**
- **Reverse proxy forwards** from 3000 to 8080 based on Host header

The `WorkerdPool` with port allocation (8080-8180) is replaced by a single managed process.

### Config Regeneration Flow

```
1. Query PostgreSQL for all apps
2. For each deployed app, load app metadata → determine framework type
3. Generate dispatch.mjs with route map:
     { "my-blog": "APP_MY_BLOG", "my-api": "APP_MY_API" }
4. Generate config.capnp with:
     - Dispatch worker (main service)
     - Per-app services (entry, ssr, static for frameworks; single for simple)
     - Appropriate service bindings
5. Write config.capnp to ~/.fugue/workerd/config.capnp
6. Write dispatch.mjs to ~/.fugue/workerd/dispatch.mjs
7. Reload workerd:
     a. If --watch: auto-reloads by touching config file
     b. If manual: stop old process, start new one on port 8080
8. Update app status in PostgreSQL
```

## Key Change: PostgreSQL

### Why PostgreSQL Instead of SQLite

- **Concurrent access**: Multiple API requests can read/write simultaneously without locking
- **Relational queries**: Complex queries across apps, builds, deployments are natural in SQL
- **Scalability**: Prepared for multi-user, multi-tenant in future stages
- **JSON support**: Native `jsonb` type for flexible metadata (env vars, build config)
- **WAL**: Write-ahead logging for durability
- **Production readiness**: Battle-tested, well-understood operations

### Schema

```sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Core app table
CREATE TABLE apps (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        TEXT NOT NULL,
    slug        TEXT UNIQUE NOT NULL,
    subdomain   TEXT UNIQUE NOT NULL,
    framework   TEXT NOT NULL CHECK (framework IN ('single-file', 'nuxtjs', 'react-router')),
    status      TEXT NOT NULL DEFAULT 'created'
                CHECK (status IN ('created', 'building', 'deploying', 'running', 'stopped', 'error')),
    description TEXT,
    env_vars    JSONB NOT NULL DEFAULT '{}',
    source_path TEXT,
    build_path  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_apps_slug ON apps(slug);
CREATE INDEX idx_apps_subdomain ON apps(subdomain);
CREATE INDEX idx_apps_status ON apps(status);

-- Build history
CREATE TABLE builds (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    app_id      UUID NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    status      TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending', 'running', 'success', 'failed')),
    log         TEXT,
    error       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ
);

CREATE INDEX idx_builds_app_id ON builds(app_id);
CREATE INDEX idx_builds_status ON builds(status);

-- Deployment history
CREATE TABLE deployments (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    app_id      UUID NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    build_id    UUID NOT NULL REFERENCES builds(id),
    version     INTEGER NOT NULL DEFAULT 1,
    status      TEXT NOT NULL DEFAULT 'starting'
                CHECK (status IN ('starting', 'running', 'stopped')),
    started_at  TIMESTAMPTZ,
    stopped_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_deployments_app_id ON deployments(app_id);

-- AI generation history (placeholder for Stage 8)
CREATE TABLE ai_generations (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    app_id      UUID REFERENCES apps(id) ON DELETE SET NULL,
    prompt      TEXT NOT NULL,
    framework   TEXT NOT NULL,
    result_code TEXT,
    status      TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending', 'generating', 'success', 'failed')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Rust Integration

Use `sqlx` with PostgreSQL:

```rust
// Cargo.toml:
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }

// Connection (in src/db/mod.rs):
use sqlx::postgres::PgPool;

pub async fn init_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
```

### Configuration

```toml
# ~/.fugue/config.toml
[database]
url = "postgresql://fugue:fugue@localhost:5432/fugue"

[platform]
host = "0.0.0.0"
port = 3000
domain = "fugue.local"

[workerd]
port = 8080
binary = "workerd"
health_check_interval_secs = 30
watch_mode = true  # Use --watch for auto-reload

[logging]
level = "info"
file = "~/.fugue/fugue.log"
```

### Local Development Setup

Provide `docker-compose.yml` for PostgreSQL:

```yaml
version: "3.8"
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: fugue
      POSTGRES_PASSWORD: fugue
      POSTGRES_DB: fugue
    ports:
      - "5432:5432"
    volumes:
      - fugue_pgdata:/var/lib/postgresql/data

volumes:
  fugue_pgdata:
```

## API Design

### App Management

```
POST   /api/v1/apps
  Body: { name: "My Blog", framework: "react-router", description?: "..." }
  Response: { id, name, slug, subdomain, framework, status, created_at }

GET    /api/v1/apps
  Query: ?status=running&framework=nuxtjs
  Response: [{ id, name, slug, framework, status, subdomain, created_at, updated_at }]

GET    /api/v1/apps/:id
  Response: { id, name, slug, framework, status, subdomain, env_vars, description, ... }

PATCH  /api/v1/apps/:id
  Body: { name?, description?, env_vars? }
  Response: updated app

DELETE /api/v1/apps/:id
  Response: { deleted: true }
```

### Source Code

```
POST   /api/v1/apps/:id/source
  Content-Type: multipart/form-data
  Body: file=@project.zip OR { files: { "src/index.tsx": "content..." } }
  Response: { source_path, file_count, total_size }

GET    /api/v1/apps/:id/source
  Query: ?path=src/App.tsx (optional, get single file)
  Response: { files: { "path": "content" } } or { content: "..." }

PUT    /api/v1/apps/:id/source/:path
  Body: { content: "..." }
  Response: { updated: true }
  (For online editor integration)
```

### Build & Deploy

```
POST   /api/v1/apps/:id/deploy
  Body: {} (deploy from current source)
  OR: { source: { files: {...} } } (upload + build + deploy in one step)
  Response: { build_id, deployment_id, status: "building" }

POST   /api/v1/apps/:id/redeploy
  Body: {}
  Response: { build_id, deployment_id, status: "building" }

GET    /api/v1/apps/:id/builds
  Response: [{ id, status, created_at, finished_at }]

GET    /api/v1/apps/:id/builds/:build_id
  Response: { id, status, log, error, created_at, finished_at }
```

### Runtime

```
POST   /api/v1/apps/:id/start
  Response: { status: "running", url: "http://my-blog.fugue.local:3000" }

POST   /api/v1/apps/:id/stop
  Response: { status: "stopped" }

GET    /api/v1/apps/:id/status
  Response: { status, url, uptime_seconds }
```

### Platform

```
GET    /api/v1/platform/status
  Response: { version, apps_total, apps_running, uptime_seconds }
```

## Files Changed / Created

### New Files

```
src/
├── db/
│   ├── mod.rs              -- PgPool init, migration runner
│   ├── models.rs           -- App, Build, Deployment Rust structs
│   └── crud.rs             -- All CRUD operations (sqlx queries)
├── proxy/
│   ├── mod.rs              -- Reverse proxy module
│   └── router.rs           -- Host-based routing, subdomain extraction
├── process/
│   ├── mod.rs              -- Single workerd process manager
│   ├── config_gen.rs       -- Dynamic config.capnp + dispatch.mjs generation
│   ├── health.rs           -- Health check logic
│   └── lifecycle.rs        -- Start/stop/reload workerd process
├── api/
│   ├── mod.rs               -- API route definitions
│   ├── apps.rs              -- App CRUD handlers
│   ├── deploy.rs            -- Deploy/redeploy handlers
│   ├── source.rs            -- Source upload handlers
│   └── runtime.rs           -- Start/stop/status handlers
└── config/
    ├── mod.rs               -- Config file parsing (~/.fugue/config.toml)
    └── defaults.rs          -- Default values

migrations/
└── 001_initial.sql          -- Create tables

docker-compose.yml           -- PostgreSQL for development
```

### Modified Files

```
Cargo.toml                    -- Add sqlx (postgres), toml, axum multipart, tower-http
src/main.rs                   -- Unchanged: CLI + daemon two-process model
src/config.rs                  -- Add platform config (port, domain, workerd port, db url)
src/daemon/server.rs           -- Rewrite: mount /api/v1/* + proxy routes
src/daemon/state.rs            -- Replace with DbPool + AppState struct
src/runtime/workerd.rs         -- Major refactor: extract to process/ module
src/registry/metadata.rs       -- Add slug, subdomain, description fields
src/registry/storage.rs        -- Refactor to use PostgreSQL for metadata
src/error.rs                   -- Add database errors
```

### Cargo.toml Dependencies

```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json", "migrate"] }
toml = "0.8"
axum = { version = "0.7", features = ["multipart", "ws"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
base64 = "0.22"
walkdir = "2"
thiserror = "2"
```

## Process Manager: Single Workerd Process

### Previous: WorkerdPool (per-app processes)

```rust
pub struct WorkerdPool {
    processes: HashMap<String, WorkerdProcess>,  // One process per app
    available_ports: Vec<u16>,                     // Port pool 8080-8180
    workerd_dir: PathBuf,
}
```

### New: ProcessManager (single shared process)

```rust
pub struct ProcessManager {
    process: Option<ManagedProcess>,         // Single workerd process
    config_path: PathBuf,                    // Path to config.capnp
    workerd_dir: PathBuf,                    // ~/.fugue/workerd/
    health_interval: Duration,               // Default 30s
}

pub struct ManagedProcess {
    child: Child,                             // workerd process handle
    started_at: Instant,
    health_status: HealthStatus,
    restart_count: u32,
}

pub enum HealthStatus {
    Starting,
    Healthy,
    Unhealthy,
}
```

### Health Check

```rust
async fn health_check(manager: &ProcessManager) -> HealthStatus {
    // HTTP GET to http://127.0.0.1:8080/ (dispatch worker)
    // Any response (even 404) = healthy
    // Connection refused = unhealthy
}
```

### Config Reload

workerd supports `--watch` flag to monitor config changes and reload automatically:

```rust
async fn reload_config(manager: &ProcessManager, apps: &[App]) -> Result<()> {
    // 1. Regenerate config.capnp and dispatch.mjs
    generate_dispatch_config(apps, &manager.workerd_dir)?;

    // 2. If using --watch, just touch the config file to trigger reload
    //    Otherwise: stop old process, start new one
    if manager.watch_mode {
        let config_path = manager.workerd_dir.join("config.capnp");
        filetime::set_file_mtime(&config_path, filetime::FileTime::now())?;
        wait_for_healthy(Duration::from_secs(5)).await?;
    } else {
        manager.restart().await?;
    }

    Ok(())
}
```

## Reverse Proxy

The platform listens on port 3000 and proxies app requests to workerd on port 8080:

```
*.fugue.local:3000 → Axum Host middleware → Proxy to 127.0.0.1:8080
                                                  │
                                            workerd dispatch
                                            worker routes by
                                            Host header internally
                                                  │
                                            app-my-blog (etc.)
```

Or, for direct access (no proxy needed):
```
*.fugue.local:8080 → workd dispatch worker handles routing directly
```

The proxy on port 3000 is needed for:
1. **API access**: `/api/v1/*` only on port 3000
2. **Dashboard service**: Can serve dashboard from platform binary directly
3. **TLS termination**: Future SSL support
4. **Logging/metrics**: Request tracking before hitting workerd

## Storage Layout

```
~/.fugue/
├── config.toml                    -- Platform configuration
├── fugue.pid                      -- PID file
├── fugue.log                      -- Log file
├── workerd/                       -- Shared workerd artifacts
│   ├── config.capnp               -- Dispatch config (regenerated on deploy)
│   ├── dispatch.mjs               -- Dispatch worker (regenerated on deploy)
│   ├── my-blog/                   -- Per-app artifacts
│   │   ├── entry.mjs
│   │   ├── bundle.mjs
│   │   └── static-assets.mjs
│   └── my-api/
│       └── worker.js
├── data/
│   └── apps/
│       ├── <uuid-1>/
│       │   ├── source/            -- Original project files
│       │   └── build/             -- Build output
│       └── <uuid-2>/
│           ├── source/
│           └── build/

PostgreSQL (via docker-compose or external):
├── apps table
├── builds table
├── deployments table
└── ai_generations table (Stage 8)
```

## CLI Changes

```bash
# Start/stop platform (checks PostgreSQL + starts workerd)
fugue start [--port 3000] [--db-url postgresql://...]
fugue stop
fugue status

# App management (via API)
fugue create my-blog --framework react-router
fugue deploy my-blog ./my-react-app/     # Upload + build + deploy
fugue deploy --source ./handler.js my-api
fugue list
fugue info my-blog
fugue logs my-blog
fugue start my-blog                      # Add app to dispatch config
fugue stop my-blog                       # Remove app from dispatch config
fugue url my-blog                         # Print http://my-blog.fugue.local:3000
fugue delete my-blog

# Config
fugue config set platform.domain fugue.local
fugue config set database.url postgresql://...
```

## Data Migration

On startup, if PostgreSQL tables don't exist, run migrations. If `functions/` directory exists (old format), migrate data to PostgreSQL and reorganize files:

```rust
async fn migrate_from_filesystem(pool: &PgPool, old_dir: &Path) -> Result<()> {
    for entry in fs::read_dir(old_dir)? {
        let dir = entry?;
        let metadata_json = dir.path().join("metadata.json");
        if metadata_json.exists() {
            let old_meta: FunctionMetadata = /* deserialize */;
            let new_id = Uuid::new_v4();

            // Insert into PostgreSQL
            sqlx::query!(
                "INSERT INTO apps (id, name, slug, subdomain, framework, status)
                 VALUES ($1, $2, $3, $4, $5, 'stopped')",
                new_id, old_meta.name, slugify(&old_meta.name),
                slugify(&old_meta.name), framework,
            )
            .execute(pool)
            .await?;

            // Move files from functions/<name>/ to data/apps/<id>/
            let new_dir = data_dir().join("apps").join(&new_id.to_string());
            fs::create_dir_all(&new_dir)?;
            // ... move source/, build/, etc.
        }
    }
}
```

## Testing Plan

```bash
# 1. Start PostgreSQL
docker compose up -d

# 2. Start platform
cargo build && ./target/debug/fugue start

# 3. Create an app via API
curl -X POST http://localhost:3000/api/v1/apps \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Blog", "framework": "react-router"}'
# → { "id": "...", "slug": "test-blog", "subdomain": "test-blog", "status": "created" }

# 4. Upload source + deploy
curl -X POST http://localhost:3000/api/v1/apps/<id>/deploy \
  -F "source=@my-app.zip"
# → { "build_id": "...", "status": "building" }

# 5. Poll status
curl http://localhost:3000/api/v1/apps/<id>/status
# → { "status": "running", "url": "http://test-blog.fugue.local:3000" }

# 6. Access via subdomain (add to /etc/hosts: 127.0.0.1 test-blog.fugue.local)
curl http://test-blog.fugue.local:3000/

# 7. Or access via direct workerd port
curl http://127.0.0.1:8080/ -H "Host: test-blog.fugue.local"

# 8. Verify dispatch config
cat ~/.fugue/workerd/config.capnp

# 9. Deploy second app and verify routing
curl -X POST http://localhost:3000/api/v1/apps \
  -H "Content-Type: application/json" \
  -d '{"name": "My API", "framework": "single-file"}'
# Verify config.capnp now contains both apps' services

# 10. Migrate from old format (functions/ dir exists)
./target/debug/fugue start --migrate
```

## Supersedes

- `WorkerdPool` → `ProcessManager` (single process, dynamic dispatch)
- `FunctionRegistry` filesystem storage → PostgreSQL
- Per-app port allocation → single port + dispatch routing
- `functions/<name>/` layout → `data/apps/<uuid>/`

## Dependencies on Prior Stages

- Stage 4 (Nuxt.js workerd): artifact generation reused as-is
- Stage 5 (React Router workerd): artifact generation reused as-is
- The `generate_*_workerd_artifacts()` functions remain, but output is organized per-app under `~/.fugue/workerd/<slug>/`

## Open Questions

1. **Workerd --watch reliability**: Need to verify that `--watch` flag properly reloads all worker modules on config change. If not reliable, will use graceful restart (stop old → start new).
2. **Large config scaling**: What happens when there are 100+ apps in a single config.capnp? workerd embeds all modules, so memory usage scales with total app size. May need pagination or grouping.
3. **Hot reload without downtime**: With --watch, does workerd serve requests during reload? Or is there a brief gap? Need testing.
4. **App isolation**: All apps share the same workerd process. A misbehaving app can potentially affect others. This is the same tradeoff Cloudflare makes with Workers.