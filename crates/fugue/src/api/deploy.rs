use crate::api::{ApiError, AppState};
use crate::db::crud;
use fugue_common::models;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
    Json,
};
use std::path::Path as StdPath;
use uuid::Uuid;

fn detect_framework(source_dir: &StdPath) -> String {
    // Try framework-specific detectors first, falling back to worker.
    if crate::vite::detection::detect_vite_project(source_dir).is_ok() {
        return "vite".to_string();
    }
    if crate::reactrouter::detection::detect_reactrouter_project(source_dir).is_ok() {
        return "react-router".to_string();
    }
    if crate::nuxtjs::detection::detect_nuxt_project(source_dir).is_ok() {
        return "nuxtjs".to_string();
    }
    if crate::worker::detection::detect_worker_project(source_dir).is_ok() {
        return "worker".to_string();
    }
    // If nothing matches, keep the source as a generic worker project.
    "worker".to_string()
}

async fn maybe_update_framework(db: &sqlx::PgPool, app: &crate::db::models::App, source_dir: &StdPath) -> Result<String, fugue_common::error::FugueError> {
    let detected = detect_framework(source_dir);
    if detected != app.framework {
        tracing::info!(
            "App {} framework mismatch: stored='{}', detected='{}'; updating",
            app.id,
            app.framework,
            detected
        );
        crud::update_app_framework(db, app.id, &detected).await?;
        Ok(detected)
    } else {
        Ok(app.framework.clone())
    }
}

pub async fn trigger_deploy(
    state: &AppState,
    app: &crate::db::models::App,
    source_path: std::path::PathBuf,
) -> Result<uuid::Uuid, fugue_common::error::FugueError> {
    // Auto-detect framework from the uploaded source and update if mismatched.
    let framework = maybe_update_framework(&state.db, app, &source_path).await?;

    // Create build record
    let build = crud::create_build(&state.db, app.id).await?;

    // Update app status to building
    crud::update_app(&state.db, app.id, None, None, None, Some("building"), None, None).await?;

    // Publish build task to NATS
    let task = fugue_common::models::BuildTask {
        build_id: build.id,
        app_id: app.id,
        app_slug: app.slug.clone(),
        source_path,
        framework: fugue_common::models::Framework::from_str(&framework)
            .unwrap_or(fugue_common::models::Framework::Worker),
        skip_install: false,
    };

    if let Some(nats) = &state.nats {
        let payload = serde_json::to_vec(&task)?;
        nats.publish("fugue.build.requests", payload.into()).await
            .map_err(|e| fugue_common::error::FugueError::NatsError(format!("Failed to publish build task: {}", e)))?;
        tracing::info!("Published build task {} to NATS", build.id);
    } else {
        // Fallback to in-process build if NATS not available
        let db = state.db.clone();
        let config = state.config.clone();
        let process = state.process.clone();
        let app_id = app.id;
        let build_id = build.id;
        let framework = framework.clone();
        let app_name = app.name.clone();
        let app_slug = app.slug.clone();

        tokio::spawn(async move {
            let result =
                run_build(&db, &config, &process, app_id, build_id, &framework, &app_name, &app_slug)
                    .await;

            match result {
                Ok(_) => {
                    tracing::info!("Build {} completed successfully", build_id);
                }
                Err(e) => {
                    tracing::error!("Build {} failed: {:?}", build_id, e);
                    let _ = crud::update_build(
                        &db,
                        build_id,
                        "failed",
                        None,
                        Some(&e.to_string()),
                    )
                    .await;
                    let _ = crud::update_app(
                        &db,
                        app_id,
                        None,
                        None,
                        None,
                        Some("error"),
                        None,
                        None,
                    )
                    .await;
                }
            }
        });
    }

    Ok(build.id)
}

pub async fn deploy_app(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_req): Json<models::DeployRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let app = crud::get_app(&state.db, id).await?;
    let source_path = crate::config::apps_data_dir().join(id.to_string()).join("source");
    let build_id = trigger_deploy(&state, &app, source_path).await?;

    Ok(Json(serde_json::json!({
        "build_id": build_id,
        "status": "building",
    })))
}

pub async fn redeploy_app(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let app = crud::get_app(&state.db, id).await?;

    if app.source_path.is_none() {
        return Err(ApiError(fugue_common::error::FugueError::ValidationError(
            "No source code uploaded. Upload source first before deploying.".to_string(),
        )));
    }

    let source_path = crate::config::apps_data_dir().join(id.to_string()).join("source");
    let build_id = trigger_deploy(&state, &app, source_path).await?;

    Ok(Json(serde_json::json!({
        "build_id": build_id,
        "status": "building",
    })))
}

