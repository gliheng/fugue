use crate::config::PlatformConfig;
use crate::db::{crud, init_pool};
use crate::process::ProcessManager;
use axum::response::IntoResponse;
use fugue_common::error::{FugueError, Result};
use futures_util::StreamExt;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub async fn start_platform(db_url: Option<&str>, port: u16) -> Result<()> {
    let mut config = PlatformConfig::load()?;

    if let Some(url) = db_url {
        config.database.url = url.to_string();
    }
    config.platform.port = port;

    std::fs::create_dir_all(crate::config::fugue_dir())?;
    std::fs::create_dir_all(crate::config::workerd_dir())?;
    std::fs::create_dir_all(crate::config::apps_data_dir())?;

    config.save()?;

    write_pid_file(std::process::id() as i32)?;

    init_tracing(&config.logging.level);

    tracing::info!("Starting Fugue platform v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Database: {}", mask_db_url(&config.database.url));
    tracing::info!("API: {}:{}", config.platform.host, config.platform.port);
    tracing::info!("Domain: {}", config.platform.domain);
    tracing::info!(
        "Workerd: http://{}.{}:{}",
        "<app>",
        config.platform.domain,
        config.workerd.port
    );

    let db = init_pool(&config.database.url).await?;
    tracing::info!("Connected to PostgreSQL");

    // Connect to NATS
    let nats_client = match async_nats::connect(&config.nats.url).await {
        Ok(client) => {
            tracing::info!("Connected to NATS at {}", config.nats.url);
            Some(client)
        }
        Err(e) => {
            tracing::warn!(
                "Failed to connect to NATS: {}. Builds will run in-process.",
                e
            );
            None
        }
    };

    let pm = ProcessManager::new(&config)?;
    let process = Arc::new(RwLock::new(pm));

    let state = crate::api::AppState {
        db: db.clone(),
        process: process.clone(),
        config: config.clone(),
        nats: nats_client.clone(),
    };

    // Start background tasks for NATS if connected
    if let Some(nats) = nats_client.clone() {
        let db_clone = db.clone();
        let process_clone = process.clone();
        let nats_clone1 = nats.clone();

        // Listen for build results
        tokio::spawn(async move {
            listen_build_results(nats_clone1, db_clone, process_clone).await;
        });

        // Listen for build logs and persist to DB
        let nats_clone2 = nats.clone();
        let db_clone2 = db.clone();
        tokio::spawn(async move {
            persist_build_logs(nats_clone2, db_clone2).await;
        });
    }

    let api_router = crate::api::api_router(state);

    let app = api_router.fallback(|_req: axum::extract::Request| async move {
        (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({
                "error": "Not found",
                "hint": format!("API endpoints are at /api/v1/*. App traffic is served on the workerd port directly."),
            })),
        ).into_response()
    });

    {
        let all_apps = crud::list_apps(&db, None, None).await?;
        let running_apps: Vec<_> = all_apps
            .into_iter()
            .filter(|a| a.status == "running")
            .collect();

        if !running_apps.is_empty() {
            let mut pm = process.write().await;
            pm.start(&running_apps).await?;
            tracing::info!("Started workerd with {} running apps", running_apps.len());
        }
    }

    let addr = format!("{}:{}", config.platform.host, config.platform.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| FugueError::ProcessError(format!("Server error: {}", e)))?;

    Ok(())
}

