use std::path::PathBuf;

pub const DAEMON_PORT: u16 = 7878;
pub const DAEMON_HOST: &str = "127.0.0.1";

pub const WORKERD_PORT_START: u16 = 8080;
pub const WORKERD_PORT_END: u16 = 8180;

pub const DEFAULT_TIMEOUT_MS: u64 = 5000;
pub const MAX_FUNCTION_SIZE: usize = 1024 * 1024; // 1MB

// Framework build constants (used by Nuxt.js)
pub const MAX_BUILD_TIME_MS: u64 = 300_000; // 5 minutes
pub const MAX_PROJECT_SIZE: usize = 100 * 1024 * 1024; // 100MB
pub const SUPPORTED_NODE_VERSIONS: &[&str] = &["18", "20"];

pub fn fugue_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".fugue")
}

pub fn daemon_pid_file() -> PathBuf {
    fugue_dir().join("daemon.pid")
}

pub fn daemon_log_file() -> PathBuf {
    fugue_dir().join("daemon.log")
}

pub fn workerd_dir() -> PathBuf {
    fugue_dir().join("workerd")
}

pub fn functions_dir() -> PathBuf {
    std::env::current_dir()
        .expect("Could not get current directory")
        .join("functions")
}
