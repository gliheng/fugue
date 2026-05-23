pub mod apps;
pub mod deploy;
pub mod platform;
pub mod runtime;
pub mod source;

use crate::config::PlatformConfig;
use crate::process::ProcessManager;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub process: Arc<RwLock<ProcessManager>>,
    pub config: PlatformConfig,
}

pub fn api_router(state: AppState) -> Router {
    Router::new()
        // App management
        .route("/api/v1/apps", post(apps::create_app))
        .route("/api/v1/apps", get(apps::list_apps))
        .route("/api/v1/apps/:id", get(apps::get_app))
        .route("/api/v1/apps/:id", patch(apps::update_app))
        .route("/api/v1/apps/:id", delete(apps::delete_app))
        // Source code
        .route("/api/v1/apps/:id/source", post(source::upload_source))
        .route("/api/v1/apps/:id/source", get(source::get_source))
        // Build & deploy
        .route("/api/v1/apps/:id/deploy", post(deploy::deploy_app))
        .route("/api/v1/apps/:id/redeploy", post(deploy::redeploy_app))
        .route("/api/v1/apps/:id/builds", get(deploy::list_builds))
        .route("/api/v1/apps/:id/builds/:build_id", get(deploy::get_build))
        // Runtime
        .route("/api/v1/apps/:id/start", post(runtime::start_app))
        .route("/api/v1/apps/:id/stop", post(runtime::stop_app))
        .route("/api/v1/apps/:id/status", get(runtime::app_status))
        // Platform
        .route("/api/v1/platform/status", get(platform::platform_status))
        .with_state(state)
}

pub struct ApiError(pub crate::error::FugueError);

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("API error: {:?}", self.0);

        let (status, message) = match &self.0 {
            crate::error::FugueError::AppNotFound(_)
            | crate::error::FugueError::BuildNotFound(_) => {
                (StatusCode::NOT_FOUND, self.0.to_string())
            }
            crate::error::FugueError::AppAlreadyExists(_) => {
                (StatusCode::CONFLICT, self.0.to_string())
            }
            crate::error::FugueError::ValidationError(_) => {
                (StatusCode::BAD_REQUEST, self.0.to_string())
            }
            crate::error::FugueError::AppNotRunning(_) => {
                (StatusCode::BAD_REQUEST, self.0.to_string())
            }
            crate::error::FugueError::AppAlreadyRunning(_) => {
                (StatusCode::CONFLICT, self.0.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

impl<E> From<E> for ApiError
where
    E: Into<crate::error::FugueError>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
