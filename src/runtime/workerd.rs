use crate::error::{FugueError, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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

    pub async fn get_or_spawn_nuxtjs(
        &mut self,
        function_name: &str,
        workerd_func_dir: &Path,
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
            .spawn_workerd_nuxtjs(function_name, workerd_func_dir, env_vars, port)
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
        workerd_func_dir: &Path,
        _env_vars: &HashMap<String, String>,
        port: u16,
    ) -> Result<Child> {
        let config_path = workerd_func_dir.join("config.capnp");
        if !config_path.exists() {
            return Err(FugueError::WorkerdError(format!(
                "workerd config not found at {:?}. Was generate_nuxtjs_workerd_artifacts() called during deploy?",
                config_path
            )));
        }

        tracing::info!("Spawning workerd for Nuxt.js at: {:?}", workerd_func_dir);

        let child = Command::new("workerd")
            .arg("serve")
            .arg(&config_path)
            .arg("-s")
            .arg(format!("http=*:{}", port))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                FugueError::WorkerdError(format!("Failed to spawn workerd: {}", e))
            })?;

        tracing::info!("workerd spawned with PID: {:?}", child.id());

        // Wait for workerd to start
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

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

/// Find the esbuild binary. Checks node_modules/.bin first, then PATH.
fn find_esbuild() -> Result<PathBuf> {
    // Check node_modules/.bin/esbuild in current directory
    let local = std::env::current_dir()
        .unwrap_or_default()
        .join("node_modules/.bin/esbuild");
    if local.exists() {
        return Ok(local);
    }

    // Check npx esbuild (available via npm)
    if let Ok(output) = std::process::Command::new("npx")
        .args(["esbuild", "--version"])
        .output()
    {
        if output.status.success() {
            return Ok(PathBuf::from("npx"));
        }
    }

    // Check PATH for esbuild
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(':') {
            let esbuild = Path::new(dir).join("esbuild");
            if esbuild.exists() {
                return Ok(esbuild);
            }
        }
    }

    Err(FugueError::BuildError(
        "esbuild not found. Install it with: npm install -g esbuild".to_string(),
    ))
}

/// Generate workerd artifacts for a Nuxt.js deployment (standalone, no WorkerdPool needed).
///
/// This creates:
/// - static-assets.mjs: all public files base64-encoded in a JS module
/// - entry.mjs: router that serves static assets or delegates to SSR
/// - bundle.mjs: the Nitro server bundled into a single ES module (via esbuild)
/// - config.capnp: workerd configuration with 3 services (entry, SSR, static)
pub fn generate_nuxtjs_workerd_artifacts(
    function_name: &str,
    build_output_dir: &Path,
    workerd_dir: &Path,
) -> Result<PathBuf> {
    let server_dir = build_output_dir.join("server");
    let public_dir = build_output_dir.join("public");

    if !server_dir.join("index.mjs").exists() {
        return Err(FugueError::BuildError(
            ".output/server/index.mjs not found".to_string(),
        ));
    }

    // Create workerd artifacts directory
    let workerd_func_dir = workerd_dir.join(function_name);
    std::fs::create_dir_all(&workerd_func_dir)?;

    // Step 1: Embed static assets
    tracing::info!("Embedding static assets for '{}'...", function_name);
    let assets_count = embed_static_assets(&public_dir, &workerd_func_dir)?;
    tracing::info!("Embedded {} static assets", assets_count);

    // Step 2: Bundle SSR server with esbuild
    tracing::info!("Bundling SSR server with esbuild for '{}'...", function_name);
    bundle_server_with_esbuild(&server_dir, &workerd_func_dir)?;

    // Step 3: Generate entry.mjs (router)
    tracing::info!("Generating entry worker for '{}'...", function_name);
    generate_entry_worker(&workerd_func_dir)?;

    // Step 4: Generate config.capnp
    tracing::info!("Generating workerd config for '{}'...", function_name);
    generate_nuxtjs_capnp_config(&workerd_func_dir)?;

    tracing::info!(
        "workerd artifacts generated for '{}' at {:?}",
        function_name,
        workerd_func_dir
    );

    Ok(workerd_func_dir)
}

fn embed_static_assets(public_dir: &Path, workerd_func_dir: &Path) -> Result<usize> {
    if !public_dir.exists() {
        return Ok(0);
    }

    let mime_types: HashMap<&str, &str> = [
        (".js", "application/javascript"),
        (".mjs", "application/javascript"),
        (".css", "text/css"),
        (".json", "application/json"),
        (".html", "text/html"),
        (".svg", "image/svg+xml"),
        (".png", "image/png"),
        (".jpg", "image/jpeg"),
        (".jpeg", "image/jpeg"),
        (".gif", "image/gif"),
        (".ico", "image/x-icon"),
        (".woff", "font/woff"),
        (".woff2", "font/woff2"),
        (".ttf", "font/ttf"),
        (".webp", "image/webp"),
        (".avif", "image/avif"),
        (".txt", "text/plain"),
        (".xml", "application/xml"),
    ]
    .iter()
    .copied()
    .collect();

    let mut entries = Vec::new();
    let mut count = 0usize;
    walk_and_embed(public_dir, public_dir, &mime_types, &mut entries, &mut count)?;

    let entries_str = entries.join(",\n");
    let code = format!(
        "// Auto-generated by fugue — do not edit\n\
         const assets = new Map([{}]);\n\
         export default assets;\n",
        entries_str
    );

    let assets_path = workerd_func_dir.join("static-assets.mjs");
    std::fs::write(&assets_path, code)?;

    Ok(count)
}

