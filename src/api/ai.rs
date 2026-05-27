use crate::api::AppState;
use crate::error::FugueError;
use axum::extract::State;
use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub workspace_id: String,
}

#[derive(Debug, Serialize)]
pub struct GenerateEvent {
    pub event_type: String,
    pub data: serde_json::Value,
}

pub async fn generate(
    State(state): State<AppState>,
    axum::Json(req): axum::Json<GenerateRequest>,
) -> Result<impl IntoResponse, crate::api::ApiError> {
    let ai_config = state.config.ai.as_ref().ok_or_else(|| {
        crate::api::ApiError(FugueError::AiNotConfigured(
            "AI is not configured. Set ai.api_key in ~/.fugue/config.toml".to_string(),
        ))
    })?;

    let workspace_id = uuid::Uuid::parse_str(&req.workspace_id).map_err(|e| {
        crate::api::ApiError(FugueError::ValidationError(format!(
            "Invalid workspace_id: {}",
            e
        )))
    })?;

    // Get workspace to determine framework
    let workspace = crate::db::crud::get_workspace(&state.db, workspace_id).await?;
    let framework = workspace.framework.clone();
    let prompt = req.prompt.clone();

    // Build system prompt
    let system_prompt = crate::ai::prompts::get_system_prompt(&framework);
    let system_prompt = system_prompt.replace("{prompt}", &prompt);

    // Create AI client
    let client = crate::ai::AiClient::new(ai_config.clone());

    // Start generation in background, stream results via SSE
    let db = state.db.clone();

    let stream = async_stream::stream! {
        // Generate
        let stream_result = client.generate_stream(&system_prompt, &prompt).await;

        let mut ai_stream = match stream_result {
            Ok(s) => s,
            Err(e) => {
                let event = GenerateEvent {
                    event_type: "error".to_string(),
                    data: serde_json::json!({ "error": e.to_string() }),
                };
                yield Ok::<Event, Infallible>(
                    Event::default().data(serde_json::to_string(&event).unwrap())
                );
                return;
            }
        };

        use futures::StreamExt;

        let mut full_response = String::new();

        while let Some(item) = ai_stream.next().await {
            match item {
                Ok(crate::ai::client::StreamEvent::Token(token)) => {
                    full_response.push_str(&token);
                    let event = GenerateEvent {
                        event_type: "token".to_string(),
                        data: serde_json::json!({ "text": token }),
                    };
                    yield Ok::<Event, Infallible>(
                        Event::default().data(serde_json::to_string(&event).unwrap())
                    );
                }
                Ok(crate::ai::client::StreamEvent::Done) => {
                    break;
                }
                Err(e) => {
                    let event = GenerateEvent {
                        event_type: "error".to_string(),
                        data: serde_json::json!({ "error": e.to_string() }),
                    };
                    yield Ok::<Event, Infallible>(
                        Event::default().data(serde_json::to_string(&event).unwrap())
                    );
                    return;
                }
            }
        }

        // Parse the full response into files
        let files = crate::ai::parser::parse_ai_response(&full_response);

        if files.is_empty() {
            let event = GenerateEvent {
                event_type: "error".to_string(),
                data: serde_json::json!({ "error": "No files were generated from the AI response" }),
            };
            yield Ok::<Event, Infallible>(
                Event::default().data(serde_json::to_string(&event).unwrap())
            );
            return;
        }

        // Validate
        if let Err(e) = crate::ai::parser::validate_project_structure(&files, &framework) {
            tracing::warn!("Validation warning: {}", e);
            // Continue anyway - partial generation is still useful
        }

        // Save files to workspace directory
        let ws_dir = crate::config::workspaces_data_dir().join(workspace_id.to_string());
        if ws_dir.exists() {
            let _ = std::fs::remove_dir_all(&ws_dir);
        }
        if let Err(e) = std::fs::create_dir_all(&ws_dir) {
            let event = GenerateEvent {
                event_type: "error".to_string(),
                data: serde_json::json!({ "error": format!("Failed to create workspace dir: {}", e) }),
            };
            yield Ok::<Event, Infallible>(
                Event::default().data(serde_json::to_string(&event).unwrap())
            );
            return;
        }

        let mut file_count = 0i32;
        for (path, content) in &files {
            let file_path = ws_dir.join(path.trim_start_matches('/'));
            if let Some(parent) = file_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(&file_path, content) {
                tracing::error!("Failed to write file {}: {}", path, e);
            } else {
                file_count += 1;
            }
        }

        // Update workspace file count
        let _ = crate::db::crud::update_workspace(&db, workspace_id, None, Some(file_count)).await;

        // Emit file events for each generated file
        for (path, content) in &files {
            let event = GenerateEvent {
                event_type: "file".to_string(),
                data: serde_json::json!({ "path": path, "content": content }),
            };
            yield Ok::<Event, Infallible>(
                Event::default().data(serde_json::to_string(&event).unwrap())
            );
        }

        // Done event
        let event = GenerateEvent {
            event_type: "done".to_string(),
            data: serde_json::json!({
                "file_count": file_count,
                "files": files,
            }),
        };
        yield Ok::<Event, Infallible>(
            Event::default().data(serde_json::to_string(&event).unwrap())
        );
    };

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    ))
}
