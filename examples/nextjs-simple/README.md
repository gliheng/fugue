# Next.js Example App for Fugue

This is a minimal Next.js application configured to run on the Fugue FAAS platform.

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

This will generate the `.next` directory with the production build.

## Deploy to Fugue

```bash
# Start the Fugue daemon
fugue start

# Deploy the Next.js app
fugue deploy my-nextjs-app .

# Invoke the function
fugue invoke my-nextjs-app
```

## Configuration

The `next.config.ts` file is configured with:
- `output: 'standalone'` - Ensures the build output is optimized for serverless deployment
- Minimal configuration for compatibility with Node.js runtime

## Requirements

- Node.js >= 18
- Next.js 15.x
- React 19.x
