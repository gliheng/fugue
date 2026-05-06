# Fugue Platform Roadmap

## Implementation Priority

The stages are broken into smaller, shippable increments. Each increment is a working system you can test.

### P0: Dynamic Dispatch (Foundation)

**Goal**: Replace per-app workerd processes with a single shared process + dispatch worker.

This is the architectural backbone — everything else depends on it.

| Step | Description | Key Changes |
|------|-------------|-------------|
| P0.1 | Dispatch worker template | Generate `dispatch.mjs` with route map |
| P0.2 | Shared config.capnp generator | Generate single config with all apps as services |
| P0.3 | Service naming convention | `app-{slug}`, `ssr-{slug}`, `static-{slug}` |
| P0.4 | ProcessManager (single process) | Replace `WorkerdPool` with single workerd process manager |
| P0.5 | Config reload | Use `--watch` or graceful restart on deploy/delete |
| P0.6 | Reverse proxy | Axum Host-based proxy → workerd port 8080 |
| P0.7 | Slugify app names | Generate URL-safe slugs from app names |
| P0.8 | Migration from old format | Convert existing per-app configs to shared dispatch |

### P1: PostgreSQL Migration

**Goal**: Replace filesystem metadata with PostgreSQL.

| Step | Description |
|------|-------------|
| P1.1 | Add sqlx + docker-compose.yml |
| P1.2 | Create migration file (apps, builds, deployments tables) |
| P1.3 | Database models + CRUD in Rust |
| P1.4 | Replace `FunctionRegistry` with PostgreSQL queries |
| P1.5 | Migrate existing `functions/` data to PostgreSQL |

### P2: Platform API v1

**Goal**: REST API for full app lifecycle.

| Step | Description |
|------|-------------|
| P2.1 | App CRUD endpoints (create, list, get, delete) |
| P2.2 | Source upload endpoint (multipart) |
| P2.3 | Deploy endpoint (trigger build + dispatch config regeneration) |
| P2.4 | Start/stop/polling endpoints |
| P2.5 | Config file support (~/.fugue/config.toml) |

### P3: Async Build Pipeline

**Goal**: Background builds with WebSocket log streaming.

| Step | Description |
|------|-------------|
| P3.1 | Build queue (MPSC channel + BuildWorker task) |
| P3.2 | Shell command runner (npm install, npm build, esbuild) |
| P3.3 | Build status tracking in PostgreSQL |
| P3.4 | WebSocket endpoint for live build logs |
| P3.5 | Deploy → build → regenerate dispatch config → reload |

### P4: AI Code Generation

| Step | Description |
|------|-------------|
| P4.1 | OpenAI-compatible API client |
| P4.2 | Framework-specific system prompts |
| P4.3 | Code parser (extract files from AI response) |
| P4.4 | AI generation API endpoint |
| P4.5 | Auto-deploy flow (generate → save source → build) |

### P5: Dashboard

| Step | Description |
|------|-------------|
| P5.1 | React Router v7 project scaffold |
| P5.2 | App list + detail pages |
| P5.3 | Code editor (Monaco) |
| P5.4 | Deploy + build log UI |
| P5.5 | AI generation UI |
| P5.6 | Dashboard as self-hosted Fugue app |

### P6: Production Hardening

| Step | Description |
|------|-------------|
| P6.1 | Bearer token auth middleware |
| P6.2 | CORS + security headers |
| P6.3 | Workerd health check + crash recovery |
| P6.4 | TLS support (rustls) |
| P6.5 | Prometheus metrics endpoint |
| P6.6 | Backup/export CLI |

## Dependency Graph

```
P0 (Dynamic Dispatch)
 ├──→ P1 (PostgreSQL)
 │     └──→ P2 (Platform API)
 │           └──→ P3 (Async Build)
 │                 └──→ P4 (AI Code Gen)
 │                       └──→ P5 (Dashboard)
 │                             └──→ P6 (Production)
 └──→ P2 can start partially before P1 (proxy + dispatch work without DB)
```

## Current Status: Starting P0 (Dynamic Dispatch)