pub async fn stop_platform() -> Result<()> {
    let pid_path = crate::config::pid_path();

    if !pid_path.exists() {
        println!("Fugue platform is not running (no PID file found)");
        return Ok(());
    }

    let pid_str = std::fs::read_to_string(&pid_path)?;
    let pid: i32 = pid_str.trim().parse().map_err(|e| {
        FugueError::ProcessError(format!("Invalid PID in {}: {}", pid_path.display(), e))
    })?;

    if !is_process_alive(pid) {
        println!("Removing stale PID file for process {}", pid);
        std::fs::remove_file(&pid_path)?;
        return Ok(());
    }

    if pid == std::process::id() as i32 {
        return Err(FugueError::ProcessError(
            "Cannot stop the platform from within itself".to_string(),
        ));
    }

    println!("Stopping Fugue platform (PID {})...", pid);
    signal::kill(Pid::from_raw(pid), Signal::SIGTERM).map_err(|e| {
        FugueError::ProcessError(format!("Failed to send SIGTERM to platform process: {}", e))
    })?;

    // Wait up to 10 seconds for the process to exit
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if !is_process_alive(pid) {
            std::fs::remove_file(&pid_path)?;
            println!("Fugue platform stopped");
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // If still alive, send SIGKILL
    println!("Platform did not stop gracefully, forcing shutdown...");
    signal::kill(Pid::from_raw(pid), Signal::SIGKILL).map_err(|e| {
        FugueError::ProcessError(format!("Failed to send SIGKILL to platform process: {}", e))
    })?;

    let kill_timeout = Duration::from_secs(5);
    let start = std::time::Instant::now();
    while start.elapsed() < kill_timeout {
        if !is_process_alive(pid) {
            std::fs::remove_file(&pid_path)?;
            println!("Fugue platform stopped");
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    Err(FugueError::ProcessError(
        "Platform process did not terminate".to_string(),
    ))
}

fn write_pid_file(pid: i32) -> Result<()> {
    let pid_path = crate::config::pid_path();
    std::fs::write(&pid_path, format!("{}\n", pid))?;
    Ok(())
}

fn is_process_alive(pid: i32) -> bool {
    signal::kill(Pid::from_raw(pid), None).is_ok()
}

async fn listen_build_results(
    nats: async_nats::Client,
    db: sqlx::PgPool,
    process: Arc<RwLock<ProcessManager>>,
) {
    let mut subscriber = match nats.subscribe("fugue.build.results.>").await {
        Ok(sub) => sub,
        Err(e) => {
            tracing::error!("Failed to subscribe to build results: {}", e);
            return;
        }
    };

    tracing::info!("Listening for build results on fugue.build.results.>");

    while let Some(msg) = subscriber.next().await {
        let result: fugue_common::models::BuildResult = match serde_json::from_slice(&msg.payload) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to deserialize build result: {}", e);
                continue;
            }
        };

        tracing::info!(
            "Received build result for {}: success={}",
            result.build_id,
            result.success
        );

        let status = if result.success { "success" } else { "failed" };
        let error_msg = result.error.as_deref();

        if !result.success {
            tracing::error!(
                "Build {} for app {} failed: {}",
                result.build_id,
                result.app_id,
                error_msg.unwrap_or("no error details")
            );
        }

        if let Err(e) =
            crate::db::crud::update_build(&db, result.build_id, status, None, error_msg).await
        {
            tracing::error!("Failed to update build status: {}", e);
            continue;
        }

        if result.success {
            // Mark app as deploying while we regenerate the workerd config
            if let Err(e) = crate::db::crud::update_app(
                &db,
                result.app_id,
                None,
                None,
                None,
                Some("deploying"),
                None,
                None,
            )
            .await
            {
                tracing::error!("Failed to update app status to deploying: {}", e);
            }

            // Generate dispatch config and reload workerd
            if let Ok(all_apps) = crate::db::crud::list_apps(&db, None, None).await {
                let running_apps: Vec<_> = all_apps
                    .into_iter()
                    .filter(|a| a.id == result.app_id || a.status == "running")
                    .collect();

                let mut pm = process.write().await;
                match pm.reload(&running_apps).await {
                    Ok(_) => {
                        if let Err(e) = crate::db::crud::update_app(
                            &db,
                            result.app_id,
                            None,
                            None,
                            None,
                            Some("running"),
                            None,
                            None,
                        )
                        .await
                        {
                            tracing::error!("Failed to update app status to running: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to reload workerd: {}", e);
                        if let Err(e) = crate::db::crud::update_app(
                            &db,
                            result.app_id,
                            None,
                            None,
                            None,
                            Some("error"),
                            None,
                            None,
                        )
                        .await
                        {
                            tracing::error!("Failed to update app status to error: {}", e);
                        }
                    }
                }
            } else {
                tracing::error!("Failed to list apps for workerd reload");
                if let Err(e) = crate::db::crud::update_app(
                    &db,
                    result.app_id,
                    None,
                    None,
                    None,
                    Some("error"),
                    None,
                    None,
                )
                .await
                {
                    tracing::error!("Failed to update app status to error: {}", e);
                }
            }
        } else {
            if let Err(e) = crate::db::crud::update_app(
                &db,
                result.app_id,
                None,
                None,
                None,
                Some("error"),
                None,
                None,
            )
            .await
            {
                tracing::error!("Failed to update app status: {}", e);
            }
        }
    }
}

async fn persist_build_logs(nats: async_nats::Client, db: sqlx::PgPool) {
    let mut subscriber = match nats.subscribe("fugue.build.logs.>").await {
        Ok(sub) => sub,
        Err(e) => {
            tracing::error!("Failed to subscribe to build logs: {}", e);
            return;
        }
    };

    tracing::info!("Listening for build logs on fugue.build.logs.>");

    while let Some(msg) = subscriber.next().await {
        let log: fugue_common::models::BuildLog = match serde_json::from_slice(&msg.payload) {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to deserialize build log: {}", e);
                continue;
            }
        };

        if let Err(e) = sqlx::query("UPDATE builds SET log = COALESCE(log, '') || $1 WHERE id = $2")
            .bind(format!("{}\n", log.line))
            .bind(log.build_id)
            .execute(&db)
            .await
        {
            tracing::error!("Failed to persist build log: {}", e);
        }
    }
}

fn init_tracing(level: &str) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn mask_db_url(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(slash_pos) = url[..at_pos].rfind('/') {
            let prefix = &url[..slash_pos + 2];
            let suffix = &url[at_pos..];
            return format!("{}***:***{}", prefix, suffix);
        }
    }
    url.to_string()
}
