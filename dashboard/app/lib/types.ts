export interface App {
  id: string;
  name: string;
  slug: string;
  subdomain: string;
  framework: string;
  status: string;
  description: string | null;
  env_vars: Record<string, string>;
  source_path: string | null;
  build_path: string | null;
  created_at: string;
  updated_at: string;
}

export interface Build {
  id: string;
  app_id: string;
  status: string;
  log: string | null;
  error: string | null;
  created_at: string;
  finished_at: string | null;
}

export interface Deployment {
  id: string;
  app_id: string;
  build_id: string;
  version: number;
  status: string;
  started_at: string | null;
  stopped_at: string | null;
  created_at: string;
}

export interface SourceFile {
  files: Record<string, string>;
}

export interface CreateAppRequest {
  name: string;
  framework: string;
  description?: string;
}

export interface UpdateAppRequest {
  name?: string;
  description?: string;
  env_vars?: Record<string, string>;
}

export interface DeployRequest {
  source?: {
    files: Record<string, string>;
  };
}

export interface AppStatus {
  status: string;
  url: string | null;
  updated_at: string;
}

export interface PlatformStatus {
  status: string;
  version: string;
  uptime: number;
  apps: {
    total: number;
    running: number;
  };
  domain: string;
  port: number;
  workerd_port: number;
}

export interface TemplateInfo {
  id: string;
  name: string;
  framework: string;
  description: string;
}

export interface TemplateDetail {
  id: string;
  name: string;
  framework: string;
  description: string;
  files: Record<string, string>;
}

export interface Workspace {
  id: string;
  name: string;
  framework: string;
  files: Record<string, string>;
  created_at: string;
  updated_at: string;
}

export interface CreateWorkspaceRequest {
  name?: string;
  framework: string;
}

export interface UpdateWorkspaceRequest {
  name?: string;
  files?: Record<string, string>;
}

