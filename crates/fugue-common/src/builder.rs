use crate::error::{FugueError, Result};
use crate::fs::calculate_dir_size;
use crate::package::PackageManager;
use crate::project_config::ProjectConfig;
use std::path::Path;
use std::process::Command;
use tracing::info;

pub struct BuildResult {
    pub output_size: u64,
    pub build_time_ms: u128,
}

pub fn build_project(source_dir: &Path, framework: &str, skip_install: bool) -> Result<BuildResult> {
    let start_time = std::time::Instant::now();
    let pm = PackageManager::detect(source_dir);
    let config = ProjectConfig::load(source_dir, framework)?;

    info!("Detected package manager: {:?}", pm);

    // Install dependencies
    if !skip_install {
        info!("Installing dependencies...");
        let install_cmd = config.get_install_command(&pm);
        run_shell_command(&install_cmd, source_dir)?;
    } else {
        info!("Skipping dependency installation");
    }

    // Build
    match framework {
        "worker" => build_worker(source_dir, &config)?,
        "nuxtjs" => build_framework(source_dir, &config, &pm, framework)?,
        "react-router" => build_framework(source_dir, &config, &pm, framework)?,
        "vite" => build_framework(source_dir, &config, &pm, framework)?,
        _ => {
            return Err(FugueError::ValidationError(format!(
                "Unknown framework: {}",
                framework
            )))
        }
    }

    // Validate output
    let output_size = validate_and_measure_output(source_dir, framework, &config)?;
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

fn build_worker(source_dir: &Path, config: &ProjectConfig) -> Result<()> {
    match &config.build_command {
        Some(build_cmd) => {
            info!("Building worker with command: {}", build_cmd);
            run_shell_command(build_cmd, source_dir)
        }
        None => Err(FugueError::BuildError(
            "Worker projects require build.command in fugue.toml".to_string(),
        )),
    }
}

fn build_framework(
    source_dir: &Path,
    config: &ProjectConfig,
    pm: &PackageManager,
    framework: &str,
) -> Result<()> {
    let build_cmd = config.get_build_command(pm);
    info!("Building {} project...", framework);
    run_shell_command(&build_cmd, source_dir)?;
    Ok(())
}

fn validate_and_measure_output(
    source_dir: &Path,
    framework: &str,
    config: &ProjectConfig,
) -> Result<u64> {
    match framework {
        "worker" => {
            let worker_js = source_dir.join("worker.js");
            if !worker_js.exists() {
                return Err(FugueError::BuildError(
                    "worker.js not found after build".to_string(),
                ));
            }
            Ok(std::fs::metadata(&worker_js)?.len())
        }
        _ => {
            let output_dir = source_dir.join(&config.build_output_dir);
            if !output_dir.exists() {
                return Err(FugueError::BuildError(format!(
                    "{} directory not found after build",
                    config.build_output_dir
                )));
            }
            calculate_dir_size(&output_dir)
        }
    }
}

fn run_shell_command(cmd: &str, cwd: &Path) -> Result<()> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(cwd)
        .status()
        .map_err(|e| FugueError::BuildError(format!("Failed to run '{}': {}", cmd, e)))?;

    if !status.success() {
        return Err(FugueError::BuildError(format!(
            "Command failed: {}",
            cmd
        )));
    }

    Ok(())
}
