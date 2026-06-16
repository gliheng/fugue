use crate::api::{ApiError, AppState};
use crate::db::crud;
use crate::db::models;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct DeployWorkspaceRequest {
    pub app_id: Uuid,
}

pub async fn list_workspaces(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
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
    // Vite template aliases share the same runtime framework. Everything else must be a
    // known framework enum value.
    let (framework, template_id) = match req.framework.as_str() {
        "vite-react" | "vite-vue" => ("vite", req.framework.as_str()),
        other => {
            if models::Framework::from_str(other).is_none() {
                return Err(ApiError(fugue_common::error::FugueError::ValidationError(format!(
                    "Invalid framework '{}'. Must be one of: worker, nuxtjs, react-router, vite, vite-react, vite-vue, hono",
                    req.framework
                ))));
            }
            (other, other)
        }
    };

    let name = match req.name {
        Some(name) => name,
        None => crud::generate_unique_workspace_name(&state.db).await?,
    };

    // Create workspace record first (file_count = 0)
    let workspace = crud::create_workspace(&state.db, &name, framework, 0).await?;

    // Write template files to disk
    let ws_dir = crate::config::workspaces_data_dir().join(workspace.id.to_string());
    std::fs::create_dir_all(&ws_dir)?;

    let template_files = crate::templates::get_template_files(template_id)
        .map_err(|e| fugue_common::error::FugueError::ValidationError(e))?;

    let mut file_count = 0i32;
    for (path, content) in &template_files {
        let file_path = ws_dir.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file_path, content)?;
        file_count += 1;
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
                fugue_common::error::FugueError::ValidationError(format!(
                    "Invalid files format: {}",
                    e
                ))
            })?;

        let ws_dir = crate::config::workspaces_data_dir().join(id.to_string());
        std::fs::create_dir_all(&ws_dir)?;

        // Update only the files provided in the request. Files not included
        // (e.g. binary assets like images) are left untouched on disk.
        for (path, content) in &files {
            let file_path = ws_dir.join(path.trim_start_matches('/'));
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&file_path, content)?;
        }

        // Recalculate file count from disk so binary assets are accounted for.
        let file_count = count_workspace_files(&ws_dir)?;

        let workspace =
            crud::update_workspace(&state.db, id, req.name.as_deref(), Some(file_count)).await?;

        return Ok(Json(workspace));
    }

    // Name-only update
    let workspace = crud::update_workspace(&state.db, id, req.name.as_deref(), None).await?;
    Ok(Json(workspace))
}

pub async fn deploy_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeployWorkspaceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let _workspace = crud::get_workspace(&state.db, id).await?;
    let app = crud::get_app(&state.db, req.app_id).await?;

    let ws_dir = crate::config::workspaces_data_dir().join(id.to_string());
    if !ws_dir.exists() {
        return Err(ApiError(fugue_common::error::FugueError::ValidationError(
            "Workspace source directory not found".to_string(),
        )));
    }

    let app_source_dir = crate::config::apps_data_dir()
        .join(app.id.to_string())
        .join("source");
    std::fs::create_dir_all(&app_source_dir)?;

    // Copy the full workspace directory (including binary assets) to the app's source directory.
    copy_dir_recursive(&ws_dir, &app_source_dir)?;

    // Update the app's source_path to point to the copied directory.
    crud::update_app(
        &state.db,
        app.id,
        None,
        None,
        None,
        None,
        Some(app_source_dir.to_str().unwrap_or("")),
        None,
    )
    .await?;

    let build_id = crate::api::deploy::trigger_deploy(&state, &app, app_source_dir).await?;

    Ok(Json(serde_json::json!({
        "build_id": build_id,
        "status": "building",
    })))
}

fn count_workspace_files(dir: &std::path::Path) -> Result<i32, fugue_common::error::FugueError> {
    let mut count = 0i32;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            count += count_workspace_files(&path)?;
        } else {
            count += 1;
        }
    }
    Ok(count)
}

fn copy_dir_recursive(
    src: &std::path::Path,
    dst: &std::path::Path,
) -> Result<(), fugue_common::error::FugueError> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let dest = dst.join(&name);
        if path.is_dir() {
            copy_dir_recursive(&path, &dest)?;
        } else {
            std::fs::copy(&path, &dest)?;
        }
    }
    Ok(())
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
