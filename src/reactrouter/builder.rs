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

    pub fn build_command(&self) -> Vec<&str> {
        match self {
            PackageManager::Npm => vec!["npm", "run", "build"],
            PackageManager::Yarn => vec!["yarn", "build"],
            PackageManager::Pnpm => vec!["pnpm", "build"],
        }
    }
}

pub struct BuildResult {
    pub output_size: u64,
    pub build_time_ms: u128,
}

pub fn build_reactrouter_project(project_dir: &Path, skip_install: bool) -> Result<BuildResult> {
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

    info!("Building React Router project...");
    let build_cmd = pm.build_command();
    let build_status = Command::new(build_cmd[0])
        .args(&build_cmd[1..])
        .current_dir(project_dir)
        .status()
        .map_err(|e| FugueError::BuildError(format!("Failed to run build: {}", e)))?;

    if !build_status.success() {
        return Err(FugueError::BuildError("Build failed".to_string()));
    }

    let build_dir = project_dir.join("build");
    if !build_dir.exists() {
        return Err(FugueError::BuildError(
            "build directory not found after build".to_string(),
        ));
    }

    let output_size = calculate_dir_size(&build_dir)?;
    let build_time_ms = start_time.elapsed().as_millis();

    info!(
        "Build completed in {}ms, output size: {} bytes",
        build_time_ms, output_size
    );

    Ok(BuildResult {
        output_size,
        build_time_ms,
    })
}

fn calculate_dir_size(dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;

    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                total_size += calculate_dir_size(&path)?;
            } else {
                total_size += entry.metadata()?.len();
            }
        }
    }

    Ok(total_size)
}
