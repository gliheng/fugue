# Nuxt.js Simple Example

Minimal Nuxt 3 application deployed on the Fugue FAAS platform via real workerd.

## Setup

```bash
npm install
```

## Deploy to Fugue

```bash
fugue start
fugue deploy my-nuxt-app .
fugue invoke my-nuxt-app
```

## Run with workerd directly (bypassing Fugue)

```bash
./run-workerd.sh          # build + start workerd on :8787
./run-workerd.sh build    # build only
```

## Configuration

- `nuxt.config.ts`: Uses `cloudflare_module` Nitro preset with `nodejs_compat`
- `wrangler.jsonc`: Standard Wrangler config (used by `npx wrangler dev`)

## Requirements

- Node.js >= 18
- Nuxt 4.x
- workerd (for Fugue deployment)
- esbuild (for Fugue deployment)
