use crate::error::{FugueError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[derive(Debug)]
pub struct BuildContext {
    pub source_dir: PathBuf,
    pub build_dir: PathBuf,
    pub function_name: String,
}

#[derive(Debug)]
pub struct BuildResult {
    pub success: bool,
    pub build_time_ms: u64,
    pub output_size_bytes: u64,
    pub standalone_path: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
}

impl PackageManager {
    pub fn detect(source_dir: &Path) -> Self {
        if source_dir.join("pnpm-lock.yaml").exists() {
            PackageManager::Pnpm
        } else if source_dir.join("yarn.lock").exists() {
            PackageManager::Yarn
        } else {
            PackageManager::Npm
        }
    }

    pub fn install_command(&self) -> Vec<&str> {
        match self {
            PackageManager::Npm => vec!["npm", "install"],
            PackageManager::Yarn => vec!["yarn", "install"],
            PackageManager::Pnpm => vec!["pnpm", "install"],
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

/// Build a Next.js project
pub async fn build_nextjs_project(ctx: &BuildContext) -> Result<BuildResult> {
    let start = Instant::now();

    let pkg_manager = PackageManager::detect(&ctx.source_dir);

    // Install dependencies
    tracing::info!("Installing dependencies with {:?}...", pkg_manager);
    let install_cmd = pkg_manager.install_command();
    let install_status = Command::new(install_cmd[0])
        .args(&install_cmd[1..])
        .current_dir(&ctx.source_dir)
        .status()
        .map_err(|e| FugueError::BuildError(format!("Failed to run install: {}", e)))?;

    if !install_status.success() {
        return Err(FugueError::BuildError(
            "Dependency installation failed".to_string(),
        ));
    }

    // Run next build
    tracing::info!("Building Next.js project...");
    let build_cmd = pkg_manager.build_command();
    let build_status = Command::new(build_cmd[0])
        .args(&build_cmd[1..])
        .current_dir(&ctx.source_dir)
        .status()
        .map_err(|e| FugueError::BuildError(format!("Failed to run build: {}", e)))?;

    if !build_status.success() {
        return Err(FugueError::BuildError("Build failed".to_string()));
    }

    tracing::info!("Build completed successfully");

    // Verify .next/standalone exists
    let standalone_path = ctx.source_dir.join(".next/standalone");
    if !standalone_path.exists() {
        return Err(FugueError::BuildError(
            "Build succeeded but .next/standalone not found. Ensure next.config.js has output: 'standalone'".to_string(),
        ));
    }

    // Calculate output size
    let output_size = calculate_dir_size(&standalone_path)?;

    let build_time_ms = start.elapsed().as_millis() as u64;

    Ok(BuildResult {
        success: true,
        build_time_ms,
        output_size_bytes: output_size,
        standalone_path,
    })
}

/// Calculate the total size of a directory
fn calculate_dir_size(path: &Path) -> Result<u64> {
    let mut total_size = 0u64;

    if path.is_file() {
        return Ok(path.metadata()?.len());
    }

    for entry in walkdir::WalkDir::new(path) {
        let entry = entry.map_err(|e| FugueError::Other(format!("Failed to walk directory: {}", e)))?;
        if entry.file_type().is_file() {
            total_size += entry.metadata()
                .map_err(|e| FugueError::Other(format!("Failed to get metadata: {}", e)))?
                .len();
        }
    }

    Ok(total_size)
}
