use crate::api::{ApiError, AppState};
use crate::db::{crud, models};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub framework: Option<String>,
}

pub async fn create_app(
    State(state): State<AppState>,
    Json(req): Json<models::CreateAppRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if req.name.is_empty() {
        return Err(ApiError(crate::error::FugueError::ValidationError(
            "App name cannot be empty".to_string(),
        )));
    }

    if models::Framework::from_str(&req.framework).is_none() {
        return Err(ApiError(crate::error::FugueError::ValidationError(format!(
            "Invalid framework '{}'. Must be one of: worker, nuxtjs, react-router",
            req.framework
        ))));
    }

    let mut app = crud::create_app(&state.db, &req.name, &req.framework, req.description.as_deref())
        .await?;

    match crate::templates::populate_template_source(&app.id, &req.framework) {
        Ok(source_dir) => {
            if let Some(source_str) = source_dir.to_str() {
                app = crud::update_app(
                    &state.db,
                    app.id,
                    None,
                    None,
                    None,
                    None,
                    Some(source_str),
                    None,
                )
                .await?;
            }
        }
        Err(e) => {
            tracing::warn!("Failed to populate template source for app '{}': {}", app.id, e);
        }
    }

    Ok((axum::http::StatusCode::CREATED, Json(app)))
}

pub async fn list_apps(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let apps = crud::list_apps(
        &state.db,
        query.status.as_deref(),
        query.framework.as_deref(),
    )
    .await?;

    Ok(Json(apps))
}

pub async fn get_app(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let app = crud::get_app(&state.db, id).await?;
    Ok(Json(app))
}

pub async fn update_app(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<models::UpdateAppRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let env_vars_ref = req.env_vars.as_ref();
    let app = crud::update_app(
        &state.db,
        id,
        req.name.as_deref(),
        req.description.as_deref(),
        env_vars_ref,
        None,
        None,
        None,
    )
    .await?;

    Ok(Json(app))
}

pub async fn delete_app(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    // Stop the app's workerd services if running
    let app = crud::get_app(&state.db, id).await?;

    if app.status == "running" {
        let mut pm = state.process.write().await;
        // Regenerate config without this app
        let remaining_apps: Vec<_> = crud::list_apps(&state.db, None, None)
            .await?
            .into_iter()
            .filter(|a| a.id != id)
            .collect();
        pm.reload(&remaining_apps).await?;
    }

    crud::delete_app(&state.db, id).await?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}
