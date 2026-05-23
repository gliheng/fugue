# Stage 10: Production Hardening

## Overview

Stage 10 adds production-grade security, stability, observability, and operational features. This makes Fugue suitable for self-hosted deployment by teams.

## Areas of Improvement

### 1. Authentication & Authorization

**Goal**: Simple token-based auth for API and Dashboard.

```rust
pub struct AuthMiddleware;

impl AuthMiddleware {
    /// Check bearer token against configured API keys
    /// Multiple keys supported: admin key, read-only key
    /// Skip auth for health check endpoints
}
```

**Configuration**:

```toml
[auth]
enabled = true
admin_keys = ["fugue-admin-xxx"]     # Full access
read_keys = ["fugue-read-yyy"]       # Read-only (list apps, view status)
```

**Implementation**:
- `Authorization: Bearer <key>` header on all `/api/v1/*` endpoints
- Skip auth for `/health` and public app access (`*.fugue.local`)
- Store keys as hashed values in PostgreSQL
- Dashboard stores key in localStorage/sessionStorage

**Database**:

```sql
CREATE TABLE api_keys (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        TEXT NOT NULL,
    key_hash    TEXT NOT NULL,        -- bcrypt hash
    role        TEXT NOT NULL CHECK (role IN ('admin', 'read')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);
```

### 2. CORS & Security Headers

```rust
use tower_http::cors::{CorsLayer, Any};
use tower_http::headers::DefaultHeadersLayer;

// CORS: allow dashboard.fugue.local
let cors = CorsLayer::new()
    .allow_origin(["http://dashboard.fugue.local:3000".parse().unwrap()])
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
    .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

// Security headers
let headers = DefaultHeadersLayer::new()
    .header("X-Content-Type-Options", "nosniff")
    .header("X-Frame-Options", "DENY")
    .header("X-XSS-Protection", "1; mode=block");
```

### 3. Process Health & Recovery

**Workerd Process Health Check**:

```rust
pub struct HealthChecker {
    interval: Duration,         // Default: 30s
    max_failures: u32,          // Default: 3
    restart_delay: Duration,    // Default: 1s
    max_restart_attempts: u32,  // Default: 3
}

impl HealthChecker {
    /// Probe: HTTP GET to http://127.0.0.1:8080/
    /// Any response (even 404) = healthy
    /// Connection refused = unhealthy
    /// After max_failures consecutive failures: restart workerd
    /// After max_restart_attempts: mark all apps as "error"
}
```

**Workerd Crash Recovery**:

```rust
impl ProcessManager {
    /// Called when workerd process exits unexpectedly
    async fn on_workerd_crash(&mut self) -> Result<()> {
        tracing::error!("workerd process crashed! Restarting...");

        self.restart_count += 1;
        if self.restart_count > self.max_restart_attempts {
            tracing::error!("Max restart attempts reached. Marking all apps as error.");
            self.mark_all_apps_error().await?;
            return Ok(());
        }

        // Wait before restart
        tokio::time::sleep(self.restart_delay).await;

        // Restart workerd with current config
        self.start().await?;

        Ok(())
    }
}
```

### 4. Structured Logging

```rust
use tracing_subscriber::fmt::format::FmtContext;

// JSON-structured logging to file, pretty-printed to stderr
let file_appender = tracing_appender::rolling::daily(
    config::log_dir(),
    "fugue.log"
);

// Log format:
// {"timestamp":"2026-05-06T10:30:00Z","level":"INFO","target":"fugue::daemon",
//  "message":"workerd started","app_id":"...","port":8080}
```

### 5. TLS Support

```rust
// Using tokio-rustls for HTTPS
use tokio_rustls::TlsAcceptor;

// Configuration:
[platform]
tls_enabled = true
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"

// Auto-generated self-signed cert for local development:
// fugue tls generate-self-signed
```

### 6. Resource Limits

**Build Resource Limits**:

```rust
pub struct BuildLimits {
    pub max_concurrent_builds: usize,       // Default: 2
    pub npm_install_timeout: Duration,       // Default: 120s
    pub build_timeout: Duration,             // Default: 300s
    pub max_source_size: usize,              // Default: 100MB
    pub max_app_count: usize,                // Default: 50
}
```

**Workerd Resource Limits** (via cgroups or ulimit):

```rust
// Set resource limits for workerd process
pub struct WorkerdLimits {
    pub max_memory_mb: u32,       // Default: 512
    pub max_cpu_percent: u32,     // Not directly enforceable without cgroups
    pub max_request_timeout: Duration, // Default: 30s
}
```

### 7. Backup & Export

```bash
# Export all app data
fugue export --output fugue-backup.tar.gz

# Import from backup
fugue import --input fugue-backup.tar.gz

# Backup includes:
# - PostgreSQL dump (pg_dump)
# - App source files (data/apps/)
# - Workerd artifacts (workerd/)
# - Platform config (config.toml)
```

### 8. Metrics & Monitoring

