# Fugue

A Rust-based serverless platform POC using V8 isolates via workerd.

## Architecture

Fugue uses a daemon architecture for optimal performance:

```
CLI Client → HTTP (localhost:7878) → Daemon Server → workerd Process Pool → V8 Isolates
```

- **CLI Client**: Lightweight command-line tool that sends commands to daemon
- **Daemon Server**: Long-running background process managing workerd instances
- **workerd**: Cloudflare's Workers runtime for V8 isolation
- **Function Registry**: Filesystem-based storage for functions and build artifacts

## Prerequisites

1. Install Rust (https://rustup.rs/)
2. Install workerd: `npm install -g workerd`
3. Install esbuild (for Nuxt.js): `npm install -g esbuild`

## Installation

```bash
cargo build --release
```

## Usage

### Start the daemon
```bash
fugue start
```

### Deploy a single-file function
```bash
fugue deploy hello examples/hello.js
```

### Deploy a Nuxt.js app
```bash
fugue deploy my-nuxt-app examples/nuxtjs-simple/
```

Each project must include a `fugue.toml` with the `framework` set:

```toml
framework = "nuxtjs"

[assets]
dir = ".output/public"
prefix = "/_nuxt/"

[build]
output_dir = ".output"
server_entry = "server/index.mjs"
```

Fugue reads `fugue.toml`, builds the project, generates workerd artifacts (esbuild bundle + static assets + Cap'n Proto config), and deploys.

### Invoke a function
```bash
fugue invoke hello --data '{"name":"World"}'
fugue invoke my-nuxt-app
```

### List functions
```bash
fugue list
```

### Delete a function
```bash
fugue delete hello
```

### Stop the daemon
```bash
fugue stop
```

## Function Formats

### Single-File Functions

Functions should export a Cloudflare Workers handler:

```javascript
export default {
  async fetch(request, env, ctx) {
    return new Response(JSON.stringify({ message: "Hello" }), {
      headers: { 'Content-Type': 'application/json' }
    });
  }
}
```

### Nuxt.js Applications

Any Nuxt 3+ project with `cloudflare_module` preset:

```typescript
// nuxt.config.ts
export default defineNuxtConfig({
  nitro: {
    preset: 'cloudflare_module',
    cloudflare: { deployConfig: true, nodeCompat: true }
  }
})
```

Add `fugue.toml`:

```toml
framework = "nuxtjs"
```

The deploy command:
1. Runs `npm run build` (generates `.output/`)
2. Embeds `.output/public/` static assets into a JS module
3. Bundles `.output/server/` into a single ES module via esbuild
4. Generates a 3-service workerd Cap'n Proto config
5. Stores artifacts in `~/.fugue/workerd/<name>/`

## Project Status

**Phase 1 (Complete)**: Basic infrastructure
- CLI interface, daemon server, workerd integration
- Single-file function deployment and invocation

**Phase 2 (Complete)**: Nuxt.js support
- Auto-detection and build pipeline
- Real workerd runtime (not Node.js)
- Static asset serving with correct MIME types
- 3-service Cap'n Proto config (entry + SSR + static)

## Examples

| Example | Description |
|---------|-------------|
| `examples/hello.js` | Single-file Cloudflare Workers function |
| `examples/nuxtjs-simple/` | Minimal Nuxt 3 app |
| `examples/nuxtjs-app/` | Full Nuxt app with Nuxt UI, Tailwind, pages |

## Performance

- Cold start: ~100ms (spawn workerd + V8 isolate)
- Warm invocation: <10ms (reuse existing workerd)
- Static assets: served from embedded base64 map (no disk I/O at runtime)

## License

MIT
