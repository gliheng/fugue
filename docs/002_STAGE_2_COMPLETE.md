# Stage 2 Implementation Complete: Next.js Support

## Summary

Successfully implemented Next.js deployment support for Fugue, extending the serverless platform to handle full Next.js applications alongside single-file functions.

## What Was Implemented

### 1. Core Architecture Changes

**Deployment Type System** (`src/registry/metadata.rs`)
- Added `DeploymentType` enum with `SingleFile` and `NextJs` variants
- Extended `FunctionMetadata` with:
  - `deployment_type: DeploymentType`
  - `updated_at: DateTime<Utc>` for tracking rebuilds
  - `environment_vars: HashMap<String, String>` for env var support
- Backward compatible with existing single-file functions

### 2. Next.js Module (`src/nextjs/`)

**Detection** (`detection.rs`)
- `detect_nextjs_project()` - Detects Next.js projects by checking package.json
- `validate_nextjs_project()` - Validates Next.js version >= 13 and project structure
- Checks for app/ or pages/ directories
- Extracts Next.js and Node.js versions

**Builder** (`builder.rs`)
- `build_nextjs_project()` - Orchestrates full build process
- Auto-detects package manager (npm/pnpm/yarn)
- Runs dependency installation and `next build`
- Verifies standalone output exists
- Tracks build time and output size

### 3. Storage Layer Updates (`src/registry/storage.rs`)

- `deploy_nextjs_function()` - Stores Next.js projects with source + build output
- `rebuild_nextjs_function()` - Rebuilds existing deployments
- `copy_dir_recursive()` - Helper for directory operations
- Storage structure:
  ```
  functions/
  ├── hello/              # Single-file
  │   ├── metadata.json
  │   └── code.js
  └── my-app/            # Next.js
      ├── metadata.json
      ├── source/        # Original source
      └── build/         # Build output
  ```

### 4. Runtime Enhancements (`src/runtime/workerd.rs`)

- `generate_nextjs_config()` - Generates workerd config with:
  - `nodejs_compat` compatibility flag
  - Directory bindings for static assets
  - Environment variable injection
  - Standalone server.js entry point
- `get_or_spawn_nextjs()` - Spawns workerd for Next.js apps
- `copy_dir_recursive()` - Copies standalone output to workerd directory

### 5. API Layer (`src/client/api.rs`, `src/daemon/server.rs`)

**New Request Types:**
- `DeployNextJsRequest` - Deploy Next.js with source_dir, skip_build, env_vars
- `RebuildRequest` - Trigger rebuild

**New Endpoints:**
- `POST /api/deploy-nextjs` - Deploy Next.js application
- `POST /api/rebuild/:name` - Rebuild Next.js application
- Updated `POST /api/invoke/:name` - Routes based on deployment type

### 6. CLI Commands (`src/cli.rs`, `src/commands/mod.rs`)

**New Commands:**
```bash
fugue deploy-nextjs <name> <directory> [--skip-build] [--env KEY=VALUE]
fugue rebuild <name>
```

**Command Implementations:**
- `deploy_nextjs_command()` - Validates, detects, builds, and deploys
- `rebuild_command()` - Triggers rebuild of existing deployment

### 7. Configuration & Error Handling

**Config** (`src/config.rs`)
- `MAX_BUILD_TIME_MS: 300_000` (5 minutes)
- `MAX_PROJECT_SIZE: 100MB`
- `SUPPORTED_NODE_VERSIONS: ["18", "20"]`

**Errors** (`src/error.rs`)
- `BuildError` - Build failures
- `NotNextJsProject` - Detection failures
- `UnsupportedNextJsVersion` - Version incompatibility
- `NodeJsError` - Node.js not found

### 8. Example Application

Created `examples/nextjs-app/` with:
- `package.json` - Next.js 14 with standalone output
- `next.config.js` - Configured for standalone mode
- `app/page.js` - Simple home page
- `app/layout.js` - Root layout
- `README.md` - Deployment instructions

## Key Technical Decisions

1. **Store Source + Build Output** - Enables rebuilds without re-uploading
2. **Require Standalone Output** - Next.js standalone mode bundles all dependencies
3. **Build on Daemon** - Consistent environment, dependency caching
4. **Separate Command** - Explicit `deploy-nextjs` vs auto-detection
5. **Full HTTP Forwarding** - Next.js expects standard HTTP, not just JSON

## Backward Compatibility

- Existing single-file functions work unchanged
- Metadata migration: defaults to `DeploymentType::SingleFile`
- Storage layout: single-file functions keep flat structure
- API: existing `/api/deploy` endpoint unchanged
- Invocation: routes based on `deployment_type`

## Usage Examples

### Deploy Single-File Function (Existing)
```bash
fugue start
fugue deploy hello examples/hello.js
fugue invoke hello --data '{"name":"World"}'
```

### Deploy Next.js Application (New)
```bash
fugue deploy-nextjs my-app ./examples/nextjs-app --env API_KEY=secret
fugue invoke my-app
fugue rebuild my-app
```

## Files Modified

1. `Cargo.toml` - Added walkdir, fs_extra dependencies
2. `src/config.rs` - Added Next.js constants
3. `src/error.rs` - Added Next.js error types
4. `src/registry/metadata.rs` - Added DeploymentType enum
5. `src/registry/storage.rs` - Added Next.js deployment methods
6. `src/runtime/workerd.rs` - Added Next.js config generation
7. `src/client/api.rs` - Added Next.js request types
8. `src/daemon/server.rs` - Added Next.js endpoints
9. `src/cli.rs` - Added new commands
10. `src/commands/mod.rs` - Added command implementations
11. `src/main.rs` - Wired up new commands
12. `README.md` - Updated documentation

## Files Created

1. `src/nextjs/mod.rs` - Module declaration
2. `src/nextjs/detection.rs` - Project detection
3. `src/nextjs/builder.rs` - Build orchestration
4. `examples/nextjs-app/package.json`
5. `examples/nextjs-app/next.config.js`
6. `examples/nextjs-app/app/page.js`
7. `examples/nextjs-app/app/layout.js`
8. `examples/nextjs-app/README.md`

## Build Status

✅ Compilation successful (release build)
✅ All commands available in CLI
⚠️ 11 warnings (unused variables/methods - non-critical)

## Next Steps for Testing

1. Start daemon: `./target/release/fugue start`
2. Test single-file deployment (verify backward compatibility)
3. Install dependencies in example app: `cd examples/nextjs-app && npm install`
4. Deploy Next.js app: `./target/release/fugue deploy-nextjs test-app ./examples/nextjs-app`
5. Invoke and verify response
6. Test rebuild functionality
7. Test environment variables

## Known Limitations

- Build requires Node.js/npm on daemon host
- No build caching yet (rebuilds from scratch)
- No build logs streaming to CLI
- workerd processes not cleaned up on function delete
- No timeout enforcement on builds

## Future Enhancements

- Build caching for faster rebuilds
- Build logs streaming
- Static asset optimization
- Edge runtime support
- ISR (Incremental Static Regeneration)
- Middleware support
