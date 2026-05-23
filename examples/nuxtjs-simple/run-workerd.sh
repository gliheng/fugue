#!/usr/bin/env bash
set -euo pipefail

# Run the Nuxt.js example with workerd directly (bypassing wrangler).
#
# Usage:
#   ./run-workerd.sh             # Build and start workerd
#   ./run-workerd.sh build       # Build only, don't start workerd
#
# Static assets from .output/public/ are embedded into the worker
# so CSS/JS files load correctly (no more 404s on /_nuxt/*).

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

if [ "${1:-}" = "build" ]; then
  exec ./build-workerd.sh
else
  exec ./build-workerd.sh --run
fi
