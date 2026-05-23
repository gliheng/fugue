#!/usr/bin/env bash
set -euo pipefail

# Build the Nuxt.js app for workerd with static asset support.
#
# This script:
#   1. Runs `npm run build` to produce .output/
#   2. Embeds static assets from .output/public/ into static-assets.mjs
#   3. Bundles the SSR server code into a single ES module
#   4. Produces workerd.capnp (with ASSETS binding) ready for `workerd serve`
#
# Usage:
#   ./build-workerd.sh           # Build only
#   ./build-workerd.sh --run     # Build and start workerd

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

PORT="${PORT:-8787}"

echo "==> Building Nuxt app..."
npm run build

echo "==> Embedding static assets..."
node embed-assets.mjs

echo "==> Bundling SSR server with esbuild..."
npx esbuild .output/server/index.mjs \
  --bundle --format=esm \
  --outfile=.output/server/bundle.mjs \
  --external:'node:*' --external:'cloudflare:workers' \
  --conditions=workerd --platform=node

echo "==> Done. Files ready:"
echo "    .output/server/bundle.mjs   (SSR bundle)"
echo "    .output/server/entry.mjs    (router entry)"
echo "    static-assets.mjs           (embedded assets)"
echo "    workerd.capnp               (config)"

if [ "${1:-}" = "--run" ]; then
  echo ""
  echo "==> Starting workerd on http://localhost:$PORT"
  exec workerd serve workerd.capnp -s "http=*:$PORT" --verbose
fi
