# Fugue

A Rust-based serverless platform POC using V8 isolates via workerd.

## Architecture

Fugue uses a daemon architecture for optimal performance:

- **CLI Client**: Lightweight command-line tool that sends commands to daemon
- **Daemon Server**: Long-running background process managing workerd instances
- **workerd**: Cloudflare's Workers runtime for V8 isolation
- **Function Registry**: Filesystem-based storage for functions

## Prerequisites

1. Install Rust (https://rustup.rs/)
2. Install workerd: `npm install -g workerd`

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

### Invoke a function
```bash
fugue invoke hello --data '{"name":"World"}'
```

### List functions
```bash
fugue list
```

### View logs
```bash
fugue logs hello
```

### Delete a function
```bash
fugue delete hello
```

### Stop the daemon
```bash
fugue stop
```

## Function Format

### Single-File Functions

Functions should export a handler:

```javascript
export function handler(event) {
  return {
    message: "Hello " + (event.name || "World"),
    timestamp: Date.now()
  };
}
```

Or use Cloudflare Workers format:

```javascript
export default {
  async fetch(request, env, ctx) {
    return new Response(JSON.stringify({ message: "Hello" }), {
      headers: { 'Content-Type': 'application/json' }
    });
  }
}
```

## Project Status

**Phase 1 (Complete)**: Basic infrastructure
- ✅ Project setup and dependencies
- ✅ CLI interface
- ✅ Function registry
- ✅ Client API
- ✅ Daemon server
- ✅ workerd integration
- ✅ Single-file function deployment

**Future Enhancements**:
- Logs collection and viewing
- Timeout enforcement
- Memory limits
- Health checks for workerd processes
- Metrics and observability

## Performance Targets

- Cold start: <50ms
- Warm start: <5ms
- Simple function: <10ms end-to-end

## License

MIT
