use crate::api::{ApiError, AppState};
use crate::config::PlatformConfig;
use crate::db::{crud, models};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

pub async fn deploy_app(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_req): Json<models::DeployRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let app = crud::get_app(&state.db, id).await?;

    // Create build record
    let build = crud::create_build(&state.db, id).await?;

    // Update app status to building
    crud::update_app(&state.db, id, None, None, None, Some("building"), None, None).await?;

    // Spawn build task
    let db = state.db.clone();
    let config = state.config.clone();
    let process = state.process.clone();
    let app_id = id;
    let build_id = build.id;
    let framework = app.framework.clone();
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

    Ok(Json(serde_json::json!({
        "build_id": build.id,
        "status": "building",
    })))
}

pub async fn redeploy_app(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let app = crud::get_app(&state.db, id).await?;

    if app.source_path.is_none() {
        return Err(ApiError(crate::error::FugueError::ValidationError(
            "No source code uploaded. Upload source first before deploying.".to_string(),
        )));
    }

    // Same as deploy but from existing source
    let build = crud::create_build(&state.db, id).await?;

    crud::update_app(&state.db, id, None, None, None, Some("building"), None, None).await?;

    let db = state.db.clone();
    let config = state.config.clone();
    let process = state.process.clone();
    let app_id = id;
    let build_id = build.id;
    let framework = app.framework.clone();
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

    Ok(Json(serde_json::json!({
        "build_id": build.id,
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
    _config: &PlatformConfig,
    process: &std::sync::Arc<tokio::sync::RwLock<crate::process::ProcessManager>>,
    app_id: Uuid,
    build_id: Uuid,
    framework: &str,
    _app_name: &str,
    app_slug: &str,
) -> Result<(), crate::error::FugueError> {
    crud::update_build(db, build_id, "running", None, None).await?;

    let app_dir = crate::config::apps_data_dir().join(app_id.to_string());
    let source_dir = app_dir.join("source");
    let build_dir = app_dir.join("build");
    std::fs::create_dir_all(&build_dir)?;

    let workerd_dir = crate::config::workerd_dir();

    match framework {
        "worker" => {
            let build_result = crate::worker::build_worker_project(&source_dir, false)?;
            tracing::info!("Worker build completed in {}ms", build_result.build_time_ms);

            crate::runtime::generate_worker_workerd_artifacts(
                app_slug,
                &source_dir,
                &workerd_dir,
            )?;
        }
        "nuxtjs" => {
            // Build with npm
            let build_result =
                crate::nuxtjs::build_nuxt_project(&source_dir, false)?;
            tracing::info!("Nuxt.js build completed in {}ms", build_result.build_time_ms);

            let output_dir = source_dir.join(".output");
            crate::nuxtjs::validate_build_output(&source_dir)?;

            crate::runtime::generate_nuxtjs_workerd_artifacts(
                app_slug,
                &output_dir,
                &workerd_dir,
            )?;
        }
        "react-router" => {
            let build_result =
                crate::reactrouter::build_reactrouter_project(&source_dir, false)?;
            tracing::info!(
                "React Router build completed in {}ms",
                build_result.build_time_ms
            );

            crate::reactrouter::validate_build_output(&source_dir)?;

            let output_dir = source_dir.join("build");
            crate::runtime::generate_reactrouter_workerd_artifacts(
                app_slug,
                &output_dir,
                &workerd_dir,
            )?;
        }
        _ => {
            return Err(crate::error::FugueError::ValidationError(format!(
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


