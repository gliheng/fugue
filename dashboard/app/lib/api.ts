import type {
  App,
  AppStatus,
  Build,
  CreateAppRequest,
  CreateWorkspaceRequest,
  DeployRequest,
  PlatformStatus,
  SourceFile,
  TemplateDetail,
  TemplateInfo,
  UpdateWorkspaceRequest,
  UpdateAppRequest,
  Workspace,
  WorkspaceDetail,
} from "./types";

const API_BASE = "/api/v1";

async function fetchJSON<T>(
  url: string,
  options?: RequestInit,
): Promise<T> {
  const res = await fetch(`${API_BASE}${url}`, {
    "Content-Type": "application/json",
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
  });

  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(body.error || `API error: ${res.status}`);
  }

  return res.json();
}

export const api = {
  listApps: (params?: { status?: string; framework?: string }) => {
    const query = params
      ? `?${new URLSearchParams(
          Object.entries(params).filter(([, v]) => v != null) as [string, string][],
        ).toString()}`
      : "";
    return fetchJSON<App[]>(`/apps${query}`);
  },

  getApp: (id: string) => fetchJSON<App>(`/apps/${id}`),

  createApp: (data: CreateAppRequest) =>
    fetchJSON<App>("/apps", { method: "POST", body: JSON.stringify(data) }),

  updateApp: (id: string, data: UpdateAppRequest) =>
    fetchJSON<App>(`/apps/${id}`, { method: "PATCH", body: JSON.stringify(data) }),

  deleteApp: (id: string) =>
    fetch(`${API_BASE}/apps/${id}`, { method: "DELETE" }).then((r) => {
      if (!r.ok) throw new Error(`Failed to delete app: ${r.status}`);
    }),

  getSource: (id: string) => fetchJSON<SourceFile>(`/apps/${id}/source`),

  updateSource: (id: string, path: string, content: string) =>
    fetch(`${API_BASE}/apps/${id}/source/${encodeURIComponent(path)}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ content }),
    }).then((r) => {
      if (!r.ok) throw new Error(`Failed to update source: ${r.status}`);
    }),

  deploy: (id: string, source?: Record<string, string>) =>
    fetchJSON<{ build_id: string; status: string }>(
      `/apps/${id}/deploy`,
      {
        method: "POST",
        body: JSON.stringify(source ? { source: { files: source } } : {}),
      },
    ),

  redeploy: (id: string) =>
    fetchJSON<{ build_id: string; status: string }>(`/apps/${id}/redeploy`, {
      method: "POST",
    }),

  getBuilds: (id: string) => fetchJSON<Build[]>(`/apps/${id}/builds`),

  getBuild: (id: string, buildId: string) =>
    fetchJSON<Build>(`/apps/${id}/builds/${buildId}`),

  startApp: (id: string) =>
    fetchJSON<AppStatus>(`/apps/${id}/start`, { method: "POST" }),

  stopApp: (id: string) =>
    fetchJSON<{ status: string }>(`/apps/${id}/stop`, { method: "POST" }),

  getAppStatus: (id: string) => fetchJSON<AppStatus>(`/apps/${id}/status`),

  platformStatus: () => fetchJSON<PlatformStatus>("/platform/status"),

  listTemplates: () => fetchJSON<TemplateInfo[]>("/templates"),

  getTemplate: (framework: string) => fetchJSON<TemplateDetail>(`/templates/${framework}`),

  uploadSourceFiles: (id: string, files: Record<string, string>) => {
    return fetch(`${API_BASE}/apps/${id}/source/files`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ files }),
    }).then(async (r) => {
      if (!r.ok) {
        const body = await r.json().catch(() => ({ error: r.statusText }));
        throw new Error(body.error || `Upload failed: ${r.status}`);
      }
      return r.json() as Promise<{ source_path: string; file_count: number; total_size: number }>;
    });
  },

  listWorkspaces: () => fetchJSON<Workspace[]>("/workspaces"),

  getWorkspace: (id: string) => fetchJSON<WorkspaceDetail>(`/workspaces/${id}`),

  createWorkspace: (data: CreateWorkspaceRequest) =>
    fetchJSON<Workspace>("/workspaces", { method: "POST", body: JSON.stringify(data) }),

  updateWorkspace: (id: string, data: UpdateWorkspaceRequest) =>
    fetchJSON<Workspace>(`/workspaces/${id}`, { method: "PATCH", body: JSON.stringify(data) }),

  deleteWorkspace: (id: string) =>
    fetch(`${API_BASE}/workspaces/${id}`, { method: "DELETE" }).then((r) => {
      if (!r.ok) throw new Error(`Failed to delete workspace: ${r.status}`);
    }),
};

let cachedConfig: PlatformStatus | null = null;

export function getAppUrl(subdomain: string): string {
  const domain = cachedConfig?.domain ?? "fugue.localhost";
  const port = cachedConfig?.workerd_port ?? 8080;
  return `http://${subdomain}.${domain}:${port}`;
}

export async function loadConfig(): Promise<PlatformStatus> {
  if (!cachedConfig) {
    cachedConfig = await api.platformStatus();
  }
  return cachedConfig;
}
