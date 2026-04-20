use crate::client::api::{DeployNextJsRequest, DeployRequest, InvokeRequest, InvokeResponse};
use crate::daemon::state::DaemonState;
use crate::error::{FugueError, Result};
use crate::registry::metadata::DeploymentType;
use crate::validation;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use std::path::PathBuf;
use std::sync::Arc;

pub async fn run_server(state: Arc<DaemonState>) -> Result<()> {
    let app = Router::new()
        .route("/api/deploy", post(deploy_handler))
        .route("/api/deploy-nextjs", post(deploy_nextjs_handler))
        .route("/api/rebuild/:name", post(rebuild_handler))
        .route("/api/invoke/:name", post(invoke_handler))
        .route("/api/url/:name", get(url_handler))
        .route("/api/functions", get(list_handler))
        .route("/api/functions/:name", delete(delete_handler))
        .route("/api/status", get(status_handler))
        .route("/api/shutdown", post(shutdown_handler))
        .with_state(state);

    let addr = format!(
        "{}:{}",
        crate::config::DAEMON_HOST,
        crate::config::DAEMON_PORT
    );

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Daemon listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn deploy_handler(
    State(state): State<Arc<DaemonState>>,
    Json(req): Json<DeployRequest>,
) -> impl IntoResponse {
    let result: Result<_> = async {
        validation::validate_function_name(&req.name)?;
        validation::validate_function_code(&req.code)?;

        let metadata = state.registry.deploy_function(&req.name, &req.code)?;

        let mut functions = state.functions.write().await;
        functions.insert(req.name.clone(), metadata);

        Ok((StatusCode::OK, Json(serde_json::json!({"status": "deployed"}))))
    }
    .await;

    match result {
        Ok(response) => response.into_response(),
        Err(e) => AppError(e).into_response(),
    }
}

async fn invoke_handler(
    State(state): State<Arc<DaemonState>>,
    Path(name): Path<String>,
    Json(req): Json<InvokeRequest>,
) -> impl IntoResponse {
    let result: Result<_> = async {
        // Load function
        let (metadata, code) = state.registry.get_function(&name)?;

        // Handle based on deployment type
        match metadata.deployment_type {
            DeploymentType::SingleFile => {
                // Get or spawn workerd process for single-file function
                let mut pool = state.workerd_pool.write().await;
                let port = pool.get_or_spawn(&name, &code).await?;
                drop(pool);

                // Invoke function via HTTP
                let client = reqwest::Client::builder()
                    .no_proxy()
                    .build()
                    .map_err(|e| FugueError::Other(format!("Failed to build client: {}", e)))?;

                let response = client
                    .post(&format!("http://127.0.0.1:{}/", port))
                    .json(&req.data)
                    .send()
                    .await
                    .map_err(|e| FugueError::ExecutionError(format!("workerd request failed: {}", e)))?;

                let result = response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| FugueError::ExecutionError(format!("Failed to parse response: {}", e)))?;

                Ok(Json(InvokeResponse { result }))
            }
            DeploymentType::NextJs { ref build_output_path, .. } => {
                // Get standalone path
                let function_dir = crate::config::functions_dir().join(&name);
                let standalone_path = function_dir.join(build_output_path);

                tracing::info!("Function dir: {:?}", function_dir);
                tracing::info!("Build output path: {:?}", build_output_path);
                tracing::info!("Looking for standalone at: {:?}", standalone_path);

                if !standalone_path.exists() {
                    let error_msg = format!(
                        "Standalone directory not found at {:?}. The Next.js app may not have been built with output: 'standalone'",
                        standalone_path
                    );
                    tracing::error!("{}", error_msg);
                    return Err(FugueError::ExecutionError(error_msg));
                }

                tracing::info!("Spawning Next.js app at: {:?}", standalone_path);

                // Get or spawn workerd process for Next.js
                let mut pool = state.workerd_pool.write().await;
                let port = match pool
                    .get_or_spawn_nextjs(&name, &standalone_path, &metadata.environment_vars)
                    .await
                {
                    Ok(p) => {
                        tracing::info!("Successfully spawned Next.js on port: {}", p);
                        p
                    }
                    Err(e) => {
                        tracing::error!("Failed to spawn Next.js: {:?}", e);
                        return Err(e);
                    }
                };
                drop(pool);

                tracing::info!("Next.js app running on port: {}", port);

                // Forward HTTP request to Next.js app
                let client = reqwest::Client::builder()
                    .no_proxy()
                    .build()
                    .map_err(|e| FugueError::Other(format!("Failed to build client: {}", e)))?;

                tracing::info!("Sending request to http://127.0.0.1:{}/", port);

                let response = client
                    .get(&format!("http://127.0.0.1:{}/", port))
                    .send()
                    .await
                    .map_err(|e| {
                        tracing::error!("Request to Next.js failed: {}", e);
                        FugueError::ExecutionError(format!("Next.js request failed: {}", e))
                    })?;

                tracing::info!("Got response from Next.js");

                // Get content type before consuming response
                let content_type = response
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string();

                // Try to parse as JSON, if that fails return as text
                let result = if content_type.contains("application/json") {
                    response
                        .json::<serde_json::Value>()
                        .await
                        .map_err(|e| FugueError::ExecutionError(format!("Failed to parse JSON: {}", e)))?
                } else {
                    // Return HTML or other content as text wrapped in JSON
                    let text = response
                        .text()
                        .await
                        .map_err(|e| FugueError::ExecutionError(format!("Failed to read response: {}", e)))?;
                    serde_json::json!({
                        "content_type": content_type,
                        "body": text,
                        "url": format!("http://127.0.0.1:{}/", port)
                    })
                };

                Ok(Json(InvokeResponse { result }))
            }
            DeploymentType::NuxtJs { ref build_output_path, .. } => {
                // Get output path
                let function_dir = crate::config::functions_dir().join(&name);
                let output_path = function_dir.join(build_output_path);

                tracing::info!("Function dir: {:?}", function_dir);
                tracing::info!("Build output path: {:?}", build_output_path);
                tracing::info!("Looking for Nuxt output at: {:?}", output_path);

                if !output_path.exists() {
                    let error_msg = format!(
                        "Output directory not found at {:?}. The Nuxt.js app may not have been built correctly",
                        output_path
                    );
                    tracing::error!("{}", error_msg);
                    return Err(FugueError::ExecutionError(error_msg));
                }

                tracing::info!("Spawning Nuxt.js app at: {:?}", output_path);

                // Get or spawn workerd process for Nuxt.js
                let mut pool = state.workerd_pool.write().await;
                let port = match pool
                    .get_or_spawn_nuxtjs(&name, &output_path, &metadata.environment_vars)
                    .await
                {
                    Ok(p) => {
                        tracing::info!("Successfully spawned Nuxt.js on port: {}", p);
                        p
                    }
                    Err(e) => {
                        tracing::error!("Failed to spawn Nuxt.js: {:?}", e);
                        return Err(e);
                    }
                };
                drop(pool);

                tracing::info!("Nuxt.js app running on port: {}", port);

                // Forward HTTP request to Nuxt.js app
                let client = reqwest::Client::builder()
                    .no_proxy()
                    .build()
                    .map_err(|e| FugueError::Other(format!("Failed to build client: {}", e)))?;

                tracing::info!("Sending request to http://127.0.0.1:{}/", port);

                let response = client
                    .get(&format!("http://127.0.0.1:{}/", port))
                    .send()
                    .await
                    .map_err(|e| {
                        tracing::error!("Request to Nuxt.js failed: {}", e);
                        FugueError::ExecutionError(format!("Nuxt.js request failed: {}", e))
                    })?;

                tracing::info!("Got response from Nuxt.js");

                // Get content type before consuming response
                let content_type = response
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string();

                // Try to parse as JSON, if that fails return as text
                let result = if content_type.contains("application/json") {
                    response
                        .json::<serde_json::Value>()
                        .await
                        .map_err(|e| FugueError::ExecutionError(format!("Failed to parse JSON: {}", e)))?
                } else {
                    // Return HTML or other content as text wrapped in JSON
                    let text = response
                        .text()
                        .await
                        .map_err(|e| FugueError::ExecutionError(format!("Failed to read response: {}", e)))?;
                    serde_json::json!({
                        "content_type": content_type,
                        "body": text,
                        "url": format!("http://127.0.0.1:{}/", port)
                    })
                };

                Ok(Json(InvokeResponse { result }))
            }
        }
    }
    .await;

    match result {
        Ok(response) => response.into_response(),
        Err(e) => AppError(e).into_response(),
    }
}

