use crate::client::DaemonClient;
use crate::error::{FugueError, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub async fn start_command() -> Result<()> {
    // Check if daemon is already running
    if crate::daemon::is_daemon_running()? {
        return Err(FugueError::DaemonAlreadyRunning);
    }

    println!("Starting Fugue daemon...");

    // Fork process to run daemon in background
    #[cfg(unix)]
    {
        use std::process::Command;

        let exe = std::env::current_exe()?;

        Command::new(exe)
            .arg("__daemon")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        // Wait a bit for daemon to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Verify daemon started
        if crate::daemon::is_daemon_running()? {
            println!("✓ Daemon started successfully");
            Ok(())
        } else {
            Err(FugueError::Other("Failed to start daemon".to_string()))
        }
    }

    #[cfg(not(unix))]
    {
        Err(FugueError::Other(
            "Daemon mode not supported on this platform".to_string(),
        ))
    }
}

pub async fn stop_command() -> Result<()> {
    println!("Stopping Fugue daemon...");

    crate::daemon::stop_daemon()?;

    println!("✓ Daemon stopped");
    Ok(())
}

pub async fn status_command() -> Result<()> {
    let client = DaemonClient::new();

    match client.status().await {
        Ok(status) => {
            println!("Daemon Status:");
            println!("{}", serde_json::to_string_pretty(&status)?);
            Ok(())
        }
        Err(_) => {
            println!("Daemon is not running");
            Err(FugueError::DaemonNotRunning)
        }
    }
}

pub async fn deploy_command(
    name: String,
    path: String,
    skip_build: bool,
    env: Vec<String>,
) -> Result<()> {
    let client = DaemonClient::new();

    // Check daemon is running
    if !crate::daemon::is_daemon_running()? {
        return Err(FugueError::DaemonNotRunning);
    }

    // Validate function name
    crate::validation::validate_function_name(&name)?;

    let path_obj = Path::new(&path);

    // Auto-detect: is it a file or directory?
    if path_obj.is_file() {
        // Single-file deployment
        if skip_build || !env.is_empty() {
            return Err(FugueError::ValidationError(
                "Options --skip-build and --env are only valid for Next.js projects".to_string(),
            ));
        }

        let code = fs::read_to_string(&path)?;
        println!("Deploying function '{}'...", name);
        client.deploy(&name, &code).await?;
        println!("✓ Function '{}' deployed successfully", name);
    } else if path_obj.is_dir() {
        // Check if it's a Next.js project
        let nextjs_project = crate::nextjs::detect_nextjs_project(path_obj)?;

        // Check if it's a Nuxt.js project
        let nuxtjs_project = crate::nuxtjs::detect_nuxt_project(path_obj).ok();

        if let Some(project) = nextjs_project {
            // Next.js deployment
            crate::nextjs::validate_nextjs_project(&project)?;

            // Parse environment variables
            let mut env_vars = HashMap::new();
            for env_str in env {
                let parts: Vec<&str> = env_str.splitn(2, '=').collect();
                if parts.len() != 2 {
                    return Err(FugueError::ValidationError(format!(
                        "Invalid environment variable format: {}. Expected KEY=VALUE",
                        env_str
                    )));
                }
                env_vars.insert(parts[0].to_string(), parts[1].to_string());
            }

            println!("Deploying Next.js app '{}'...", name);
            if skip_build {
                println!("Skipping build (using existing .next directory)");
            }

            client
                .deploy_nextjs(&name, &path, skip_build, env_vars)
                .await?;

            println!("✓ Next.js app '{}' deployed successfully", name);
        } else if let Some(project) = nuxtjs_project {
            // Nuxt.js deployment
            // Parse environment variables
            let mut env_vars = HashMap::new();
            for env_str in env {
                let parts: Vec<&str> = env_str.splitn(2, '=').collect();
                if parts.len() != 2 {
                    return Err(FugueError::ValidationError(format!(
                        "Invalid environment variable format: {}. Expected KEY=VALUE",
                        env_str
                    )));
                }
                env_vars.insert(parts[0].to_string(), parts[1].to_string());
            }

            println!("Deploying Nuxt.js app '{}'...", name);
            if skip_build {
                println!("Skipping build (using existing .output directory)");
            }

            // Build if not skipping
            if !skip_build {
                println!("Building Nuxt.js project...");
                let build_result = crate::nuxtjs::build_nuxt_project(path_obj, false)?;
                println!("Build completed in {}ms", build_result.build_time_ms);
            }

            // Validate build output
            crate::nuxtjs::validate_build_output(path_obj)?;

            // Deploy via registry
            let output_dir = path_obj.join(".output");
            let registry = crate::registry::FunctionRegistry::new(crate::config::functions_dir())?;
            let _metadata = registry.deploy_nuxtjs_function(
                &name,
                path_obj,
                &output_dir,
                env_vars,
                project.node_version,
            )?;

            println!("✓ Nuxt.js app '{}' deployed successfully", name);
        } else {
            return Err(FugueError::ValidationError(
                "Directory is not a Next.js or Nuxt.js project. Use a single .js file for simple functions.".to_string(),
            ));
        }
    } else {
        return Err(FugueError::ValidationError(format!(
            "Path not found: {}",
            path
        )));
    }

    Ok(())
}

pub async fn invoke_command(name: String, data: Option<String>) -> Result<()> {
    let client = DaemonClient::new();

    // Check daemon is running
    if !crate::daemon::is_daemon_running()? {
        return Err(FugueError::DaemonNotRunning);
    }

    // Parse data - if no data provided, use empty object
    let input = if let Some(d) = data {
        serde_json::from_str(&d)?
    } else {
        serde_json::json!({})
    };

    println!("Invoking function '{}'...", name);

    let result = client.invoke(&name, input).await?;

    println!("\nResult:");
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

pub async fn list_command() -> Result<()> {
    let client = DaemonClient::new();

    // Check daemon is running
    if !crate::daemon::is_daemon_running()? {
        return Err(FugueError::DaemonNotRunning);
    }

    let functions = client.list().await?;

    if functions.is_empty() {
        println!("No functions deployed");
    } else {
        println!("Deployed Functions:");
        println!();
        for func in functions {
            println!("  • {} (ID: {})", func.name, func.id);
            println!("    Created: {}", func.created_at);
            println!("    Timeout: {}ms", func.timeout_ms);
            println!();
        }
    }

    Ok(())
}

pub async fn delete_command(name: String) -> Result<()> {
    let client = DaemonClient::new();

    // Check daemon is running
    if !crate::daemon::is_daemon_running()? {
        return Err(FugueError::DaemonNotRunning);
    }

    println!("Deleting function '{}'...", name);

    client.delete(&name).await?;

    println!("✓ Function '{}' deleted", name);
    Ok(())
}

pub async fn logs_command(name: String) -> Result<()> {
    println!("Logs for function '{}':", name);
    println!("Note: Logs not yet implemented");
    Ok(())
}

pub async fn deploy_nextjs_command(
    name: String,
    directory: String,
    skip_build: bool,
    env: Vec<String>,
) -> Result<()> {
    let client = DaemonClient::new();

    // Check daemon is running
    if !crate::daemon::is_daemon_running()? {
        return Err(FugueError::DaemonNotRunning);
    }

    // Validate function name
    crate::validation::validate_function_name(&name)?;

    // Validate directory exists
    let dir_path = Path::new(&directory);
    if !dir_path.exists() {
        return Err(FugueError::ValidationError(format!(
            "Directory not found: {}",
            directory
        )));
    }

    // Detect Next.js project
    let project = crate::nextjs::detect_nextjs_project(dir_path)?
        .ok_or_else(|| FugueError::NotNextJsProject("Not a Next.js project".to_string()))?;

    // Validate project
    crate::nextjs::validate_nextjs_project(&project)?;

    // Parse environment variables
    let mut env_vars = HashMap::new();
    for env_str in env {
        let parts: Vec<&str> = env_str.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(FugueError::ValidationError(format!(
                "Invalid environment variable format: {}. Expected KEY=VALUE",
                env_str
            )));
        }
        env_vars.insert(parts[0].to_string(), parts[1].to_string());
    }

    println!("Deploying Next.js app '{}'...", name);
    if skip_build {
        println!("Skipping build (using existing .next directory)");
    }

    client
        .deploy_nextjs(&name, &directory, skip_build, env_vars)
        .await?;

    println!("✓ Next.js app '{}' deployed successfully", name);
    Ok(())
}

pub async fn rebuild_command(name: String) -> Result<()> {
    let client = DaemonClient::new();

    // Check daemon is running
    if !crate::daemon::is_daemon_running()? {
        return Err(FugueError::DaemonNotRunning);
    }

    println!("Rebuilding Next.js app '{}'...", name);

    client.rebuild(&name).await?;

    println!("✓ Next.js app '{}' rebuilt successfully", name);
    Ok(())
}

pub async fn url_command(name: String) -> Result<()> {
    let client = DaemonClient::new();

    // Check daemon is running
    if !crate::daemon::is_daemon_running()? {
        return Err(FugueError::DaemonNotRunning);
    }

    let url = client.get_url(&name).await?;

    if url.is_empty() {
        println!("Function '{}' is not currently running", name);
        println!("Invoke it first to start the workerd process");
    } else {
        println!("Function '{}' is available at:", name);
        println!("{}", url);
        println!("\nYou can access it with:");
        println!("  curl {}", url);
        println!("  open {}", url);
    }

    Ok(())
}

