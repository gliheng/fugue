pub mod defaults;

use crate::error::{FugueError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub database: DatabaseConfig,
    pub platform: ServerConfig,
    pub workerd: WorkerdConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerdConfig {
    pub port: u16,
    pub binary: String,
    pub health_check_interval_secs: u64,
    pub watch_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: defaults::DEFAULT_DATABASE_URL.to_string(),
            },
            platform: ServerConfig {
                host: defaults::DEFAULT_HOST.to_string(),
                port: defaults::DEFAULT_PLATFORM_PORT,
                domain: defaults::DEFAULT_DOMAIN.to_string(),
            },
            workerd: WorkerdConfig {
                port: defaults::DEFAULT_WORKERD_PORT,
                binary: defaults::DEFAULT_WORKERD_BINARY.to_string(),
                health_check_interval_secs: defaults::DEFAULT_HEALTH_CHECK_INTERVAL,
                watch_mode: defaults::DEFAULT_WATCH_MODE,
            },
            logging: LoggingConfig {
                level: defaults::DEFAULT_LOG_LEVEL.to_string(),
                file: None,
            },
        }
    }
}

#[allow(dead_code)]
impl PlatformConfig {
    pub fn load() -> Result<Self> {
        let config_path = fugue_dir().join("config.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).map_err(|e| {
                FugueError::ConfigError(format!("Failed to read config file: {}", e))
            })?;

            toml::from_str(&content).map_err(|e| {
                FugueError::ConfigError(format!("Failed to parse config file: {}", e))
            })
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = fugue_dir().join("config.toml");
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| {
            FugueError::ConfigError(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn with_db_url(mut self, url: &str) -> Self {
        self.database.url = url.to_string();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.platform.port = port;
        self
    }
}

pub fn fugue_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".fugue")
}

pub fn workerd_dir() -> PathBuf {
    fugue_dir().join("workerd")
}

pub fn data_dir() -> PathBuf {
    fugue_dir().join("data")
}

pub fn apps_data_dir() -> PathBuf {
    data_dir().join("apps")
}
