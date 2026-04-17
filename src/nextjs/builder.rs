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

/// Detect which package manager to use (npm, pnpm, or yarn)
fn detect_package_manager(source_dir: &Path) -> &'static str {
    if source_dir.join("pnpm-lock.yaml").exists() {
        "pnpm"
    } else if source_dir.join("yarn.lock").exists() {
        "yarn"
    } else {
        "npm"
    }
}

/// Build a Next.js project
pub async fn build_nextjs_project(ctx: &BuildContext) -> Result<BuildResult> {
    let start = Instant::now();

    // Detect package manager
    let pkg_manager = detect_package_manager(&ctx.source_dir);

    // Install dependencies
    println!("Installing dependencies with {}...", pkg_manager);
    tracing::info!("Installing dependencies with {}...", pkg_manager);
    let install_cmd = match pkg_manager {
        "pnpm" => "pnpm install",
        "yarn" => "yarn install",
        _ => "npm install",
    };

    let install_output = Command::new("sh")
        .arg("-c")
        .arg(install_cmd)
        .current_dir(&ctx.source_dir)
        .output()
        .map_err(|e| FugueError::BuildError(format!("Failed to run install: {}", e)))?;

    if !install_output.status.success() {
        let stderr = String::from_utf8_lossy(&install_output.stderr);
        return Err(FugueError::BuildError(format!(
            "Dependency installation failed: {}",
            stderr
        )));
    }

    println!("Dependencies installed successfully");

    // Run next build
    println!("Building Next.js project (this may take a few minutes)...");
    tracing::info!("Building Next.js project...");
    let build_cmd = match pkg_manager {
        "pnpm" => "pnpm run build",
        "yarn" => "yarn build",
        _ => "npm run build",
    };

    let build_output = Command::new("sh")
        .arg("-c")
        .arg(build_cmd)
        .current_dir(&ctx.source_dir)
        .output()
        .map_err(|e| FugueError::BuildError(format!("Failed to run build: {}", e)))?;

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        return Err(FugueError::BuildError(format!("Build failed: {}", stderr)));
    }

    println!("Build completed successfully");

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
