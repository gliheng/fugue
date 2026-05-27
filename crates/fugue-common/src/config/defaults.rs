#![allow(dead_code)]

pub const DEFAULT_DATABASE_URL: &str = "postgresql://fugue:fugue@localhost:5433/fugue";
pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PLATFORM_PORT: u16 = 3000;
pub const DEFAULT_DOMAIN: &str = "fugue.localhost";
pub const DEFAULT_WORKERD_PORT: u16 = 8080;
pub const DEFAULT_WORKERD_BINARY: &str = "workerd";
pub const DEFAULT_HEALTH_CHECK_INTERVAL: u64 = 30;
pub const DEFAULT_WATCH_MODE: bool = true;
pub const DEFAULT_LOG_LEVEL: &str = "info";
pub const DEFAULT_NATS_PORT: u16 = 4222;
pub const DEFAULT_NATS_URL: &str = "nats://localhost:4222";

pub const MAX_FUNCTION_SIZE: usize = 1024 * 1024; // 1MB
pub const MAX_PROJECT_SIZE: usize = 100 * 1024 * 1024; // 100MB
pub const MAX_BUILD_TIME_MS: u64 = 300_000; // 5 minutes
pub const SUPPORTED_NODE_VERSIONS: &[&str] = &["18", "20"];
