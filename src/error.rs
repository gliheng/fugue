use thiserror::Error;

#[derive(Error, Debug)]
pub enum FugueError {
    #[error("Daemon is not running")]
    DaemonNotRunning,

    #[error("Daemon is already running")]
    DaemonAlreadyRunning,

    #[error("Function '{0}' not found")]
    FunctionNotFound(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Function execution timed out")]
    TimeoutError,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("workerd error: {0}")]
    WorkerdError(String),

    #[error("Build error: {0}")]
    BuildError(String),

    #[error("Not a Next.js project: {0}")]
    NotNextJsProject(String),

    #[error("Next.js version not supported: {0}")]
    UnsupportedNextJsVersion(String),

    #[error("Not a Nuxt.js project: {0}")]
    NotNuxtJsProject(String),

    #[error("Nuxt.js version not supported: {0}")]
    UnsupportedNuxtJsVersion(String),

    #[error("Node.js not found or incompatible version")]
    NodeJsError,

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
