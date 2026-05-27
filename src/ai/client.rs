use crate::config::AiConfig;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub stream: bool,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub delta: Option<ChatDelta>,
    pub message: Option<ChatMessage>,
}

#[derive(Debug, Deserialize)]
pub struct ChatDelta {
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug)]
pub enum StreamEvent {
    Token(String),
    Done,
}

#[derive(Clone)]
pub struct AiClient {
    config: AiConfig,
    client: reqwest::Client,
}

impl AiClient {
    pub fn new(config: AiConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    pub async fn generate_stream(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> crate::error::Result<Pin<Box<dyn Stream<Item = crate::error::Result<StreamEvent>> + Send>>>
    {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ];

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            stream: true,
        };

        let url = format!("{}/chat/completions", self.config.api_base.trim_end_matches('/'));

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Accept", "text/event-stream")
            .json(&request)
            .send()
            .await
            .map_err(|e| crate::error::FugueError::AiError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(crate::error::FugueError::AiError(format!(
                "API error {}: {}",
                status, body
            )));
        }

        let byte_stream = response.bytes_stream();

        let stream = futures::stream::unfold(
            (byte_stream, String::new(), false),
            |(mut byte_stream, mut buffer, done)| async move {
                use futures::TryStreamExt;
                if done {
                    return None;
                }

                loop {
                    // Try to parse a complete SSE line from buffer
                    if let Some(line_end) = buffer.find('\n') {
                        let line = buffer[..line_end].trim().to_string();
                        buffer = buffer[line_end + 1..].to_string();

                        if line.is_empty() || line.starts_with(':') {
                            continue;
                        }

                        if line == "data: [DONE]" {
                            return Some((Ok(StreamEvent::Done), (byte_stream, buffer, true)));
                        }

                        if let Some(data) = line.strip_prefix("data: ") {
                            match serde_json::from_str::<ChatResponse>(data) {
                                Ok(resp) => {
                                    if let Some(choice) = resp.choices.first() {
                                        if let Some(delta) = &choice.delta {
                                            if let Some(content) = &delta.content {
                                                if !content.is_empty() {
                                                    return Some((
                                                        Ok(StreamEvent::Token(content.clone())),
                                                        (byte_stream, buffer, false),
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                    // No content in this chunk, continue reading
                                    continue;
                                }
                                Err(_) => {
                                    // Skip malformed JSON
                                    continue;
                                }
                            }
                        }
                        // Not a data line, skip
                        continue;
                    }

                    // Need more data from the stream
                    match byte_stream.try_next().await {
                        Ok(Some(chunk)) => {
                            let text = String::from_utf8_lossy(&chunk);
                            buffer.push_str(&text);
                        }
                        Ok(None) => {
                            // Stream ended
                            return Some((
                                Ok(StreamEvent::Done),
                                (byte_stream, buffer, true),
                            ));
                        }
                        Err(e) => {
                            return Some((
                                Err(crate::error::FugueError::AiError(format!(
                                    "Stream error: {}",
                                    e
                                ))),
                                (byte_stream, buffer, true),
                            ));
                        }
                    }
                }
            },
        );

        Ok(Box::pin(stream))
    }
}
