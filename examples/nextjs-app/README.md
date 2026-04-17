# Next.js Example App for Fugue

This is a Next.js application with HeroUI v3 configured to run on the Fugue FAAS platform.

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

The application uses:
- Next.js 16 with App Router
- HeroUI v3 (Beta) for UI components
- Tailwind CSS v4 for styling
- TypeScript for type safety

## Requirements

- Node.js >= 18
- Next.js 16.x
- HeroUI v3 (Beta)
