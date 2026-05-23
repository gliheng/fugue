# Stage 7: Dashboard (React Router v7 on workerd)

## Overview

Stage 7 creates the web Dashboard — a React Router v7 application that runs on Fugue itself (self-hosting on workerd). The Dashboard provides:

1. App management (create, view, edit, delete)
2. Code editor (Monaco Editor for editing source files)
3. AI code generation (prompt → code → deploy)
4. Build log streaming (WebSocket)
5. Runtime status (running/stopped, URL, uptime)

The Dashboard is deployed as a Fugue app itself, running at `dashboard.fugue.local`.

## Architecture

```
Browser
  │
  ├── dashboard.fugue.local:3000 → Fugue Axum Proxy → workerd (Dashboard SSR)
  │
  │   Dashboard (React Router v7 on workerd)
  │   ┌───────────────────────────────────────────┐
  │   │  Pages:                                    │
  │   │    /apps          App list                  │
  │   │    /apps/:id      App detail + code editor  │
  │   │    /apps/:id/deploy  Deploy management     │
  │   │    /ai            AI code generation        │
  │   └───────────────────────────────────────────┘
  │         │
  │         │ REST API calls
  │         ▼
  │   Fugue Platform API (localhost:3000/api/v1/*)
  │         │
  │         │ WebSocket
  │         ▼
  │   Build log streaming
  │
  ├── my-app.fugue.local:3000 → workerd → user's app
  └── another.fugue.local:3000 → workerd → another app
```

## Why React Router v7?

The Dashboard itself is a React Router v7 app running on workerd — the same runtime that hosts user apps. This is self-bootstrapping:

1. React Router v7 is already supported (Stage 5)
2. It has a rich ecosystem (shadcn/ui, Monaco Editor via npm)
3. SSR provides fast initial load + good SEO (not critical but nice)
4. It validates that the Fugue platform works for real-world apps

## Technology Stack

| Layer | Technology | Reason |
|-------|-----------|--------|
| Framework | React Router v7 | Already supported on workerd |
| UI Components | HeroUI v3 | Modern component library with compound component pattern |
| Code Editor | @monaco-editor/react | VS Code-quality editor |
| Styling | Tailwind CSS v4 | Already used by React Router template |
| State | React Query (TanStack Query) | Server state management, caching |
| WebSocket | Native browser API | Build log streaming |
| HTTP Client | Built-in fetch | API calls |

> **Note on HeroUI v3**: Uses compound component pattern (e.g., `Card.Header`, `Modal.Backdrop`, `Select.Trigger`). No `HeroUIProvider` is needed — components work standalone with CSS imports. Selectable options use `ToggleButtonGroup`/`ToggleButton` instead of `Chip` with `onPress`. Modal uses controlled `Modal.Backdrop` with `isOpen`/`onOpenChange`.

## Directory Structure

```
dashboard/
├── app/
│   ├── root.tsx                 # Root layout
│   ├── routes/
│   │   ├── home.tsx             # Landing / dashboard
│   │   ├── apps.tsx             # App list
│   │   ├── apps_.$id.tsx       # App detail
│   │   ├── apps_.$id.deploy.tsx # Deploy page
│   │   └── ai.tsx              # AI generation
│   ├── components/
│   │   ├── app-card.tsx         # App card component
│   │   ├── build-log.tsx       # Real-time build log viewer
│   │   ├── code-editor.tsx     # Monaco editor wrapper
│   │   ├── deploy-button.tsx   # Deploy/redeploy button
│   │   ├── status-badge.tsx   # Running/stopped/error badge
│   │   ├── ai-chat.tsx        # AI prompt + streaming response
│   │   └── file-tree.tsx       # Source file navigation
│   ├── lib/
│   │   ├── api.ts              # API client
│   │   ├── ws.ts               # WebSocket client
│   │   └── types.ts            # TypeScript types
│   ├── entry.client.tsx
│   ├── entry.server.tsx
│   └── app.css
├── public/
│   └── favicon.ico
├── package.json
├── vite.config.ts
├── tsconfig.json
└── wrangler.jsonc
```