fn walk_and_embed(
    base_dir: &Path,
    current_dir: &Path,
    mime_types: &HashMap<&str, &str>,
    entries: &mut Vec<String>,
    count: &mut usize,
) -> Result<()> {
    for entry in std::fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            walk_and_embed(base_dir, &path, mime_types, entries, count)?;
        } else if path.is_file() {
            let relative = path.strip_prefix(base_dir).unwrap_or(&path);
            let key = format!("/{}", relative.display());

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let ext_with_dot = format!(".{}", ext);
            let mime = mime_types
                .get(ext_with_dot.as_str())
                .copied()
                .unwrap_or("application/octet-stream");

            let content = std::fs::read(&path)?;
            let b64 = BASE64.encode(&content);

            entries.push(format!(
                "[{}, {{ mime: {}, data: \"{}\" }}]",
                serde_json::to_string(&key)?,
                serde_json::to_string(mime)?,
                b64
            ));
            *count += 1;
        }
    }
    Ok(())
}

fn bundle_server_with_esbuild(server_dir: &Path, workerd_func_dir: &Path) -> Result<()> {
    let index_mjs = server_dir.join("index.mjs");
    let bundle_out = workerd_func_dir.join("bundle.mjs");

    let esbuild_bin = find_esbuild()?;

    let mut cmd = if esbuild_bin.file_name().map(|f| f == "npx").unwrap_or(false) {
        let mut c = std::process::Command::new(&esbuild_bin);
        c.arg("esbuild");
        c
    } else {
        std::process::Command::new(&esbuild_bin)
    };

    let output = cmd
        .arg(&index_mjs)
        .arg("--bundle")
        .arg("--format=esm")
        .arg(format!("--outfile={}", bundle_out.display()))
        .arg("--external:node:*")
        .arg("--external:cloudflare:workers")
        .arg("--conditions=workerd")
        .arg("--platform=node")
        .output()
        .map_err(|e| FugueError::BuildError(format!("Failed to run esbuild: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FugueError::BuildError(format!("esbuild failed: {}", stderr)));
    }

    tracing::info!("Bundle written to {:?}", bundle_out);
    Ok(())
}

fn generate_entry_worker(workerd_func_dir: &Path) -> Result<()> {
    let entry_code = r#"// Auto-generated by fugue — do not edit
import assets from "static-assets.mjs";

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const pathname = url.pathname;

    // Serve static assets for known file types
    if (pathname.startsWith("/_nuxt/") || pathname.match(/\.(js|css|json|svg|png|jpg|jpeg|gif|ico|woff2?|ttf|eot|webp|avif|txt|xml)$/)) {
      const asset = assets.get(pathname);
      if (asset) {
        return new Response(
          Uint8Array.from(atob(asset.data), c => c.charCodeAt(0)),
          {
            headers: {
              "Content-Type": asset.mime,
              "Cache-Control": "public, max-age=31536000, immutable",
            },
          }
        );
      }
    }

    // Delegate to SSR handler via service binding
    return env.SSR.fetch(request);
  },
};
"#;
    let entry_path = workerd_func_dir.join("entry.mjs");
    std::fs::write(&entry_path, entry_code)?;
    Ok(())
}

fn generate_nuxtjs_capnp_config(workerd_func_dir: &Path) -> Result<()> {
    let config = r#"using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [
    (name = "main", worker = .entryWorker),
    (name = "ssr", worker = .ssrWorker),
    (name = "static", worker = .staticWorker),
  ],
  sockets = [
    ( name = "http",
      address = "*:8787",
      http = (),
      service = "main"
    ),
  ],
);

const entryWorker :Workerd.Worker = (
  modules = [
    (name = "entry.mjs", esModule = embed "entry.mjs"),
    (name = "static-assets.mjs", esModule = embed "static-assets.mjs"),
  ],
  compatibilityDate = "2026-04-21",
  compatibilityFlags = ["nodejs_compat"],
  bindings = [
    (name = "SSR", service = "ssr"),
    (name = "STATIC", service = "static"),
  ],
);

const ssrWorker :Workerd.Worker = (
  modules = [
    (name = "bundle.mjs", esModule = embed "bundle.mjs"),
  ],
  compatibilityDate = "2026-04-21",
  compatibilityFlags = ["nodejs_compat"],
  bindings = [
    (name = "ASSETS", service = "static"),
  ],
);

const staticWorker :Workerd.Worker = (
  modules = [
    (name = "static-assets.mjs", esModule = embed "static-assets.mjs"),
  ],
  compatibilityDate = "2026-04-21",
);
"#;
    let config_path = workerd_func_dir.join("config.capnp");
    std::fs::write(&config_path, config)?;
    Ok(())
}