pub async fn list_builds(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let _app = crud::get_app(&state.db, id).await?;
    let builds = crud::list_builds(&state.db, id).await?;
    Ok(Json(builds))
}

pub async fn get_build(
    State(state): State<AppState>,
    Path((id, build_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let _app = crud::get_app(&state.db, id).await?;
    let build = crud::get_build(&state.db, build_id).await?;
    Ok(Json(build))
}

async fn run_build(
    db: &sqlx::PgPool,
    _config: &fugue_common::config::PlatformConfig,
    process: &std::sync::Arc<tokio::sync::RwLock<crate::process::ProcessManager>>,
    app_id: Uuid,
    build_id: Uuid,
    framework: &str,
    _app_name: &str,
    app_slug: &str,
) -> Result<(), fugue_common::error::FugueError> {
    crud::update_build(db, build_id, "running", None, None).await?;

    let app_dir = crate::config::apps_data_dir().join(app_id.to_string());
    let source_dir = app_dir.join("source");
    let build_dir = app_dir.join("build");
    std::fs::create_dir_all(&build_dir)?;

    let workerd_dir = crate::config::workerd_dir();

    let build_result = fugue_common::builder::build_project(&source_dir, framework, false)?;
    tracing::info!("Build completed in {}ms", build_result.build_time_ms);

    match framework {
        "worker" => {
            crate::runtime::generate_worker_workerd_artifacts(
                app_slug,
                &source_dir,
                &workerd_dir,
            )?;
        }
        "nuxtjs" => {
            crate::nuxtjs::validate_build_output(&source_dir)?;
            crate::runtime::generate_nuxtjs_workerd_artifacts(
                app_slug,
                &source_dir,
                &workerd_dir,
            )?;
        }
        "react-router" => {
            crate::reactrouter::validate_build_output(&source_dir)?;
            crate::runtime::generate_reactrouter_workerd_artifacts(
                app_slug,
                &source_dir,
                &workerd_dir,
            )?;
        }
        "vite" => {
            crate::vite::validate_build_output(&source_dir)?;
            crate::runtime::generate_vite_workerd_artifacts(
                app_slug,
                &source_dir,
                &workerd_dir,
            )?;
        }
        _ => {
            return Err(fugue_common::error::FugueError::ValidationError(format!(
                "Unknown framework: {}",
                framework
            )));
        }
    }

    // Mark build as success
    crud::update_build(db, build_id, "success", Some("Build completed"), None).await?;

    // Update app status and paths
    crud::update_app(
        db,
        app_id,
        None,
        None,
        None,
        Some("deploying"),
        Some(source_dir.to_str().unwrap_or("")),
        Some(build_dir.to_str().unwrap_or("")),
    )
    .await?;

    // Create deployment
    let deployment = crud::create_deployment(db, app_id, build_id).await?;

    // Start the app in workerd
    let all_apps = crud::list_apps(db, None, None).await?;
    let running_apps: Vec<_> = all_apps
        .into_iter()
        .filter(|a| a.id == app_id || a.status == "running")
        .collect();

    let mut pm = process.write().await;
    pm.reload(&running_apps).await?;

    // Update app and deployment status
    crud::update_app(db, app_id, None, None, None, Some("running"), None, None).await?;
    crud::update_deployment_status(db, deployment.id, "running").await?;

    Ok(())
}

pub async fn build_logs_ws(
    ws: WebSocketUpgrade,
    Path((app_id, build_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        stream_build_logs(socket, state.db, app_id, build_id)
    })
}

use axum::extract::ws::{Message, WebSocket};
use std::time::Duration;

async fn stream_build_logs(
    mut socket: WebSocket,
    db: sqlx::PgPool,
    _app_id: Uuid,
    build_id: Uuid,
) {
    tracing::info!("WebSocket connected for build logs: {}", build_id);

    let mut last_offset = 0usize;

    loop {
        // Get the build from DB
        let build = match crate::db::crud::get_build(&db, build_id).await {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("Failed to get build: {}", e);
                let _ = socket.send(Message::Text(format!("ERROR: {}", e))).await;
                return;
            }
        };

        // Send new log lines since last_offset
        if let Some(log) = &build.log {
            let lines: Vec<&str> = log.lines().collect();
            for line in lines.iter().skip(last_offset) {
                let msg = serde_json::json!({
                    "message": line,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });
                if socket.send(Message::Text(msg.to_string())).await.is_err() {
                    return;
                }
            }
            last_offset = lines.len();
        }

        // Check if build is done
        if build.status == "success" || build.status == "failed" {
            let status_msg = if build.status == "success" {
                "BUILD_SUCCESS"
            } else {
                "BUILD_FAILED"
            };
            let _ = socket.send(Message::Text(status_msg.to_string())).await;
            return;
        }

        // Poll every 200ms
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
