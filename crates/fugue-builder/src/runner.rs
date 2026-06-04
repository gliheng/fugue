use fugue_common::builder::build_project;
use fugue_common::error::Result;
use fugue_common::models::BuildTask;
use tracing::info;

pub async fn build_worker(task: &BuildTask) -> Result<(u64, u128)> {
    build_and_report(task, "worker").await
}

pub async fn build_nuxtjs(task: &BuildTask) -> Result<(u64, u128)> {
    build_and_report(task, "nuxtjs").await
}

pub async fn build_reactrouter(task: &BuildTask) -> Result<(u64, u128)> {
    build_and_report(task, "react-router").await
}

async fn build_and_report(task: &BuildTask, framework: &str) -> Result<(u64, u128)> {
    let result = build_project(&task.source_path,
        framework,
        task.skip_install,
    )?;

    info!(
        "{} build completed in {}ms, output size: {} bytes",
        framework,
        result.build_time_ms,
        result.output_size
    );

    Ok((result.output_size, result.build_time_ms))
}
