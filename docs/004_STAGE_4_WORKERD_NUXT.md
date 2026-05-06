# Stage 4: Real workerd for Nuxt.js

## Overview

Stage 4 replaces the Node.js runtime for Nuxt.js deployments with real `workerd`. Previously, `spawn_workerd_nuxtjs()` spawned a Node.js process to run the Nitro server. Now it generates a workerd Cap'n Proto config and runs `workerd serve` directly — the same runtime Cloudflare uses in production.

## Problem

The Stage 3 implementation ran Nuxt.js via Node.js because the Nitro server output consists of multiple ES modules with relative imports, which workerd couldn't resolve from a single `embed` directive. The `npx wrangler dev` command handles this internally via miniflare, but Fugue invokes workerd directly.

## Solution

At deploy time, the platform:

1. **Embeds static assets** — reads `.output/public/`, base64-encodes every file into a `static-assets.mjs` JS module
2. **Bundles the SSR server** — runs esbuild to collapse the 15+ Nitro modules into a single `bundle.mjs`, marking `node:*` and `cloudflare:workers` as external (provided by workerd at runtime)
3. **Generates an entry worker** — `entry.mjs` routes requests: static assets from the embedded map, everything else to the SSR worker via service binding
4. **Generates `config.capnp`** — a 3-service workerd config (entry, SSR, static) with `nodejs_compat`

At invoke time, the daemon runs `workerd serve config.capnp -s http=*:<port>`.

## Architecture

```
fugue deploy
  ├── nuxt build          → .output/
  ├── embed static assets → ~/.fugue/workerd/<name>/static-assets.mjs
  ├── esbuild bundle      → ~/.fugue/workerd/<name>/bundle.mjs
  ├── generate entry      → ~/.fugue/workerd/<name>/entry.mjs
  └── generate capnp      → ~/.fugue/workerd/<name>/config.capnp

fugue invoke
  └── workerd serve config.capnp
        ├── entry worker   (routes /_nuxt/* → static, rest → SSR)
        ├── SSR worker     (Nitro server, env.ASSETS → static service)
        └── static worker  (serves embedded public files)
```

### workerd config (config.capnp)

```capnp
using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [
    (name = "main", worker = .entryWorker),
    (name = "ssr", worker = .ssrWorker),
    (name = "static", worker = .staticWorker),
  ],
  sockets = [
    ( name = "http", address = "*:8787", http = (), service = "main" ),
  ],
);

const entryWorker :Workerd.Worker = (
  modules = [
    (name = "entry.mjs", esModule = embed "entry.mjs"),
    (name = "static-assets.mjs", esModule = embed "static-assets.mjs"),
  ],
  compatibilityDate = "2026-04-21",
  compatibilityFlags = ["nodejs_compat"],
  bindings = [
    (name = "SSR", service = "ssr"),
    (name = "STATIC", service = "static"),
  ],
);

const ssrWorker :Workerd.Worker = (
  modules = [
    (name = "bundle.mjs", esModule = embed "bundle.mjs"),
  ],
  compatibilityDate = "2026-04-21",
  compatibilityFlags = ["nodejs_compat"],
  bindings = [
    (name = "ASSETS", service = "static"),
  ],
);

const staticWorker :Workerd.Worker = (
  modules = [
    (name = "static-assets.mjs", esModule = embed "static-assets.mjs"),
  ],
  compatibilityDate = "2026-04-21",
);
```

### Why 3 services?

- **entry worker**: Intercepts static asset requests (`/_nuxt/*`) before they reach the SSR handler. Returns files from the embedded map with correct MIME types and immutable cache headers.
- **SSR worker**: The bundled Nitro server. Its `env.ASSETS` binding points to the static worker, matching the Cloudflare Workers contract (`env.ASSETS.fetch(request)`).
- **static worker**: Standalone service satisfying the ASSETS binding. Not directly called in the happy path (the entry worker intercepts first), but present for correctness.

## Files changed

### `src/runtime/workerd.rs`

- **`generate_nuxtjs_workerd_artifacts()`** (standalone): New public function called at deploy time. Generates all 4 artifacts in `~/.fugue/workerd/<name>/`.
- **`spawn_workerd_nuxtjs()`**: Replaced Node.js spawning with `workerd serve config.capnp -s http=*:<port>`.
- **`embed_static_assets()`**: Walks `.output/public/`, base64-encodes files into a JS Map module.
- **`bundle_server_with_esbuild()`**: Runs esbuild with `--external:node:* --external:cloudflare:workers --conditions=workerd --platform=node`.
- **`generate_entry_worker()`**: Writes the routing entry point.
- **`generate_nuxtjs_capnp_config()`**: Writes the Cap'n Proto config.
- **`find_esbuild()`**: Locates esbuild binary (checks `node_modules/.bin`, `npx`, `PATH`).

### `src/commands/mod.rs`

- `deploy_command()`: Calls `generate_nuxtjs_workerd_artifacts()` after build, before registry deployment.

### `src/daemon/server.rs`

- `invoke_handler()`: Passes `~/.fugue/workerd/<name>/` as the workerd func dir.

### `src/runtime/mod.rs`

- Exports `generate_nuxtjs_workerd_artifacts`.

### `Cargo.toml`

- Added `base64 = "0.22"` dependency.

## Storage layout

```
~/.fugue/workerd/<name>/
├── config.capnp          # workerd configuration
├── entry.mjs             # routing entry worker
├── bundle.mjs            # esbuild-bundled Nitro server
└── static-assets.mjs     # base64-encoded public files

functions/<name>/
├── metadata.json
├── source/               # original Nuxt project
└── build/                # .output directory
    ├── server/           # Nitro server (original modules)
    └── public/           # static assets
```

## Dependencies

- **workerd**: `npm install -g workerd` (must be in PATH)
- **esbuild**: `npm install -g esbuild` (or available via `npx`)

## MIME type fix

`std::path::Path::extension()` returns `"js"` (without dot), but the MIME type map uses `".js"` (with dot). The lookup uses `format!(".{}", ext)` to match.

## Testing

```bash
cargo build

# Start daemon
./target/debug/fugue start

# Deploy nuxtjs-simple
./target/debug/fugue deploy nuxt-test examples/nuxtjs-simple/

# Test SSR
./target/debug/fugue invoke nuxt-test

# Test static assets
curl -sI http://127.0.0.1:8180/_nuxt/entry.DTLfg5kr.css
# Content-Type: text/css

curl -sI http://127.0.0.1:8180/_nuxt/CwVcg5pw.js
# Content-Type: application/javascript

# Deploy nuxtjs-app
./target/debug/fugue deploy nuxt-app examples/nuxtjs-app/
./target/debug/fugue invoke nuxt-app
```

## Supersedes

Stage 3's "Node.js Runtime" architecture decision. Nuxt.js now runs on real workerd, matching the production Cloudflare Workers environment.
