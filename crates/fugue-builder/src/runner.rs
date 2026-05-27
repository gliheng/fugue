use fugue_common::error::{FugueError, Result};
use fugue_common::models::BuildTask;
use fugue_common::package::PackageManager;
use fugue_common::fs::calculate_dir_size;
use std::path::Path;
use std::process::Command;
use tracing::info;

pub async fn build_worker(task: &BuildTask) -> Result<(u64, u128)> {
    let start_time = std::time::Instant::now();
    let pm = PackageManager::detect(&task.source_path);

    info!("Detected package manager: {:?}", pm);

    if !task.skip_install {
        info!("Installing dependencies...");
        let install_status = Command::new(pm.install_command())
            .arg("install")
            .current_dir(&task.source_path)
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

    let entry_point = find_entry_point(&task.source_path)?;
    let output_path = task.source_path.join("worker.js");

    info!("Bundling worker with esbuild from {:?}...", entry_point);
    bundle_with_esbuild(&entry_point, &output_path)?;

    if !output_path.exists() {
        return Err(FugueError::BuildError(
            "worker.js not found after bundling".to_string(),
        ));
    }

    let output_size = std::fs::metadata(&output_path)?.len();
    let build_time_ms = start_time.elapsed().as_millis();

    info!(
        "Worker build completed in {}ms, output size: {} bytes",
        build_time_ms, output_size
    );

    Ok((output_size, build_time_ms))
}

pub async fn build_nuxtjs(task: &BuildTask) -> Result<(u64, u128)> {
    let start_time = std::time::Instant::now();
    let pm = PackageManager::detect(&task.source_path);

    info!("Detected package manager: {:?}", pm);

    if !task.skip_install {
        info!("Installing dependencies...");
        let install_status = Command::new(pm.install_command())
            .arg("install")
            .current_dir(&task.source_path)
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

    info!("Building Nuxt project...");
    let build_cmd = pm.build_command();
    let build_status = Command::new(build_cmd[0])
        .args(&build_cmd[1..])
        .current_dir(&task.source_path)
        .status()
        .map_err(|e| FugueError::BuildError(format!("Failed to run build: {}", e)))?;

    if !build_status.success() {
        return Err(FugueError::BuildError("Build failed".to_string()));
    }

    let output_dir = task.source_path.join(".output");
    if !output_dir.exists() {
        return Err(FugueError::BuildError(
            ".output directory not found after build".to_string(),
        ));
    }

    let output_size = calculate_dir_size(&output_dir)?;
    let build_time_ms = start_time.elapsed().as_millis();

    info!(
        "Nuxt.js build completed in {}ms, output size: {} bytes",
        build_time_ms, output_size
    );

    Ok((output_size, build_time_ms))
}

pub async fn build_reactrouter(task: &BuildTask) -> Result<(u64, u128)> {
    let start_time = std::time::Instant::now();
    let pm = PackageManager::detect(&task.source_path);

    info!("Detected package manager: {:?}", pm);

    if !task.skip_install {
        info!("Installing dependencies...");
        let install_status = Command::new(pm.install_command())
            .arg("install")
            .current_dir(&task.source_path)
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
        .current_dir(&task.source_path)
        .status()
        .map_err(|e| FugueError::BuildError(format!("Failed to run build: {}", e)))?;

    if !build_status.success() {
        return Err(FugueError::BuildError("Build failed".to_string()));
    }

    let build_dir = task.source_path.join("build");
    if !build_dir.exists() {
        return Err(FugueError::BuildError(
            "build directory not found after build".to_string(),
        ));
    }

    let output_size = calculate_dir_size(&build_dir)?;
    let build_time_ms = start_time.elapsed().as_millis();

    info!(
        "React Router build completed in {}ms, output size: {} bytes",
        build_time_ms, output_size
    );

    Ok((output_size, build_time_ms))
}

fn find_entry_point(project_dir: &Path) -> Result<std::path::PathBuf> {
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

    let result = cmd
        .arg(entry)
        .arg("--bundle")
        .arg("--format=esm")
        .arg(format!("--outfile={}", output.display()))
        .arg("--external:cloudflare:workers")
        .arg("--conditions=workerd")
        .arg("--platform=browser")
        .output()
        .map_err(|e| FugueError::BuildError(format!("Failed to run esbuild: {}", e)))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(FugueError::BuildError(format!("esbuild failed: {}", stderr)));
    }

    info!("Worker bundle written to {:?}", output);
    Ok(())
}
