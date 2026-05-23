use crate::api::{ApiError, AppState};
use crate::db::crud;
use crate::db::models;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

pub async fn list_workspaces(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let workspaces = crud::list_workspaces(&state.db).await?;
    Ok(Json(workspaces))
}

pub async fn get_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let workspace = crud::get_workspace(&state.db, id).await?;
    Ok(Json(workspace))
}

pub async fn create_workspace(
    State(state): State<AppState>,
    Json(req): Json<models::CreateWorkspaceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if models::Framework::from_str(&req.framework).is_none() {
        return Err(ApiError(crate::error::FugueError::ValidationError(format!(
            "Invalid framework '{}'. Must be one of: worker, nuxtjs, react-router",
            req.framework
        ))));
    }

    let name = req.name.unwrap_or_else(|| crud::generate_workspace_name());

    let files = crate::templates::get_template_files(&req.framework)
        .map_err(|e| crate::error::FugueError::ValidationError(e))?;

    let files_map: std::collections::HashMap<String, String> = files
        .into_iter()
        .filter_map(|(path, content)| {
            String::from_utf8(content).ok().map(|text| (path, text))
        })
        .collect();

    let files_json = serde_json::to_value(files_map)
        .map_err(|e| crate::error::FugueError::Other(format!("Failed to serialize files: {}", e)))?;

    let workspace = crud::create_workspace(&state.db, &name, &req.framework, &files_json).await?;

    Ok((axum::http::StatusCode::CREATED, Json(workspace)))
}

pub async fn update_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<models::UpdateWorkspaceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let workspace = crud::update_workspace(
        &state.db,
        id,
        req.name.as_deref(),
        req.files.as_ref(),
    )
    .await?;

    Ok(Json(workspace))
}

pub async fn delete_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    crud::delete_workspace(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}