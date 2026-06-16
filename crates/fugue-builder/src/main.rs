mod artifacts;
mod runner;

use fugue_common::models::{BuildLog, BuildResult, BuildTask, Framework, LogStream};
use futures_util::StreamExt;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

    info!("Connecting to NATS at {}", nats_url);
    let client = async_nats::connect(&nats_url).await?;
    info!("Connected to NATS");

    // Subscribe with queue group for load balancing
    let mut subscriber = client
        .queue_subscribe("fugue.build.requests", "fugue-builders".to_string())
        .await?;
    info!("Listening on fugue.build.requests [queue: fugue-builders]");

    while let Some(message) = subscriber.next().await {
        let task: BuildTask = match serde_json::from_slice(&message.payload) {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to deserialize build task: {}", e);
                continue;
            }
        };

        info!(
            "Received build task: {} for app {}",
            task.build_id, task.app_id
        );

        let client = client.clone();
        tokio::spawn(async move {
            if let Err(e) = execute_build(&client, task).await {
                error!("Build execution failed: {}", e);
            }
        });
    }

    Ok(())
}

async fn execute_build(client: &async_nats::Client, task: BuildTask) -> anyhow::Result<()> {
    publish_log(client, &task.build_id, "Build started", LogStream::System).await;

    let result = match task.framework {
        Framework::Worker => runner::build_worker(&task).await,
        Framework::Hono => runner::build_hono(&task).await,
        Framework::NuxtJs => runner::build_nuxtjs(&task).await,
        Framework::ReactRouter => runner::build_reactrouter(&task).await,
        Framework::Vite => runner::build_vite(&task).await,
    };

    let build_result = match result {
        Ok((output_size, build_time_ms)) => {
            publish_log(client, &task.build_id, "Build succeeded", LogStream::System).await;

            match artifacts::generate_artifacts(&task) {
                Ok(artifacts_path) => {
                    publish_log(
                        client,
                        &task.build_id,
                        "Artifacts generated",
                        LogStream::System,
                    )
                    .await;
                    BuildResult {
                        build_id: task.build_id,
                        app_id: task.app_id,
                        success: true,
                        output_size,
                        build_time_ms,
                        error: None,
                        artifacts_path: Some(artifacts_path),
                    }
                }
                Err(e) => {
                    let error_msg = format!("Artifact generation failed: {}", e);
                    publish_log(client, &task.build_id, &error_msg, LogStream::Stderr).await;
                    BuildResult {
                        build_id: task.build_id,
                        app_id: task.app_id,
                        success: false,
                        output_size: 0,
                        build_time_ms: 0,
                        error: Some(error_msg),
                        artifacts_path: None,
                    }
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Build failed: {}", e);
            publish_log(client, &task.build_id, &error_msg, LogStream::Stderr).await;
            BuildResult {
                build_id: task.build_id,
                app_id: task.app_id,
                success: false,
                output_size: 0,
                build_time_ms: 0,
                error: Some(e.to_string()),
                artifacts_path: None,
            }
        }
    };

    let subject = format!("fugue.build.results.{}", task.build_id);
    let payload = serde_json::to_vec(&build_result)?;
    client.publish(subject, payload.into()).await?;

    info!("Published build result for {}", task.build_id);

    Ok(())
}

async fn publish_log(
    client: &async_nats::Client,
    build_id: &uuid::Uuid,
    line: &str,
    stream: LogStream,
) {
    let log = BuildLog {
        build_id: *build_id,
        line: line.to_string(),
        stream,
    };
    let subject = format!("fugue.build.logs.{}", build_id);
    if let Err(e) = client
        .publish(subject, serde_json::to_vec(&log).unwrap().into())
        .await
    {
        error!("Failed to publish log: {}", e);
    }
}