async fn list_handler(State(state): State<Arc<DaemonState>>) -> impl IntoResponse {
    let result: Result<_> = async {
        let functions = state.registry.list_functions()?;
        Ok(Json(functions))
    }
    .await;

    match result {
        Ok(response) => response.into_response(),
        Err(e) => AppError(e).into_response(),
    }
}

async fn delete_handler(
    State(state): State<Arc<DaemonState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let result: Result<_> = async {
        // Stop the workerd process first to free up the port
        state.workerd_pool.write().await.stop_process(&name).await?;

        state.registry.delete_function(&name)?;

        let mut functions = state.functions.write().await;
        functions.remove(&name);

        Ok(StatusCode::OK)
    }
    .await;

    match result {
        Ok(response) => response.into_response(),
        Err(e) => AppError(e).into_response(),
    }
}

async fn status_handler(State(state): State<Arc<DaemonState>>) -> impl IntoResponse {
    let functions = state.functions.read().await;
    let status = serde_json::json!({
        "status": "running",
        "functions_count": functions.len(),
        "version": env!("CARGO_PKG_VERSION")
    });

    Json(status)
}

async fn shutdown_handler() -> impl IntoResponse {
    tracing::info!("Shutdown requested");
    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        std::process::exit(0);
    });
    StatusCode::OK
}

