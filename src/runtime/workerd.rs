use crate::error::{FugueError, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::{Child, Command};
use std::process::Stdio;

pub struct WorkerdProcess {
    pub pid: Option<u32>,
    pub port: u16,
    pub function_name: String,
    pub process: Child,
}

pub struct WorkerdPool {
    processes: HashMap<String, WorkerdProcess>,
    available_ports: Vec<u16>,
    workerd_dir: PathBuf,
}

impl WorkerdPool {
    pub fn new(workerd_dir: PathBuf) -> Self {
        let mut available_ports = Vec::new();
        for port in crate::config::WORKERD_PORT_START..=crate::config::WORKERD_PORT_END {
            available_ports.push(port);
        }

        Self {
            processes: HashMap::new(),
            available_ports,
            workerd_dir,
        }
    }

    pub async fn get_or_spawn(
        &mut self,
        function_name: &str,
        code: &str,
    ) -> Result<u16> {
        // Check if process already exists
        if let Some(process) = self.processes.get(function_name) {
            return Ok(process.port);
        }

        // Get available port
        let port = self
            .available_ports
            .pop()
            .ok_or_else(|| FugueError::WorkerdError("No available ports".to_string()))?;

        // Spawn workerd process
        let process = self.spawn_workerd(function_name, code, port).await?;

        self.processes.insert(
            function_name.to_string(),
            WorkerdProcess {
                pid: process.id(),
                port,
                function_name: function_name.to_string(),
                process,
            },
        );

        Ok(port)
    }

    async fn spawn_workerd(
        &self,
        function_name: &str,
        code: &str,
        port: u16,
    ) -> Result<Child> {
        // Create function directory
        let func_dir = self.workerd_dir.join(function_name);
        std::fs::create_dir_all(&func_dir)?;

        // Write function code
        let code_path = func_dir.join("worker.js");
        std::fs::write(&code_path, code)?;

        // Generate workerd config
        let config_path = func_dir.join("config.capnp");
        let config = self.generate_config(function_name, port);
        std::fs::write(&config_path, config)?;

        // Spawn workerd
        let child = Command::new("workerd")
            .arg("serve")
            .arg(&config_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                FugueError::WorkerdError(format!("Failed to spawn workerd: {}", e))
            })?;

        // Wait a bit for workerd to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(child)
    }

    fn generate_config(&self, _function_name: &str, port: u16) -> String {
        format!(
            r#"using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [
    (name = "main", worker = .mainWorker),
  ],
  sockets = [
    ( name = "http",
      address = "*:{}",
      http = (),
      service = "main"
    ),
  ],
);

const mainWorker :Workerd.Worker = (
  modules = [
    (name = "worker.js", esModule = embed "worker.js"),
  ],
  compatibilityDate = "2024-01-01",
);
"#,
            port
        )
    }

    pub async fn stop_process(&mut self, function_name: &str) -> Result<()> {
        if let Some(mut process) = self.processes.remove(function_name) {
            process.process.kill().await.map_err(|e| {
                FugueError::WorkerdError(format!("Failed to kill workerd: {}", e))
            })?;
            self.available_ports.push(process.port);
        }
        Ok(())
    }

    pub fn get_port(&self, function_name: &str) -> Option<u16> {
        self.processes.get(function_name).map(|p| p.port)
    }

    pub async fn get_or_spawn_nextjs(
        &mut self,
        function_name: &str,
        standalone_path: &PathBuf,
        env_vars: &HashMap<String, String>,
    ) -> Result<u16> {
        // Check if process already exists
        if let Some(process) = self.processes.get(function_name) {
            return Ok(process.port);
        }

        // Get available port
        let port = self
            .available_ports
            .pop()
            .ok_or_else(|| FugueError::WorkerdError("No available ports".to_string()))?;

        // Spawn workerd process for Next.js
        let process = self
            .spawn_workerd_nextjs(function_name, standalone_path, env_vars, port)
            .await?;

        self.processes.insert(
            function_name.to_string(),
            WorkerdProcess {
                pid: process.id(),
                port,
                function_name: function_name.to_string(),
                process,
            },
        );

        Ok(port)
    }

    async fn spawn_workerd_nextjs(
        &self,
        _function_name: &str,
        standalone_path: &PathBuf,
        env_vars: &HashMap<String, String>,
        port: u16,
    ) -> Result<Child> {
        // For Next.js, we run it directly with Node.js instead of workerd
        // because Next.js standalone uses CommonJS which workerd doesn't support well

        tracing::info!("Spawning Next.js with Node.js at: {:?}", standalone_path);

        let server_js = standalone_path.join("server.js");
        if !server_js.exists() {
            return Err(FugueError::WorkerdError(format!(
                "server.js not found in standalone directory: {:?}",
                standalone_path
            )));
        }

        tracing::info!("Found server.js at: {:?}", server_js);

        // Build environment variables
        // Use full path to node to ensure it's found
        let node_path = std::env::var("PATH")
            .ok()
            .and_then(|path| {
                path.split(':')
                    .find_map(|p| {
                        let node_bin = std::path::Path::new(p).join("node");
                        if node_bin.exists() {
                            Some(node_bin)
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or_else(|| PathBuf::from("node"));

        tracing::info!("Using node at: {:?}", node_path);

        let mut cmd = Command::new(&node_path);
        cmd.arg(&server_js)
            .current_dir(standalone_path)
            .env("PORT", port.to_string())
            .env("NODE_ENV", "production")
            // Clear proxy environment variables to avoid interference
            .env_remove("http_proxy")
            .env_remove("https_proxy")
            .env_remove("HTTP_PROXY")
            .env_remove("HTTPS_PROXY")
            .env_remove("all_proxy")
            .env_remove("ALL_PROXY")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add custom environment variables
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        tracing::info!("Spawning Node.js on port {}", port);
        tracing::debug!("Command: {:?}", cmd);

        // Spawn Node.js process
        let child = cmd.spawn().map_err(|e| {
            tracing::error!("Failed to spawn Node.js: {:?}", e);
            FugueError::WorkerdError(format!("Failed to spawn Node.js: {}", e))
        })?;

        tracing::info!("Node.js spawned with PID: {:?}", child.id());

        // Wait a bit for Next.js to start
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

        Ok(child)
    }

    fn generate_nextjs_config(
        &self,
        _function_name: &str,
        port: u16,
        _env_vars: &HashMap<String, String>,
    ) -> String {
        format!(
            r#"using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [
    (name = "main", worker = .nextWorker),
  ],
  sockets = [
    ( name = "http",
      address = "*:{}",
      http = (),
      service = "main"
    ),
  ],
);

const nextWorker :Workerd.Worker = (
  modules = [
    (name = "server.js", esModule = embed "standalone/server.js"),
  ],
  compatibilityDate = "2024-01-01",
  compatibilityFlags = ["nodejs_compat"],
);
"#,
            port
        )
    }

    pub async fn get_or_spawn_nuxtjs(
        &mut self,
        function_name: &str,
        output_path: &PathBuf,
        env_vars: &HashMap<String, String>,
    ) -> Result<u16> {
        // Check if process already exists
        if let Some(process) = self.processes.get(function_name) {
            return Ok(process.port);
        }

        // Get available port
        let port = self
            .available_ports
            .pop()
            .ok_or_else(|| FugueError::WorkerdError("No available ports".to_string()))?;

        // Spawn workerd process for Nuxt.js
        let process = self
            .spawn_workerd_nuxtjs(function_name, output_path, env_vars, port)
            .await?;

        self.processes.insert(
            function_name.to_string(),
            WorkerdProcess {
                pid: process.id(),
                port,
                function_name: function_name.to_string(),
                process,
            },
        );

        Ok(port)
    }

    async fn spawn_workerd_nuxtjs(
        &self,
        _function_name: &str,
        output_path: &PathBuf,
        env_vars: &HashMap<String, String>,
        port: u16,
    ) -> Result<Child> {
        // For Nuxt.js, we run it directly with Node.js instead of workerd
        // Nuxt 3 uses Nitro server which outputs ESM (index.mjs)

        tracing::info!("Spawning Nuxt.js with Node.js at: {:?}", output_path);

        let index_mjs = output_path.join("index.mjs");
        if !index_mjs.exists() {
            return Err(FugueError::WorkerdError(format!(
                "index.mjs not found in output directory: {:?}",
                output_path
            )));
        }

        tracing::info!("Found index.mjs at: {:?}", index_mjs);

        // Build environment variables
        let node_path = std::env::var("PATH")
            .ok()
            .and_then(|path| {
                path.split(':')
                    .find_map(|p| {
                        let node_bin = std::path::Path::new(p).join("node");
                        if node_bin.exists() {
                            Some(node_bin)
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or_else(|| PathBuf::from("node"));

        tracing::info!("Using node at: {:?}", node_path);

        let mut cmd = Command::new(&node_path);
        cmd.arg(&index_mjs)
            .current_dir(output_path)
            .env("PORT", port.to_string())
            .env("NITRO_PORT", port.to_string())
            .env("NODE_ENV", "production")
            // Clear proxy environment variables to avoid interference
            .env_remove("http_proxy")
            .env_remove("https_proxy")
            .env_remove("HTTP_PROXY")
            .env_remove("HTTPS_PROXY")
            .env_remove("all_proxy")
            .env_remove("ALL_PROXY")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add custom environment variables
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        tracing::info!("Spawning Node.js on port {}", port);
        tracing::debug!("Command: {:?}", cmd);

        // Spawn Node.js process
        let child = cmd.spawn().map_err(|e| {
            tracing::error!("Failed to spawn Node.js: {:?}", e);
            FugueError::WorkerdError(format!("Failed to spawn Node.js: {}", e))
        })?;

        tracing::info!("Node.js spawned with PID: {:?}", child.id());

        // Wait a bit for Nuxt.js to start
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

        Ok(child)
    }
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in walkdir::WalkDir::new(src) {
        let entry = entry.map_err(|e| FugueError::Other(format!("Failed to walk directory: {}", e)))?;
        let path = entry.path();
        let relative_path = path.strip_prefix(src).unwrap();
        let target_path = dst.join(relative_path);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target_path)?;
        } else {
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(path, &target_path)?;
        }
    }

    Ok(())
}
