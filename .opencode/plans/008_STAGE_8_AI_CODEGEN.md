# Stage 8: AI Code Generation

## Overview

Stage 8 integrates an OpenAI-compatible LLM API to generate complete project code from natural language prompts. Users describe what they want, the AI generates a full project (with correct framework configuration), and one click deploys it to workerd.

The AI engine uses framework-specific system prompts to ensure generated code is workerd-compatible (e.g., React Router projects include `wrangler.jsonc` and `@cloudflare/vite-plugin`).

## Architecture

```
User (Dashboard or CLI)
  │
  │ POST /api/v1/ai/generate
  │ Body: { prompt: "创建一个博客", framework: "react-router" }
  │
  ▼
Fugue Platform
  │
  ├── AI Engine (src/ai/)
  │     ├── OpenAI-compatible client
  │     ├── Framework-specific system prompts
  │     ├── Code parser (extract files from AI response)
  │     └── Validator (check project structure)
  │
  ├── Generation Flow:
  │     1. Select system prompt by framework
  │     2. Call LLM API → streaming response
  │     3. Parse response into files (detect code blocks, filenames)
  │     4. Validate project structure
  │     5. Save to data/apps/<id>/source/
  │     6. (Optional) Auto-deploy: enqueue BuildTask
  │
  └── PostgreSQL
        └── ai_generations table (prompt, result_code, status)
```

## Database Schema

Already defined in Stage 6:

```sql
CREATE TABLE ai_generations (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    app_id      UUID REFERENCES apps(id) ON DELETE SET NULL,
    prompt      TEXT NOT NULL,
    framework   TEXT NOT NULL,
    result_code TEXT,             -- Generated project files as JSON
    status      TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending', 'generating', 'success', 'failed')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## AI Client Design

```rust
pub struct AiClient {
    api_base: String,           // e.g. "https://api.openai.com/v1"
    api_key: String,
    model: String,              // e.g. "gpt-4o"
    client: reqwest::Client,
}

pub struct GenerateRequest {
    pub prompt: String,
    pub framework: Framework,
    pub app_id: Option<Uuid>,   // If provided, auto-deploy after generation
}

pub struct GenerateResponse {
    pub generation_id: Uuid,
    pub files: HashMap<String, String>,  // filename → content
    pub app_id: Option<Uuid>,
}
```

## System Prompts

### React Router System Prompt

```
You are a web application code generator. Generate a complete React Router v7 application
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

The user's request: {prompt}
```

### Nuxt.js System Prompt

```
You are a web application code generator. Generate a complete Nuxt 3 application
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

The user's request: {prompt}
```

### Single File System Prompt

```
You are a Cloudflare Workers code generator. Generate a single JavaScript file
that exports a default fetch handler compatible with the Workers runtime.

Requirements:
- Use export default { fetch(request, env, ctx) } pattern
- Return a Response object
- Use only standard Web APIs (fetch, Request, Response, etc.)
- No external dependencies
- Generate clean, production-ready code

The user's request: {prompt}
```

## Code Parsing

AI responses typically contain multiple code blocks. The parser must:

1. Detect markdown code blocks with filenames (e.g., ` ```tsx:app/root.tsx `)
2. Detect code blocks without filenames (infer from context)
3. Handle the common `===` filename pattern some models use
4. Validate the generated project structure matches the framework requirements

```rust
pub fn parse_ai_response(response: &str, framework: &Framework) -> Result<HashMap<String, String>> {
    let mut files = HashMap::new();

    // Pattern 1: ```lang:filename
    for cap in CODE_BLOCK_FILENAME_REGEX.captures_iter(response) {
        let filename = cap[1].to_string();
        let content = cap[2].to_string();
        files.insert(filename, content);
    }

    // Pattern 2: === filename === (some models)
    // Pattern 3: // filename header comment

    // Validate minimum required files exist
    validate_project_structure(&files, framework)?;

    Ok(files)
}
```

## API Endpoints

```
POST   /api/v1/ai/generate
  Body: {
    prompt: "创建一个博客网站，支持文章列表和详情页",
    framework: "react-router",      // or "nuxtjs" or "single-file"
    auto_deploy: true               // optional, auto-deploy after generation
  }
  Response: { generation_id, app_id, status: "generating" }

