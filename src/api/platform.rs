use crate::api::AppState;
use crate::db::crud;
use axum::{extract::State, response::IntoResponse, Json};

pub async fn platform_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let (total, running) = crud::count_apps(&state.db).await.unwrap_or((0, 0));

    let pm = state.process.read().await;
    let workerd_running = pm.is_running();

    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "apps_total": total,
        "apps_running": running,
        "workerd_running": workerd_running,
        "domain": state.config.platform.domain,
        "port": state.config.platform.port,
        "workerd_port": state.config.workerd.port,
    }))
}
