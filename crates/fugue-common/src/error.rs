use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum FugueError {
    #[error("App '{0}' not found")]
    AppNotFound(String),

    #[error("App '{0}' already exists")]
    AppAlreadyExists(String),

    #[error("Build '{0}' not found")]
    BuildNotFound(String),

    #[error("App is not running: {0}")]
    AppNotRunning(String),

    #[error("App is already running: {0}")]
    AppAlreadyRunning(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("workerd error: {0}")]
    WorkerdError(String),

    #[error("Build error: {0}")]
    BuildError(String),

    #[error("Not a Nuxt.js project: {0}")]
    NotNuxtJsProject(String),

    #[error("Not a React Router project: {0}")]
    NotReactRouterProject(String),

    #[error("Not a Vite project: {0}")]
    NotViteProject(String),

    #[error("Nuxt.js version not supported: {0}")]
    UnsupportedNuxtJsVersion(String),

    #[error("Node.js not found or incompatible version")]
    NodeJsError,

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Proxy error: {0}")]
    ProxyError(String),

    #[error("Process error: {0}")]
    ProcessError(String),

    #[error("NATS error: {0}")]
    NatsError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, FugueError>;