## Dashboard Pages

### `/` — Home / Dashboard
- Overview card: total apps, running apps, recent builds
- Quick actions: Create App, AI Generate
- Recent activity feed

### `/apps` — App List
- Grid/list view of all apps
- Status badge (running/stopped/error/building)
- Framework badge (React Router / Nuxt.js / Worker)
- Quick actions: Start/Stop, Delete
- Search/filter by name, framework, status

### `/apps/:id` — App Detail
- **Header**: App name, status, URL, framework
- **Tabs**: Overview | Code | Builds | Settings
- **Overview tab**:
  - Status card (running/stopped, uptime, URL)
  - Quick actions (Start/Stop/Redeploy/Delete)
  - Recent builds
- **Code tab**:
  - File tree (left sidebar)
  - Monaco Editor (main area)
  - Save button → PUT /api/v1/apps/:id/source/:path
  - Deploy button → POST /api/v1/apps/:id/deploy
- **Builds tab**:
  - Build history list
  - Click build → view full log
  - Real-time log streaming (WebSocket)
- **Settings tab**:
  - Environment variables editor
  - Delete app (with confirmation)

### `/apps/:id/deploy` — Deploy
- Upload source (drag & drop zip, or file picker)
- Or: Use existing source → Deploy button
- Build progress with real-time log
- Success: link to deployed app URL

### `/ai` — AI Code Generation
- Prompt input (textarea)
- Framework selector (React Router / Nuxt.js / Worker)
- Auto-deploy toggle
- Streaming response (code appears in real-time)
- Preview: show generated file tree
- "Deploy" button → creates app + auto-deploys

> **Note:** The `/settings` page was removed from the plan — platform settings are configured via environment variables rather than a UI page.

## API Client

```typescript
// dashboard/app/lib/api.ts

const API_BASE = '/api/v1';  // Proxied through same domain

export const api = {
  // Apps
  listApps: (params?: { status?: string; framework?: string }) =>
    fetchJSON(`${API_BASE}/apps?${new URLSearchParams(params || {})}`),

  getApp: (id: string) =>
    fetchJSON(`${API_BASE}/apps/${id}`),

  createApp: (data: { name: string; framework: string }) =>
    fetchJSON(`${API_BASE}/apps`, { method: 'POST', body: JSON.stringify(data) }),

  deleteApp: (id: string) =>
    fetch(`${API_BASE}/apps/${id}`, { method: 'DELETE' }),

  // Source
  getSource: (id: string, path?: string) =>
    fetchJSON(`${API_BASE}/apps/${id}/source${path ? `?path=${path}` : ''}`),

  updateSource: (id: string, path: string, content: string) =>
    fetch(`${API_BASE}/apps/${id}/source/${encodeURIComponent(path)}`, {
      method: 'PUT',
      body: JSON.stringify({ content }),
    }),

  // Build & Deploy
  deploy: (id: string, source?: Record<string, string>) =>
    fetchJSON(`${API_BASE}/apps/${id}/deploy`, {
      method: 'POST',
      body: JSON.stringify(source ? { source: { files: source } } : {}),
    }),

  getBuilds: (id: string) =>
    fetchJSON(`${API_BASE}/apps/${id}/builds`),

  getBuild: (id: string, buildId: string) =>
    fetchJSON(`${API_BASE}/apps/${id}/builds/${buildId}`),

  // Runtime
  startApp: (id: string) =>
    fetchJSON(`${API_BASE}/apps/${id}/start`, { method: 'POST' }),

  stopApp: (id: string) =>
    fetchJSON(`${API_BASE}/apps/${id}/stop`, { method: 'POST' }),

  getAppStatus: (id: string) =>
    fetchJSON(`${API_BASE}/apps/${id}/status`),

  // AI
  generate: (data: { prompt: string; framework: string; auto_deploy?: boolean }) =>
    fetchJSON(`${API_BASE}/ai/generate`, { method: 'POST', body: JSON.stringify(data) }),

  getGeneration: (id: string) =>
    fetchJSON(`${API_BASE}/ai/generations/${id}`),
};
```

