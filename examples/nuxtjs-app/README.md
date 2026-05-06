# Nuxt.js App Example

Full-featured Nuxt 4 application with Nuxt UI, Tailwind CSS, and multi-page routing. Deployed on the Fugue FAAS platform via real workerd.

## Setup

```bash
npm install
```

## Deploy to Fugue

```bash
fugue start
fugue deploy nuxt-app .
fugue invoke nuxt-app
```

## Run with workerd directly (bypassing Fugue)

```bash
./run-workerd.sh          # build + start workerd on :8787
./run-workerd.sh build    # build only
```

## Pages

- `/` — Home page with feature cards
- `/about` — About page
- `/features` — Feature list
- `/contact` — Contact form

## Configuration

- `nuxt.config.ts`: Uses `cloudflare_module` Nitro preset with `nodejs_compat`, Nuxt UI module
- `wrangler.jsonc`: Standard Wrangler config (used by `npx wrangler dev`)

## Requirements

- Node.js >= 18
- Nuxt 4.x
- workerd (for Fugue deployment)
- esbuild (for Fugue deployment)
