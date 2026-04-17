use crate::error::{FugueError, Result};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct NextJsProject {
    pub root_dir: PathBuf,
    pub has_app_dir: bool,
    pub has_pages_dir: bool,
    pub next_version: String,
    pub node_version: String,
}

/// Detect if a directory contains a Next.js project
pub fn detect_nextjs_project(dir: &Path) -> Result<Option<NextJsProject>> {
    let dir = dir.canonicalize()?;

    // Check for package.json
    let package_json_path = dir.join("package.json");
    if !package_json_path.exists() {
        return Ok(None);
    }

    // Parse package.json
    let package_json_content = fs::read_to_string(&package_json_path)?;
    let package_json: Value = serde_json::from_str(&package_json_content)
        .map_err(|e| FugueError::NotNextJsProject(format!("Invalid package.json: {}", e)))?;

    // Check for Next.js dependency
    let next_version = package_json
        .get("dependencies")
        .and_then(|deps| deps.get("next"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            package_json
                .get("devDependencies")
                .and_then(|deps| deps.get("next"))
                .and_then(|v| v.as_str())
        });

    let next_version = match next_version {
        Some(v) => v.to_string(),
        None => return Ok(None),
    };

    // Check for app or pages directory
    let has_app_dir = dir.join("app").exists() || dir.join("src/app").exists();
    let has_pages_dir = dir.join("pages").exists() || dir.join("src/pages").exists();

    // Get Node.js version (default to 18 if not specified)
    let node_version = package_json
        .get("engines")
        .and_then(|e| e.get("node"))
        .and_then(|v| v.as_str())
        .unwrap_or("18")
        .to_string();

    Ok(Some(NextJsProject {
        root_dir: dir,
        has_app_dir,
        has_pages_dir,
        next_version,
        node_version,
    }))
}

/// Validate that a Next.js project is compatible with Fugue
pub fn validate_nextjs_project(project: &NextJsProject) -> Result<()> {
    // Check Next.js version (require >= 13 for standalone output)
    let version_str = project.next_version.trim_start_matches('^').trim_start_matches('~');
    let major_version = version_str
        .split('.')
        .next()
        .and_then(|v| v.parse::<u32>().ok())
        .ok_or_else(|| {
            FugueError::UnsupportedNextJsVersion(format!(
                "Could not parse version: {}",
                project.next_version
            ))
        })?;

    if major_version < 13 {
        return Err(FugueError::UnsupportedNextJsVersion(format!(
            "Next.js version {} is not supported. Minimum version is 13.0.0",
            project.next_version
        )));
    }

    // Check for app or pages directory
    if !project.has_app_dir && !project.has_pages_dir {
        return Err(FugueError::NotNextJsProject(
            "No app/ or pages/ directory found".to_string(),
        ));
    }

    // Check for next.config.js or next.config.mjs
    let has_config = project.root_dir.join("next.config.js").exists()
        || project.root_dir.join("next.config.mjs").exists()
        || project.root_dir.join("next.config.ts").exists();

    if !has_config {
        return Err(FugueError::NotNextJsProject(
            "No next.config.js found. Please create one with output: 'standalone'".to_string(),
        ));
    }

    Ok(())
}