```rust
// Prometheus-compatible metrics endpoint
// GET /metrics

pub struct Metrics {
    pub apps_total: AtomicU64,
    pub apps_running: AtomicU64,
    pub builds_total: AtomicU64,
    pub builds_failed: AtomicU64,
    pub requests_total: AtomicU64,
    pub requests_by_app: RwLock<HashMap<String, AtomicU64>>,
    pub workerd_restarts: AtomicU64,
    pub build_duration_seconds: Histogram,
}
```

**Endpoint**:

```
GET /metrics
  → Prometheus format:
    fugue_apps_total 12
    fugue_apps_running 8
    fugue_builds_total 45
    fugue_builds_failed 3
    fugue_http_requests_total 1230
    fugue_workerd_restarts 1
    fugue_build_duration_seconds_bucket{...}
```

### 9. Graceful Shutdown

```rust
pub async fn graceful_shutdown(
    process_manager: Arc<ProcessManager>,
    db: PgPool,
) {
    // 1. Stop accepting new connections
    // 2. Wait for in-flight requests to complete (timeout: 30s)
    // 3. Stop build workers
    // 4. Stop workerd process
    // 5. Close database connections
    // 6. Exit
}
```

### 10. Database Connection Pooling

```rust
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(20)           // Max connections
    .min_connections(5)            // Min idle connections
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .connect(&database_url)
    .await?;
```

## New Files

```
src/
├── auth/
│   ├── mod.rs               -- Auth middleware
│   └── keys.rs              -- API key management
├── metrics/
│   ├── mod.rs               -- Metrics collection
│   └── prometheus.rs         -- Prometheus endpoint handler
└── backup/
    ├── mod.rs               -- Export/import
    └── tar.rs               -- Tarball creation/extraction
```

## Modified Files

```
Cargo.toml                    -- Add bcrypt, tower-http (cors, headers), tokio-rustls, prometheus
src/daemon/server.rs           -- Add CORS, auth middleware, metrics endpoint, /health
src/process/lifecycle.rs       -- Add health checking, crash recovery
src/process/mod.rs             -- Add resource limits, restart tracking
src/daemon/state.rs            -- Add Metrics to AppState
src/config/mod.rs              -- Add auth, TLS, limits config
src/main.rs                    -- Add graceful shutdown, TLS listener
```

## Cargo.toml Additions

```toml
bcrypt = "0.16"
tower-http = { version = "0.5", features = ["cors", "trace", "timeout", "limit"] }
tokio-rustls = "0.26"
rcgen = "0.13"           # Self-signed cert generation
prometheus = "0.13"
tracing-appender = "0.2"
```

## Configuration

```toml
# ~/.fugue/config.toml (full production config)

[database]
url = "postgresql://fugue:fugue@localhost:5432/fugue"
max_connections = 20
min_connections = 5

[platform]
host = "0.0.0.0"
port = 3000
domain = "fugue.local"

[workerd]
port = 8080
binary = "workerd"
health_check_interval_secs = 30
watch_mode = true

[auth]
enabled = true
admin_keys = []            # Auto-generated on first start
read_keys = []

[tls]
enabled = false
cert = ""
key = ""

[limits]
max_concurrent_builds = 2
max_apps = 50
max_source_size_mb = 100
npm_install_timeout_secs = 120
build_timeout_secs = 300

[logging]
level = "info"
file = "~/.fugue/fugue.log"
format = "json"            # or "pretty"

[metrics]
enabled = true
endpoint = "/metrics"
```

## CLI Additions

```bash
# Auth management
fugue auth create-key --role admin    # Generate admin API key
fugue auth create-key --role read     # Generate read-only API key
fugue auth list-keys
fugue auth revoke-key <key-id>

# TLS
fugue tls generate-self-signed       # Generate self-signed cert for local dev
fugue tls check                      # Verify TLS configuration

# Backup
fugue export --output backup.tar.gz
fugue import --input backup.tar.gz

# Metrics
fugue metrics                        # Show current metrics summary
```

## Testing

```bash
# 1. Auth
curl -X POST http://localhost:3000/api/v1/apps \
  -H "Authorization: Bearer fugue-admin-xxx" \
  -d '{"name": "Test", "framework": "react-router"}'
# → 200 OK

curl -X POST http://localhost:3000/api/v1/apps \
  # (no auth header)
# → 401 Unauthorized

curl -X POST http://localhost:3000/api/v1/apps \
  -H "Authorization: Bearer fugue-read-yyy" \
  -d '{"name": "Test", "framework": "react-router"}'
# → 403 Forbidden (read-only key)

# 2. App access still public (no auth needed)
curl http://test-app.fugue.local:3000/
# → 200 OK (no auth required for app access)

# 3. Health check
curl http://localhost:3000/health
# → { "status": "healthy", "version": "0.6.0", "uptime_seconds": 3600 }

# 4. Metrics
curl http://localhost:3000/metrics
# → Prometheus format metrics

# 5. Backup
fugue export --output fugue-backup-2026-05-06.tar.gz

# 6. Crash recovery
kill -9 $(pgrep workerd)
# → Platform detects crash, restarts workerd, recovers

# 7. TLS
fugue tls generate-self-signed
fugue start --tls
curl https://dashboard.fugue.local:3000/ --insecure
```

## Dependencies

- Stage 6-9 must be complete
- This stage is the final production hardening layer