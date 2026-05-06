use crate::error::{FugueError, Result};
use serde_json::Value;
use std::path::Path;

#[derive(Debug)]
pub struct ReactRouterProjectInfo {
    pub node_version: String,
    pub react_router_version: String,
}

pub fn detect_reactrouter_project(project_dir: &Path) -> Result<ReactRouterProjectInfo> {
    let package_json_path = project_dir.join("package.json");
    if !package_json_path.exists() {
        return Err(FugueError::NotReactRouterProject(
            "package.json not found".to_string(),
        ));
    }

    let package_json_content = std::fs::read_to_string(&package_json_path)?;
    let package_json: Value = serde_json::from_str(&package_json_content)?;

    let has_react_router = package_json["dependencies"]["react-router"].is_string()
        || package_json["devDependencies"]["react-router"].is_string();

    if !has_react_router {
        return Err(FugueError::NotReactRouterProject(
            "react-router dependency not found in package.json".to_string(),
        ));
    }

    let react_router_version = package_json["dependencies"]["react-router"]
        .as_str()
        .or_else(|| package_json["devDependencies"]["react-router"].as_str())
        .unwrap_or("unknown")
        .to_string();

    let has_wrangler = project_dir.join("wrangler.jsonc").exists()
        || project_dir.join("wrangler.json").exists();

    if !has_wrangler {
        return Err(FugueError::NotReactRouterProject(
            "wrangler.jsonc or wrangler.json not found".to_string(),
        ));
    }

    let node_version = package_json["engines"]["node"]
        .as_str()
        .unwrap_or(">=18")
        .to_string();

    Ok(ReactRouterProjectInfo {
        node_version,
        react_router_version,
    })
}

pub fn validate_build_output(project_dir: &Path) -> Result<()> {
    let build_dir = project_dir.join("build");
    if !build_dir.exists() {
        return Err(FugueError::BuildError(
            "build directory not found. Did the build succeed?".to_string(),
        ));
    }

    let server_dir = build_dir.join("server");
    if !server_dir.exists() {
        return Err(FugueError::BuildError(
            "build/server directory not found".to_string(),
        ));
    }

    let index_js = server_dir.join("index.js");
    if !index_js.exists() {
        return Err(FugueError::BuildError(
            "build/server/index.js not found".to_string(),
        ));
    }

    Ok(())
}
