# Nuxt.js Example App for Fugue

This is a minimal Nuxt 3 application configured to run on the Fugue FAAS platform.

## Setup

```bash
npm install
```

## Development

```bash
npm run dev
```

## Build

```bash
npm run build
```

This will generate the `.output` directory with the Nitro server build.

## Deploy to Fugue

```bash
# Start the Fugue daemon
fugue start

# Deploy the Nuxt app
fugue deploy my-nuxt-app .

# Invoke the function
fugue invoke my-nuxt-app
```

## Configuration

The `nuxt.config.ts` file is configured with:
- `nitro.preset: 'node-server'` - Ensures the build output is compatible with Node.js runtime
- `compatibilityDate: '2024-01-01'` - Sets the compatibility date for Nuxt features

## Requirements

- Node.js >= 18
- Nuxt 3.x