async fn deploy_nextjs_handler(
    State(state): State<Arc<DaemonState>>,
    Json(req): Json<DeployNextJsRequest>,
) -> impl IntoResponse {
    let result: Result<_> = async {
        validation::validate_function_name(&req.name)?;

        let source_path = PathBuf::from(&req.source_dir);

        // Detect Next.js project
        let project = crate::nextjs::detect_nextjs_project(&source_path)?
            .ok_or_else(|| FugueError::NotNextJsProject("Not a Next.js project".to_string()))?;

        // Validate project
        crate::nextjs::validate_nextjs_project(&project)?;

        // Build project if not skipping
        let next_dir = if req.skip_build {
            source_path.join(".next")
        } else {
            tracing::info!("Building Next.js project: {}", req.name);

            let build_ctx = crate::nextjs::BuildContext {
                source_dir: source_path.clone(),
                build_dir: source_path.join(".next"),
                function_name: req.name.clone(),
            };

            let build_result = crate::nextjs::build_nextjs_project(&build_ctx).await?;
            tracing::info!(
                "Build completed in {}ms, size: {} bytes",
                build_result.build_time_ms,
                build_result.output_size_bytes
            );

            source_path.join(".next")
        };

        // Verify .next directory exists
        if !next_dir.exists() {
            return Err(FugueError::BuildError(
                "Build output not found (.next directory missing)".to_string(),
            ));
        }

        // Deploy to registry
        let metadata = state.registry.deploy_nextjs_function(
            &req.name,
            &source_path,
            &next_dir,
            req.env_vars,
            project.node_version,
        )?;

        let mut functions = state.functions.write().await;
        functions.insert(req.name.clone(), metadata);

        Ok((StatusCode::OK, Json(serde_json::json!({"status": "deployed"}))))
    }
    .await;

    match result {
        Ok(response) => response.into_response(),
        Err(e) => AppError(e).into_response(),
    }
}

async fn rebuild_handler(
    State(state): State<Arc<DaemonState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let result: Result<_> = async {
        // Load function metadata to determine type
        let (metadata, _) = state.registry.get_function(&name)?;

        match metadata.deployment_type {
            DeploymentType::NextJs { .. } => {
                tracing::info!("Rebuilding Next.js app: {}", name);
                let metadata = state.registry.rebuild_nextjs_function(&name)?;
                let mut functions = state.functions.write().await;
                functions.insert(name.clone(), metadata);
            }
            DeploymentType::NuxtJs { .. } => {
                tracing::info!("Rebuilding Nuxt.js app: {}", name);
                let metadata = state.registry.rebuild_nuxtjs_function(&name)?;
                let mut functions = state.functions.write().await;
                functions.insert(name.clone(), metadata);
            }
            DeploymentType::SingleFile => {
                return Err(FugueError::ValidationError(
                    "Cannot rebuild single-file functions".to_string(),
                ));
            }
        }

        Ok((StatusCode::OK, Json(serde_json::json!({"status": "rebuilt"}))))
    }
    .await;

    match result {
        Ok(response) => response.into_response(),
        Err(e) => AppError(e).into_response(),
    }
}

async fn url_handler(
    State(state): State<Arc<DaemonState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let result: Result<_> = async {
        // Check if function exists
        let (metadata, _) = state.registry.get_function(&name)?;

        // Check if workerd process is running
        let pool = state.workerd_pool.read().await;
        let port = pool.get_port(&name);
        drop(pool);

        if let Some(port) = port {
            let url = format!("http://127.0.0.1:{}", port);
            Ok(Json(serde_json::json!({
                "url": url,
                "name": name,
                "deployment_type": format!("{:?}", metadata.deployment_type)
            })))
        } else {
            Ok(Json(serde_json::json!({
                "url": "",
                "name": name,
                "message": "Function not running. Invoke it first to start the workerd process."
            })))
        }
    }
    .await;

    match result {
        Ok(response) => response.into_response(),
        Err(e) => AppError(e).into_response(),
    }
}

// Error wrapper for axum handlers
struct AppError(FugueError);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        // Log the error
        tracing::error!("Handler error: {:?}", self.0);

        let (status, message) = match self.0 {
            FugueError::FunctionNotFound(name) => {
                (StatusCode::NOT_FOUND, format!("Function '{}' not found", name))
            }
            FugueError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
        };

        (status, message).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<FugueError>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
