#![allow(dead_code)]

use fugue_common::error::{FugueError, Result};
use fugue_common::package::PackageManager;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

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
    let esbuild_bin = fugue_common::fs::find_esbuild()?;

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
