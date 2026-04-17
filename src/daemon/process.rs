use crate::config;
use crate::daemon::state::DaemonState;
use crate::error::Result;
use std::fs;
use std::sync::Arc;

pub async fn start_daemon() -> Result<()> {
    // Initialize directories
    fs::create_dir_all(config::fugue_dir())?;
    fs::create_dir_all(config::workerd_dir())?;
    fs::create_dir_all(config::functions_dir())?;

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Check if daemon is already running
    if is_daemon_running()? {
        return Err(crate::error::FugueError::DaemonAlreadyRunning);
    }

    // Write PID file
    let pid = std::process::id();
    fs::write(config::daemon_pid_file(), pid.to_string())?;

    tracing::info!("Starting Fugue daemon (PID: {})", pid);

    // Initialize registry
    let registry = crate::registry::FunctionRegistry::new(config::functions_dir())?;

    // Create daemon state
    let state = Arc::new(DaemonState::new(registry));

    // Load existing functions
    state.load_functions().await?;

    tracing::info!("Loaded {} functions", state.functions.read().await.len());

    // Start HTTP server
    crate::daemon::server::run_server(state).await?;

    Ok(())
}

pub fn is_daemon_running() -> Result<bool> {
    let pid_file = config::daemon_pid_file();

    if !pid_file.exists() {
        return Ok(false);
    }

    let pid_str = fs::read_to_string(&pid_file)?;
    let pid: u32 = pid_str.trim().parse().map_err(|_| {
        crate::error::FugueError::Other("Invalid PID file".to_string())
    })?;

    // Check if process is running
    #[cfg(unix)]
    {
        use std::process::Command;
        let output = Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .output()?;

        Ok(output.status.success())
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems, assume it's running if PID file exists
        Ok(true)
    }
}

pub fn stop_daemon() -> Result<()> {
    let pid_file = config::daemon_pid_file();

    if !pid_file.exists() {
        return Err(crate::error::FugueError::DaemonNotRunning);
    }

    let pid_str = fs::read_to_string(&pid_file)?;
    let pid: u32 = pid_str.trim().parse().map_err(|_| {
        crate::error::FugueError::Other("Invalid PID file".to_string())
    })?;

    // Send SIGTERM to daemon
    #[cfg(unix)]
    {
        use std::process::Command;
        Command::new("kill")
            .arg(pid.to_string())
            .output()?;
    }

    #[cfg(not(unix))]
    {
        return Err(crate::error::FugueError::Other(
            "Stop not implemented on this platform".to_string(),
        ));
    }

    // Remove PID file
    fs::remove_file(pid_file)?;

    Ok(())
}
