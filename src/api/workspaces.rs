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

    let ws_dir = crate::config::workspaces_data_dir().join(id.to_string());
    let mut files = std::collections::HashMap::new();
    if ws_dir.exists() {
        crate::api::source::read_source_dir(&ws_dir, &ws_dir, &mut files)?;
    }

    Ok(Json(serde_json::json!({
        "id": workspace.id,
        "name": workspace.name,
        "framework": workspace.framework,
        "file_count": workspace.file_count,
        "files": files,
        "created_at": workspace.created_at,
        "updated_at": workspace.updated_at,
    })))
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

    // Create workspace record first (file_count = 0)
    let workspace = crud::create_workspace(&state.db, &name, &req.framework, 0).await?;

    // Write template files to disk
    let ws_dir = crate::config::workspaces_data_dir().join(workspace.id.to_string());
    std::fs::create_dir_all(&ws_dir)?;

    let template_files = crate::templates::get_template_files(&req.framework)
        .map_err(|e| crate::error::FugueError::ValidationError(e))?;

    let mut file_count = 0i32;
    for (path, content) in &template_files {
        if let Ok(text) = std::str::from_utf8(content) {
            let file_path = ws_dir.join(path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&file_path, text)?;
            file_count += 1;
        }
    }

    // Update file_count
    let workspace = crud::update_workspace(&state.db, workspace.id, None, Some(file_count)).await?;

    Ok((axum::http::StatusCode::CREATED, Json(workspace)))
}

pub async fn update_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<models::UpdateWorkspaceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // If files are provided, write them to disk
    if let Some(ref files_value) = req.files {
        let files: std::collections::HashMap<String, String> =
            serde_json::from_value(files_value.clone()).map_err(|e| {
                crate::error::FugueError::ValidationError(format!("Invalid files format: {}", e))
            })?;

        let ws_dir = crate::config::workspaces_data_dir().join(id.to_string());

        // Clear existing files and recreate
        if ws_dir.exists() {
            std::fs::remove_dir_all(&ws_dir)?;
        }
        std::fs::create_dir_all(&ws_dir)?;

        let mut file_count = 0i32;
        for (path, content) in &files {
            let file_path = ws_dir.join(path.trim_start_matches('/'));
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&file_path, content)?;
            file_count += 1;
        }

        let workspace = crud::update_workspace(
            &state.db,
            id,
            req.name.as_deref(),
            Some(file_count),
        )
        .await?;

        return Ok(Json(workspace));
    }

    // Name-only update
    let workspace = crud::update_workspace(&state.db, id, req.name.as_deref(), None).await?;
    Ok(Json(workspace))
}

pub async fn delete_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    crud::delete_workspace(&state.db, id).await?;

    // Clean up workspace files from disk
    let ws_dir = crate::config::workspaces_data_dir().join(id.to_string());
    if ws_dir.exists() {
        std::fs::remove_dir_all(&ws_dir)?;
    }

    Ok(Json(serde_json::json!({ "deleted": true })))
}
