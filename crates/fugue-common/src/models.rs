use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTask {
    pub build_id: Uuid,
    pub app_id: Uuid,
    pub app_slug: String,
    pub source_path: PathBuf,
    pub framework: Framework,
    pub skip_install: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Framework {
    #[serde(rename = "worker")]
    Worker,
    #[serde(rename = "nuxtjs")]
    NuxtJs,
    #[serde(rename = "react-router")]
    ReactRouter,
}

impl Framework {
    pub fn as_str(&self) -> &str {
        match self {
            Framework::Worker => "worker",
            Framework::NuxtJs => "nuxtjs",
            Framework::ReactRouter => "react-router",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "worker" => Some(Framework::Worker),
            "nuxtjs" => Some(Framework::NuxtJs),
            "react-router" => Some(Framework::ReactRouter),
            _ => None,
        }
    }
}

impl std::fmt::Display for Framework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub build_id: Uuid,
    pub app_id: Uuid,
    pub success: bool,
    pub output_size: u64,
    pub build_time_ms: u128,
    pub error: Option<String>,
    pub artifacts_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildLog {
    pub build_id: Uuid,
    pub line: String,
    pub stream: LogStream,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogStream {
    Stdout,
    Stderr,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct App {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub subdomain: String,
    pub framework: String,
    pub status: String,
    pub description: Option<String>,
    pub env_vars: serde_json::Value,
    pub source_path: Option<String>,
    pub build_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AppStatus {
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "building")]
    Building,
    #[serde(rename = "deploying")]
    Deploying,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "error")]
    Error,
}

impl AppStatus {
    pub fn as_str(&self) -> &str {
        match self {
            AppStatus::Created => "created",
            AppStatus::Building => "building",
            AppStatus::Deploying => "deploying",
            AppStatus::Running => "running",
            AppStatus::Stopped => "stopped",
            AppStatus::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Build {
    pub id: Uuid,
    pub app_id: Uuid,
    pub status: String,
    pub log: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Deployment {
    pub id: Uuid,
    pub app_id: Uuid,
    pub build_id: Uuid,
    pub version: i32,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAppRequest {
    pub name: String,
    pub framework: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAppRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub env_vars: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployRequest {
    pub source: Option<SourcePayload>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourcePayload {
    pub files: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub framework: String,
    pub file_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: Option<String>,
    pub framework: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub files: Option<serde_json::Value>,
}
