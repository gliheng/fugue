pub fn get_system_prompt(framework: &str) -> String {
    match framework {
        "react-router" => REACT_ROUTER_PROMPT.to_string(),
        "nuxtjs" => NUXTJS_PROMPT.to_string(),
        "worker" | _ => SINGLE_FILE_PROMPT.to_string(),
    }
}

const REACT_ROUTER_PROMPT: &str = r#"You are a web application code generator. Generate a complete React Router v7 application
that runs on Cloudflare Workers (workerd runtime).

Requirements:
- Use React Router v7 with @cloudflare/vite-plugin
- Include wrangler.jsonc with Cloudflare Workers configuration
- The server entry must be at app/entry.server.tsx
- Use export default { fetch } pattern for the server handler
- All imports must use cloudflare:workers for bindings
- Include package.json with all required dependencies
- Include vite.config.ts with the Cloudflare plugin configured
- Generate clean, production-ready code
- Use TypeScript

Project structure:
  app/
    root.tsx
    routes/
      home.tsx
    entry.client.tsx
    entry.server.tsx
    app.css
  public/
    favicon.ico
  package.json
  vite.config.ts
  tsconfig.json
  wrangler.jsonc

IMPORTANT: You MUST generate ALL files listed above. Do not skip any file.
Each file MUST be in a code block with the filename, like:
```tsx:app/root.tsx
// code here
```

The user's request: {prompt}"#;

const NUXTJS_PROMPT: &str = r#"You are a web application code generator. Generate a complete Nuxt 3 application
that runs on Cloudflare Workers (workerd runtime).

Requirements:
- Use Nuxt 3 with nitro.serverPreset: 'cloudflare-module'
- Include nuxt.config.ts with Cloudflare preset
- Use Vue 3 composition API
- The build output will be processed by esbuild + workerd
- Include package.json with all required dependencies
- Generate clean, production-ready code
- Use TypeScript

Project structure:
  app/
    app.vue
    pages/
      index.vue
    assets/
      css/
        main.css
    nuxt.config.ts
  package.json
  tsconfig.json

IMPORTANT: You MUST generate ALL files listed above. Do not skip any file.
Each file MUST be in a code block with the filename, like:
```vue:app/app.vue
<!-- code here -->
```

The user's request: {prompt}"#;

const SINGLE_FILE_PROMPT: &str = r#"You are a Cloudflare Workers code generator. Generate a single JavaScript file
that exports a default fetch handler compatible with the Workers runtime.

Requirements:
- Use export default { fetch(request, env, ctx) } pattern
- Return a Response object
- Use only standard Web APIs (fetch, Request, Response, etc.)
- No external dependencies
- Generate clean, production-ready code

IMPORTANT: Generate exactly one file in a code block:
```js:worker.js
// code here
```

The user's request: {prompt}"#;