POST   /api/v1/ai/generate (SSE variant)
  Accept: text/event-stream
  Body: same as above
  Response: streaming tokens → code blocks → completion

GET    /api/v1/ai/generations/:id
  Response: { id, prompt, framework, files, status, app_id }

POST   /api/v1/ai/generations/:id/deploy
  Body: {}
  Response: { build_id, deployment_id, status: "pending" }
```

## New Files

```
src/
├── ai/
│   ├── mod.rs               -- AI module
│   ├── client.rs             -- OpenAI-compatible API client
│   ├── prompts.rs            -- Framework-specific system prompts
│   ├── parser.rs             -- Parse AI response into file map
│   └── validator.rs           -- Validate generated project structure
└── api/
    └── ai.rs                 -- AI generation API handlers
```

## Configuration

```toml
# ~/.fugue/config.toml (additions)
[ai]
provider = "openai"            # or "anthropic", "custom"
api_base = "https://api.openai.com/v1"
api_key = ""                   # or env var FUGUE_AI_API_KEY
model = "gpt-4o"              # or any OpenAI-compatible model
max_tokens = 16384
temperature = 0.7
```

## Streaming Generation

For the Dashboard (Stage 7), we want real-time streaming of AI output:

```rust
pub async fn generate_stream(
    client: &AiClient,
    request: GenerateRequest,
) -> impl Stream<Item = Result<GenerateEvent>> {
    // Use SSE (Server-Sent Events) from OpenAI API
    // Parse events into GenerateEvent types:
    //   - Token(text) — partial output
    //   - FileStart(filename) — starting a new file
    //   - FileContent(filename, content) — file content
    //   - Done(generation_id) — generation complete
}
```

The frontend can connect via SSE and render the code as it generates.

## Generation + Deploy Flow

```
User → POST /api/v1/ai/generate { prompt, framework, auto_deploy: true }
  │
  ├── 1. Create ai_generations record (status: generating)
  ├── 2. Create app record (status: created)
  ├── 3. Call LLM API → get generated files
  ├── 4. Parse files → validate structure
  ├── 5. Write files to data/apps/<app_id>/source/
  ├── 6. Update ai_generations (status: success)
  ├── 7. If auto_deploy:
  │     ├── Create build record (status: pending)
  │     ├── Enqueue BuildTask
  │     └── Return { generation_id, app_id, build_id }
  └── Else:
        └── Return { generation_id, app_id }
```

## Modified Files

```
Cargo.toml                    -- Add eventsource-stream or reqwest SSE features
src/api/mod.rs                -- Mount AI routes
src/daemon/server.rs           -- Add AI endpoints
src/daemon/state.rs            -- Add AiClient to AppState
src/config/mod.rs              -- Add AI config section
```

## Testing

```bash
# 1. Set AI API key
export FUGUE_AI_API_KEY=sk-...

# 2. Generate a React Router app
curl -X POST http://localhost:3000/api/v1/ai/generate \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "创建一个个人博客，支持文章列表和Markdown渲染",
    "framework": "react-router",
    "auto_deploy": true
  }'
# → { "generation_id": "...", "app_id": "...", "build_id": "...", "status": "generating" }

# 3. Check generation status
curl http://localhost:3000/api/v1/ai/generations/<id>
# → { "status": "success", "files": {...} }

# 4. Check build status (if auto_deploy)
curl http://localhost:3000/api/v1/apps/<app_id>/status
# → { "status": "running", "url": "http://my-blog.fugue.local:3000" }

# 5. Generate without auto-deploy
curl -X POST http://localhost:3000/api/v1/ai/generate \
  -d '{" "prompt": "一个简单的API，返回当前时间", "framework": "single-file" }'
# → { "generation_id": "...", "app_id": "..." }

# 6. Manually deploy later
curl -X POST http://localhost:3000/api/v1/ai/generations/<id>/deploy
```

## Dependencies

- Stage 6 (Platform API) — must be complete for API endpoints and PostgreSQL
- Stage 9 (Async Build) — must be complete for auto-deploy flow
- OpenAI-compatible API endpoint (can be OpenAI, Claude via proxy, or local model)