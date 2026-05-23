use crate::config::PlatformConfig;
use crate::db::{crud, init_pool};
use crate::error::{FugueError, Result};
use crate::process::ProcessManager;
use axum::response::IntoResponse;
use std::sync::Arc;
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

    init_tracing(&config.logging.level);

    tracing::info!("Starting Fugue platform v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Database: {}", mask_db_url(&config.database.url));
    tracing::info!("API: {}:{}", config.platform.host, config.platform.port);
    tracing::info!("Domain: {}", config.platform.domain);
    tracing::info!("Workerd: http://{}.{}:{}", "<app>", config.platform.domain, config.workerd.port);

    let db = init_pool(&config.database.url).await?;
    tracing::info!("Connected to PostgreSQL");

    let pm = ProcessManager::new(&config)?;
    let process = Arc::new(RwLock::new(pm));

    let state = crate::api::AppState {
        db: db.clone(),
        process: process.clone(),
        config: config.clone(),
    };

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

    axum::serve(listener, app).await.map_err(|e| {
        FugueError::ProcessError(format!("Server error: {}", e))
    })?;

    Ok(())
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