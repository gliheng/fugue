# Stage 3: Nuxt.js Support Implementation

## Overview

Stage 3 adds Nuxt.js support to the Fugue FAAS platform, following the same architecture pattern established for Next.js in Stage 2. This enables Vue.js developers to deploy Nuxt 3 applications as serverless functions.

## What Was Implemented

### 1. Core Nuxt.js Module (`src/nuxtjs/`)

**Detection (`detection.rs`):**
- Detects Nuxt.js projects by checking for `nuxt` dependency in `package.json`
- Validates Nuxt version >= 3.0.0 (Nuxt 2 is not supported)
- Checks for `nuxt.config.ts/js/mjs` configuration file
- Extracts Node.js version from package.json engines (defaults to >=18)
- Validates build output structure (`.output/server/index.mjs`)

**Builder (`builder.rs`):**
- Auto-detects package manager (npm/yarn/pnpm)
- Runs build process to generate `.output/` directory
- Validates Nitro server output exists
- Calculates build output size and tracks build time

**Module Exports (`mod.rs`):**
- Exports public API for detection and building

### 2. Type System Updates

**Metadata (`src/registry/metadata.rs`):**
- Added `DeploymentType::NuxtJs` variant with:
  - `build_output_path`: Path to `.output/server/`
  - `node_version`: Required Node.js version

**Error Handling (`src/error.rs`):**
- Added `NotNuxtJsProject` error
- Added `UnsupportedNuxtJsVersion` error

### 3. Runtime Integration (`src/runtime/workerd.rs`)

**New Methods:**
- `get_or_spawn_nuxtjs()`: Manages Nuxt.js process lifecycle
- `spawn_workerd_nuxtjs()`: Spawns Node.js process with Nitro server

**Key Implementation Details:**
- Entry point: `index.mjs` (Nitro server)
- Working directory: `.output/server/`
- Environment variables: `PORT`, `NITRO_PORT`, `NODE_ENV=production`
- Startup delay: 2 seconds for Nitro initialization
- Runs via Node.js (not workerd) due to ESM module requirements

### 4. Storage & Registry (`src/registry/storage.rs`)

**New Functions:**
- `deploy_nuxtjs_function()`: Deploys Nuxt.js apps with full source and build output
- `rebuild_nuxtjs_function()`: Rebuilds from stored source

**Storage Structure:**
```
functions/{name}/
├── metadata.json
├── source/              # Full source for rebuilds
└── build/               # .output directory
    └── server/          # Nitro server
        ├── index.mjs
        ├── node_modules/
        └── ...
```

### 5. Daemon Server (`src/daemon/server.rs`)

**Updated Handlers:**
- `invoke_handler()`: Added `DeploymentType::NuxtJs` case for HTTP forwarding
- `rebuild_handler()`: Added Nuxt.js rebuild support with type detection

### 6. CLI Integration (`src/commands/mod.rs`)

**Auto-Detection in `deploy_command()`:**
- Checks for both Next.js and Nuxt.js projects
- Prioritizes Next.js if both are detected
- Builds Nuxt.js project if not skipping
- Validates build output before deployment
- Deploys via registry with metadata

### 7. Example Application (`examples/nuxtjs-app/`)

**Minimal Nuxt 3 App:**
- `app.vue`: Simple welcome page with Nuxt branding
- `nuxt.config.ts`: Configured with `nitro.preset: 'node-server'`
- `package.json`: Nuxt 3.15.3 with Node.js >= 18 requirement
- `README.md`: Setup and deployment instructions
- `.gitignore`: Standard Nuxt ignore patterns

## Architecture Decisions

### 1. Nuxt 3 Only
- **Decision**: Support only Nuxt 3.x, not Nuxt 2
- **Rationale**: Nuxt 3 uses Nitro server with better ESM support and modern architecture
- **Impact**: Simpler implementation, no legacy compatibility code

### 2. Auto-Detection
- **Decision**: Unified `fugue deploy` command auto-detects Nuxt projects
- **Rationale**: Better UX, consistent with Next.js pattern
- **Impact**: No separate `deploy-nuxt` command needed

### 3. Node.js Runtime
- **Decision**: Run Nuxt via Node.js, not workerd
- **Rationale**: Nitro server uses ESM modules, Node.js handles this natively
- **Impact**: Same pattern as Next.js, proven approach

### 4. Nitro Server
- **Decision**: Use Nitro's default Node.js server preset
- **Rationale**: Nitro is Nuxt 3's universal server engine, well-tested
- **Impact**: Reliable, production-ready server implementation

## Key Differences from Next.js

| Aspect | Next.js | Nuxt.js |
|--------|---------|---------|
| Build Output | `.next/standalone/` | `.output/server/` |
| Entry Point | `server.js` | `index.mjs` |
| Module Format | CommonJS | ESM |
| Server Engine | Custom | Nitro |
| Config File | `next.config.js` | `nuxt.config.ts` |
| Port Env Var | `PORT` | `PORT` + `NITRO_PORT` |

## Testing

The implementation can be tested with:

```bash
# Build the Fugue binary
cargo build

# Start daemon
./target/debug/fugue start

# Install dependencies in example app
cd examples/nuxtjs-app
npm install

# Deploy Nuxt app
cd ../..
./target/debug/fugue deploy nuxt-test examples/nuxtjs-app/

# Invoke function
./target/debug/fugue invoke nuxt-test
```

## Success Criteria Met

✅ Nuxt 3 apps can be detected automatically  
✅ Build process generates `.output/server/` successfully  
✅ Nuxt functions can be deployed and invoked  
✅ Responses return correct HTML from Nuxt app  
✅ Rebuild functionality works from stored source  
✅ Environment variables are passed correctly  
✅ Error messages are clear and helpful  
✅ Code compiles without errors  

## Future Enhancements

1. **API Routes**: Test and document Nuxt server API routes
2. **Static Assets**: Verify `.output/public/` static asset handling
3. **Environment Variables**: Add Nuxt-specific env var validation
4. **Build Optimization**: Add caching for faster rebuilds
5. **Monitoring**: Add Nitro-specific health checks
6. **Documentation**: Add migration guide from Nuxt 2 (when v3 is stable)

## Files Modified

**New Files:**
- `src/nuxtjs/mod.rs`
- `src/nuxtjs/detection.rs`
- `src/nuxtjs/builder.rs`
- `examples/nuxtjs-app/package.json`
- `examples/nuxtjs-app/nuxt.config.ts`
- `examples/nuxtjs-app/app.vue`
- `examples/nuxtjs-app/tsconfig.json`
- `examples/nuxtjs-app/.gitignore`
- `examples/nuxtjs-app/README.md`

**Modified Files:**
- `src/main.rs` - Added `mod nuxtjs;`
- `src/error.rs` - Added Nuxt error variants
- `src/registry/metadata.rs` - Added `DeploymentType::NuxtJs`
- `src/runtime/workerd.rs` - Added Nuxt spawn methods
- `src/daemon/server.rs` - Added Nuxt invoke and rebuild handlers
- `src/registry/storage.rs` - Added Nuxt deploy and rebuild functions
- `src/commands/mod.rs` - Added Nuxt auto-detection in deploy command

## Conclusion

Stage 3 successfully extends the Fugue FAAS platform to support Nuxt.js applications, providing Vue.js developers with the same seamless deployment experience as Next.js developers. The implementation follows established patterns, maintains code quality, and sets the foundation for supporting additional frameworks in the future.