## Dashboard Deployment as a Fugue App

The Dashboard is deployed into the shared workerd process like any other app:

```capnp
# In the dispatch config.capnp:
(name = "app-dashboard", worker = .dashboardEntryWorker),
(name = "ssr-dashboard", worker = .dashboardSsrWorker),
(name = "static-dashboard", worker = .dashboardStaticWorker),

# In dispatch worker bindings:
(name = "APP_DASHBOARD", service = "app-dashboard"),

# Dashboard entry worker:
const dashboardEntryWorker :Workerd.Worker = (
  modules = [
    (name = "dashboard/entry.mjs", esModule = embed "dashboard/entry.mjs"),
    (name = "dashboard/static-assets.mjs", esModule = embed "dashboard/static-assets.mjs"),
  ],
  compatibilityDate = "2026-04-21",
  compatibilityFlags = ["nodejs_compat"],
  bindings = [
    (name = "SSR", service = "ssr-dashboard"),
    (name = "STATIC", service = "static-dashboard"),
  ],
);
```

The dispatch worker routes `dashboard.fugue.local` → `APP_DASHBOARD`.

Special handling: The Dashboard app entry worker adds an `/api` proxy — requests to `/api/v1/*` are forwarded to the Fugue platform API server on port 3000. This avoids CORS issues since the Dashboard and API share the same domain.

```javascript
// In dashboard entry worker (dashboard/entry.mjs):
import assets from "dashboard/static-assets.mjs";

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const pathname = url.pathname;

    // Proxy API requests to Fugue platform
    if (pathname.startsWith("/api/")) {
      return env.PLATFORM.fetch(request);
    }

    // Serve static assets
    const asset = assets.get(pathname);
    if (asset) {
      return new Response(
        Uint8Array.from(atob(asset.data), c => c.charCodeAt(0)),
        { headers: { "Content-Type": asset.mime, "Cache-Control": "public, max-age=31536000, immutable" } }
      );
    }

    // Delegate to SSR
    return env.SSR.fetch(request);
  },
};
```

The Dashboard's SSR worker has a `PLATFORM` binding pointing to a Fugue platform external service:

```capnp
# In config.capnp external services:
external = [
  (name = "PLATFORM", address = "127.0.0.1:3000", http = ())
],
```

Hmm, actually that's the wrong approach. The Dashboard's SSR worker would need to call the Fugue API from the server side. But since both the Dashboard and the API are within the same binary, the Dashboard SSR can just construct `fetch(new Request("http://127.0.0.1:3000/api/v1/..."))` — which works because workerd can make outbound HTTP requests.

But actually, the simplest approach is to have the Dashboard make API calls from the **client side** (browser) using relative URLs. Since the Dashboard is served from `dashboard.fugue.local:3000` and the API is also at `dashboard.fugue.local:3000/api/v1/*`, there are no CORS issues. The proxy on port 3000 (in the Fugue Axum server) handles both:
- `dashboard.fugue.local` → proxy to workerd (Dashboard SSR)
- `dashboard.fugue.local/api/v1/*` → handle directly (API)

This is simpler than adding service bindings for the API.

## Build & Deploy the Dashboard

A new CLI command builds and deploys the Dashboard:

```bash
fugue dashboard deploy    # Build dashboard React Router app and deploy as internal app
```

Or it's done automatically on `fugue start` if the Dashboard isn't deployed yet.

The Dashboard source lives in `dashboard/` within the Fugue repository. During platform startup:

1. Check if `dashboard` app exists in database
2. If not, build `dashboard/` with `react-router build`
3. Generate workerd artifacts (same as any React Router app)
4. Add `app-dashboard`, `ssr-dashboard`, `static-dashboard` services to dispatch config
5. Reload workerd

## New Files

```
dashboard/                        # Entire React Router v7 project
├── app/                          # Router routes, components, lib
├── public/
├── package.json
├── vite.config.ts
├── tsconfig.json
└── wrangler.jsonc

src/
├── dashboard/
│   └── mod.rs                    # Dashboard build + deploy logic
│
# Configuration additions:
fugue.toml                        # Platform config (replaces ~/.fugue/config.toml)
```

