use crate::config::PlatformConfig;
use crate::db::models::App;
use crate::error::{FugueError, Result};
use crate::process::{config_gen, health};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

pub struct ProcessManager {
    process: Option<ManagedProcess>,
    config_path: PathBuf,
    workerd_dir: PathBuf,
    workerd_binary: String,
    workerd_port: u16,
    #[allow(dead_code)]
    health_interval: std::time::Duration,
    watch_mode: bool,
}

pub struct ManagedProcess {
    pub child: Child,
    #[allow(dead_code)]
    pub started_at: Instant,
    pub health_status: HealthStatus,
    pub restart_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Starting,
    Healthy,
    Unhealthy,
}

impl ProcessManager {
    pub fn new(config: &PlatformConfig) -> Result<Self> {
        let workerd_dir = crate::config::workerd_dir();
        std::fs::create_dir_all(&workerd_dir)?;

        Ok(Self {
            process: None,
            config_path: workerd_dir.join("config.capnp"),
            workerd_dir,
            workerd_binary: config.workerd.binary.clone(),
            workerd_port: config.workerd.port,
            health_interval: std::time::Duration::from_secs(
                config.workerd.health_check_interval_secs,
            ),
            watch_mode: config.workerd.watch_mode,
        })
    }

    pub async fn start(&mut self, apps: &[App]) -> Result<()> {
        if self.process.is_some() {
            tracing::info!("workerd process already running, reloading config");
            self.reload(apps).await?;
            return Ok(());
        }

        // Generate initial config
        config_gen::generate_dispatch_config(apps, &self.workerd_dir, self.workerd_port)?;

        // Start workerd process
        self.spawn_workerd().await?;

        // Wait for healthy
        self.wait_for_healthy(std::time::Duration::from_secs(10))
            .await?;

        tracing::info!(
            "workerd process started on port {}",
            self.workerd_port
        );

        Ok(())
    }

    pub async fn reload(&mut self, apps: &[App]) -> Result<()> {
        config_gen::generate_dispatch_config(apps, &self.workerd_dir, self.workerd_port)?;

        if self.watch_mode {
            // Touch config file to trigger workerd --watch reload
            let now = filetime::FileTime::now();
            filetime::set_file_mtime(&self.config_path, now).map_err(|e| {
                FugueError::ProcessError(format!("Failed to touch config file: {}", e))
            })?;

            tracing::info!("workerd config reloaded (watch mode)");
        } else {
            // Graceful restart
            self.stop().await?;
            self.spawn_workerd().await?;
            self.wait_for_healthy(std::time::Duration::from_secs(10))
                .await?;

            tracing::info!("workerd process restarted");
        }

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut managed) = self.process.take() {
            managed.child.kill().await.map_err(|e| {
                FugueError::ProcessError(format!("Failed to kill workerd: {}", e))
            })?;
            tracing::info!("workerd process stopped");
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }

    #[allow(dead_code)]
    pub fn workerd_port(&self) -> u16 {
        self.workerd_port
    }

    #[allow(dead_code)]
    pub fn health_status(&self) -> Option<&HealthStatus> {
        self.process.as_ref().map(|p| &p.health_status)
    }

    async fn spawn_workerd(&mut self) -> Result<()> {
        let mut args = vec!["serve".to_string()];
        if self.watch_mode {
            args.push("--watch".to_string());
        }
        args.push(self.config_path.to_string_lossy().to_string());

        let mut child = Command::new(&self.workerd_binary)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                FugueError::ProcessError(format!("Failed to spawn workerd: {}", e))
            })?;

        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::error!("[workerd] {}", line);
                }
            });
        }

        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::info!("[workerd] {}", line);
                }
            });
        }

        self.process = Some(ManagedProcess {
            child,
            started_at: Instant::now(),
            health_status: HealthStatus::Starting,
            restart_count: self
                .process
                .as_ref()
                .map(|p| p.restart_count + 1)
                .unwrap_or(0),
        });

        Ok(())
    }

    async fn wait_for_healthy(&mut self, timeout: std::time::Duration) -> Result<()> {
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                if let Some(ref mut managed) = self.process {
                    managed.health_status = HealthStatus::Unhealthy;
                }
                return Err(FugueError::ProcessError(
                    "workerd failed to become healthy within timeout".to_string(),
                ));
            }

            match health::check_health(self.workerd_port).await {
                HealthStatus::Healthy => {
                    if let Some(ref mut managed) = self.process {
                        managed.health_status = HealthStatus::Healthy;
                    }
                    return Ok(());
                }
                _ => {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
            }
        }
    }
}
