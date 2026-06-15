use crate::api::{ApiError, AppState};
use crate::db::crud;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

pub async fn start_app(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let app = crud::get_app(&state.db, id).await?;

    if app.status == "running" {
        return Err(ApiError(fugue_common::error::FugueError::AppAlreadyRunning(
            app.name.clone(),
        )));
    }

    if app.status != "deploying" && app.status != "stopped" && app.status != "created" {
        return Err(ApiError(fugue_common::error::FugueError::ValidationError(format!(
            "Cannot start app in '{}' status. App must be deployed first.",
            app.status
        ))));
    }

    // Update status
    crud::update_app(&state.db, id, None, None, None, Some("running"), None, None).await?;

    // Regenerate dispatch config with this app included
    let all_apps = crud::list_apps(&state.db, None, None).await?;
    let running_apps: Vec<_> = all_apps
        .into_iter()
        .filter(|a| a.id == id || a.status == "running")
        .collect();

    let mut pm = state.process.write().await;
    pm.reload(&running_apps).await?;

    let url = format!(
        "http://{}.{}:{}",
        app.subdomain, state.config.platform.domain, state.config.workerd.port
    );

    Ok(Json(serde_json::json!({
        "status": "running",
        "url": url,
    })))
}

pub async fn stop_app(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let app = crud::get_app(&state.db, id).await?;

    if app.status != "running" {
        return Err(ApiError(fugue_common::error::FugueError::AppNotRunning(
            app.name.clone(),
        )));
    }

    // Update status
    crud::update_app(&state.db, id, None, None, None, Some("stopped"), None, None).await?;

    // Regenerate dispatch config without this app
    let all_apps = crud::list_apps(&state.db, None, None).await?;
    let running_apps: Vec<_> = all_apps
        .into_iter()
        .filter(|a| a.status == "running" && a.id != id)
        .collect();

    let mut pm = state.process.write().await;
    pm.reload(&running_apps).await?;

    Ok(Json(serde_json::json!({
        "status": "stopped",
    })))
}

pub async fn app_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let app = crud::get_app(&state.db, id).await?;

    let url = if app.status == "running" {
        Some(format!(
            "http://{}.{}:{}",
            app.subdomain, state.config.platform.domain, state.config.workerd.port
        ))
    } else {
        None
    };

    Ok(Json(serde_json::json!({
        "status": app.status,
        "url": url,
        "updated_at": app.updated_at,
    })))
}
