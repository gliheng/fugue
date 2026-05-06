# Stage 5: Real workerd for React Router

## Overview

Stage 5 adds React Router v7 (Cloudflare Workers) support to Fugue. React Router apps now run on real `workerd`, using the same 3-service architecture as Nuxt.js (Stage 4).

## Problem

Fugue only supported single-file functions and Nuxt.js. React Router v7 with `@cloudflare/vite-plugin` produces a Cloudflare Workers-compatible build, but the output consists of multiple ES modules with relative imports — the same issue Nuxt.js had.

## Solution

At deploy time, the platform:

1. **Detects React Router** — checks `package.json` for `react-router` dependency and `wrangler.jsonc` for Cloudflare config
2. **Builds the project** — runs `npm run build` (which calls `react-router build`)
3. **Embeds static assets** — reads `build/client/`, base64-encodes every file into a `static-assets.mjs` JS module
4. **Bundles the SSR server** — runs esbuild to collapse the server modules into a single `bundle.mjs`, marking `node:*` and `cloudflare:workers` as external
5. **Generates an entry worker** — `entry.mjs` routes requests: static assets from the embedded map, everything else to the SSR worker via service binding
6. **Generates `config.capnp`** — a 3-service workerd config (entry, SSR, static) with `nodejs_compat`

## Architecture

```
fugue deploy
  ├── react-router build   → build/
  ├── embed static assets  → ~/.fugue/workerd/<name>/static-assets.mjs
  ├── esbuild bundle       → ~/.fugue/workerd/<name>/bundle.mjs
  ├── generate entry       → ~/.fugue/workerd/<name>/entry.mjs
  └── generate capnp       → ~/.fugue/workerd/<name>/config.capnp

fugue invoke
  └── workerd serve config.capnp
        ├── entry worker   (routes /assets/* → static, rest → SSR)
        ├── SSR worker     (React Router server, env.ASSETS → static service)
        └── static worker  (serves embedded client files)
```

### workerd config (config.capnp)

Identical to Nuxt.js — 3 services with `nodejs_compat`:

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

### Differences from Nuxt.js

| Aspect | Nuxt.js | React Router |
|--------|---------|--------------|
| Detection | `nuxt` in package.json + `nuxt.config.ts` | `react-router` in package.json + `wrangler.jsonc` |
| Build command | `nuxt build` | `react-router build` |
| Build output | `.output/server/` + `.output/public/` | `build/server/` + `build/client/` |
| Server entry | `.output/server/index.mjs` | `build/server/index.js` |
| Static asset paths | `/_nuxt/*` | `/assets/*` + `/favicon.ico` |

## Files changed

### `src/reactrouter/` (new module)

- **`detection.rs`**: `detect_reactrouter_project()` — checks for `react-router` dependency and `wrangler.jsonc`
- **`builder.rs`**: `build_reactrouter_project()` — detects package manager, runs install + build
- **`mod.rs`**: Exports public functions

### `src/runtime/workerd.rs`

- **`generate_reactrouter_workerd_artifacts()`**: New public function called at deploy time. Generates all 4 artifacts in `~/.fugue/workerd/<name>/`.
- **`bundle_reactrouter_server_with_esbuild()`**: Runs esbuild on `build/server/index.js` with `--external:node:* --external:cloudflare:workers --conditions=workerd --platform=node`.
- **`generate_reactrouter_entry_worker()`**: Writes the routing entry point. Serves embedded assets directly, delegates everything else to SSR.
- **`generate_reactrouter_capnp_config()`**: Writes the Cap'n Proto config (identical structure to Nuxt.js).
- **`get_or_spawn_reactrouter()`** on `WorkerdPool`: Delegates to `spawn_workerd_nuxtjs` since the capnp format is identical.

### `src/registry/metadata.rs`

- Added `DeploymentType::ReactRouter` variant with `build_output_path` and `node_version`.

### `src/commands/mod.rs`

- `deploy_command()`: After Nuxt.js detection fails, tries React Router detection. Builds, validates, generates artifacts, deploys.

### `src/daemon/server.rs`

- `invoke_handler()`: Added `ReactRouter` match arm. Same pattern as NuxtJs — spawns workerd, forwards HTTP request.

### `src/registry/storage.rs`

- Added `deploy_reactrouter_function()` method.
- `get_function()`: Handles `ReactRouter` variant (returns empty code like NuxtJs).

### `src/error.rs`

- Added `NotReactRouterProject` error variant.

### `src/main.rs`

- Added `mod reactrouter;`

### `src/cli.rs`

- Updated help text to mention React Router.

## Storage layout

```
~/.fugue/workerd/<name>/
├── config.capnp          # workerd configuration
├── entry.mjs             # routing entry worker
├── bundle.mjs            # esbuild-bundled server
└── static-assets.mjs     # base64-encoded client files

functions/<name>/
├── metadata.json
├── source/               # original React Router project
└── build/                # build output
    ├── server/           # server bundle
    └── client/           # static assets
```

## Testing

```bash
cargo build

# Start daemon
./target/debug/fugue start

# Deploy react-router-app
./target/debug/fugue deploy rr-test examples/react-router-app/

# Test SSR
./target/debug/fugue invoke rr-test

# Test static assets
curl -sI http://127.0.0.1:8180/assets/root-BRsUeuG4.css
# Content-Type: text/css

curl -sI http://127.0.0.1:8180/assets/entry.client-BYHQY4F6.js
# Content-Type: application/javascript

curl -sI http://127.0.0.1:8180/favicon.ico
# Content-Type: image/x-icon
```

## Dependencies

- **workerd**: `npm install -g workerd` (must be in PATH)
- **esbuild**: `npm install -g esbuild` (or available via `npx`)
- **react-router project**: Must have `@cloudflare/vite-plugin` in `vite.config.ts` and `wrangler.jsonc` in root
