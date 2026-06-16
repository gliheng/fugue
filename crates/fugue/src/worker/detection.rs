#![allow(dead_code)]

use fugue_common::error::{FugueError, Result};
use std::path::Path;

#[derive(Debug)]
pub struct WorkerProjectInfo {
    pub has_src_dir: bool,
    pub entry_point: String,
}

pub fn detect_worker_project(project_dir: &Path) -> Result<WorkerProjectInfo> {
    let package_json_path = project_dir.join("package.json");
    if !package_json_path.exists() {
        return Err(FugueError::ValidationError(
            "package.json not found. Is this a Cloudflare Worker project?".to_string(),
        ));
    }

    let has_src_dir = project_dir.join("src").is_dir();

    let entry_point = if has_src_dir {
        let candidates = ["index.ts", "index.js", "index.mjs"];
        let mut found = None;
        for candidate in &candidates {
            let path = project_dir.join("src").join(candidate);
            if path.exists() {
                found = Some(format!("src/{}", candidate));
                break;
            }
        }
        found.unwrap_or_else(|| "src/index.ts".to_string())
    } else {
        let candidates = [
            "index.ts",
            "index.js",
            "index.mjs",
            "worker.ts",
            "worker.js",
        ];
        let mut found = None;
        for candidate in &candidates {
            let path = project_dir.join(candidate);
            if path.exists() {
                found = Some(candidate.to_string());
                break;
            }
        }
        found.unwrap_or_else(|| "index.ts".to_string())
    };

    Ok(WorkerProjectInfo {
        has_src_dir,
        entry_point,
    })
}
