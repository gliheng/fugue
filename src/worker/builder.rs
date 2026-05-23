#![allow(dead_code)]

use crate::error::{FugueError, Result};
use std::path::Path;
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone, Copy)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
}

impl PackageManager {
    pub fn detect(project_dir: &Path) -> Self {
        if project_dir.join("pnpm-lock.yaml").exists() {
            PackageManager::Pnpm
        } else if project_dir.join("yarn.lock").exists() {
            PackageManager::Yarn
        } else {
            PackageManager::Npm
        }
    }

    pub fn install_command(&self) -> &str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Pnpm => "pnpm",
        }
    }
}

pub struct BuildResult {
    pub output_size: u64,
    pub build_time_ms: u128,
}

pub fn build_worker_project(project_dir: &Path, skip_install: bool) -> Result<BuildResult> {
    let start_time = std::time::Instant::now();
    let pm = PackageManager::detect(project_dir);

    info!("Detected package manager: {:?}", pm);

    if !skip_install {
        info!("Installing dependencies...");
        let install_status = Command::new(pm.install_command())
            .arg("install")
            .current_dir(project_dir)
            .status()
            .map_err(|e| FugueError::BuildError(format!("Failed to run install: {}", e)))?;

        if !install_status.success() {
            return Err(FugueError::BuildError(
                "Dependency installation failed".to_string(),
            ));
        }
    } else {
        info!("Skipping dependency installation");
    }

    let entry_point = find_entry_point(project_dir)?;
    let output_path = project_dir.join("worker.js");

    info!("Bundling worker with esbuild from {:?}...", entry_point);
    bundle_with_esbuild(&entry_point, &output_path)?;

    if !output_path.exists() {
        return Err(FugueError::BuildError(
            " worker.js not found after bundling".to_string(),
        ));
    }

    let output_size = std::fs::metadata(&output_path)?.len();
    let build_time_ms = start_time.elapsed().as_millis();

    info!(
        "Worker build completed in {}ms, output size: {} bytes",
        build_time_ms, output_size
    );

    Ok(BuildResult {
        output_size,
        build_time_ms,
    })
}

fn find_entry_point(project_dir: &Path) -> Result<PathBuf> {
    let candidates = [
        project_dir.join("src").join("index.ts"),
        project_dir.join("src").join("index.js"),
        project_dir.join("src").join("index.mjs"),
        project_dir.join("index.ts"),
        project_dir.join("index.js"),
        project_dir.join("index.mjs"),
        project_dir.join("worker.ts"),
        project_dir.join("worker.js"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    Err(FugueError::BuildError(
        "No entry point found. Expected src/index.ts, src/index.js, index.ts, or index.js"
            .to_string(),
    ))
}

fn bundle_with_esbuild(entry: &Path, output: &Path) -> Result<()> {
    let esbuild_bin = find_esbuild()?;

    let mut cmd = if esbuild_bin
        .file_name()
        .map(|f| f == "npx")
        .unwrap_or(false)
    {
        let mut c = Command::new(&esbuild_bin);
        c.arg("esbuild");
        c
    } else {
        Command::new(&esbuild_bin)
    };

    let output = cmd
        .arg(entry)
        .arg("--bundle")
        .arg("--format=esm")
        .arg(format!("--outfile={}", output.display()))
        .arg("--external:cloudflare:workers")
        .arg("--conditions=workerd")
        .arg("--platform=browser")
        .output()
        .map_err(|e| FugueError::BuildError(format!("Failed to run esbuild: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FugueError::BuildError(format!("esbuild failed: {}", stderr)));
    }

    info!("Worker bundle written to {:?}", output);
    Ok(())
}

use std::path::PathBuf;

fn find_esbuild() -> Result<PathBuf> {
    let local = std::env::current_dir()
        .unwrap_or_default()
        .join("node_modules/.bin/esbuild");
    if local.exists() {
        return Ok(local);
    }

    if let Ok(output) = Command::new("npx")
        .args(["esbuild", "--version"])
        .output()
    {
        if output.status.success() {
            return Ok(PathBuf::from("npx"));
        }
    }

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