## Modified Files

```
src/process/config_gen.rs         # Add dashboard services to dispatch config
src/api/mod.rs                    # Add /api/v1/* route (already exists, no change needed)
src/daemon/server.rs              # Ensure API routes take priority over proxy
src/main.rs                       # Auto-deploy dashboard on startup
```

## Package Dependencies (Dashboard)

```json
{
  "dependencies": {
    "@heroui/react": "^3.0.3",
    "@heroui/styles": "^3.0.3",
    "@iconify/react": "^6.0.2",
    "@monaco-editor/react": "^4.7.0",
    "@tanstack/react-query": "^5.83.0",
    "isbot": "^5.1.31",
    "react": "^19.1.1",
    "react-dom": "^19.1.1",
    "react-router": "^7.10.0"
  },
  "devDependencies": {
    "@cloudflare/vite-plugin": "^1.13.5",
    "@react-router/dev": "^7.10.0",
    "@tailwindcss/vite": "^4.1.13",
    "@types/node": "^25.6.0",
    "@types/react": "^19.1.13",
    "@types/react-dom": "^19.1.9",
    "tailwindcss": "^4.1.13",
    "typescript": "^5.9.2",
    "vite": "^7.1.7",
    "vite-tsconfig-paths": "^5.1.4",
    "wrangler": "^4.87.0"
  }
}
```

## Testing

```bash
# 1. Start platform (auto-deploys dashboard)
fugue start

# 2. Access dashboard
open http://dashboard.fugue.local:3000/

# 3. Create app via dashboard
# — Click "Create App" → fill form → submit

# 4. AI generate via dashboard
# — Click "AI Generate" → type prompt → watch streaming generation

# 5. View build logs in real-time
# — Click "Deploy" → watch WebSocket log stream

# 6. Edit code in Monaco Editor
# — Navigate to app → Code tab → edit file → save → deploy
```

## Dependencies

- Stage 6 (Dynamic Dispatch + PostgreSQL) — Dashboard is a routed app
- Stage 8 (AI Code Gen) — AI generation UI
- Stage 9 (Async Build) — Build log WebSocket

## Implementation Notes

### Framework Naming: `worker` (formerly `single-file`)

The framework identifier `single-file` has been renamed to `worker` across the entire codebase:
- Rust backend: `src/db/models.rs` — `Framework::Worker` variant with `#[serde(rename = "worker")]`
- Rust backend: `src/api/apps.rs` — validation error message
- Rust backend: `src/api/deploy.rs` — match arm `"worker"` and `find_worker_entry()`
- Rust backend: `src/process/config_gen.rs` — match arm and `generate_worker_def()`
- Rust backend: `src/cli.rs` — default framework value and help text
- Dashboard: all framework selectors and labels use `worker`/`Worker`
- Migration: `migrations/001_initial.sql` — CHECK constraint updated

### HeroUI v3 Component Patterns

HeroUI v3 uses a compound component pattern (dot notation). Key differences from v2:
- **Card**: `Card.Header`, `Card.Title`, `Card.Description`, `Card.Content`, `Card.Footer`
- **Modal**: `Modal.Backdrop` (with `isOpen`/`onOpenChange`), `Modal.Container`, `Modal.Dialog`, `Modal.CloseTrigger`, `Modal.Heading`, `Modal.Body`
- **Tabs**: `Tabs.ListContainer`, `Tabs.List`, `Tabs.Tab`, `Tabs.Indicator`, `Tabs.Panel`
- **Select**: `Select`, `Select.Trigger`, `Select.Value`, `Select.Indicator`, `Select.Popover`, `ListBox`, `ListBox.Item`
- **No HeroUIProvider**: Components work with just CSS imports (`@import "@heroui/styles"`)
- **Selectable options**: Use `ToggleButtonGroup`/`ToggleButton` with `selectionMode="single"` and `selectedKeys`/`onSelectionChange`, not `Chip` with `onPress`