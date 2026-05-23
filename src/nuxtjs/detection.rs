#![allow(dead_code)]

use crate::error::{FugueError, Result};
use serde_json::Value;
use std::path::Path;

#[derive(Debug)]
pub struct NuxtProjectInfo {
    pub node_version: String,
    pub nuxt_version: String,
    pub has_pages: bool,
    pub has_app: bool,
    pub has_server: bool,
}

pub fn detect_nuxt_project(project_dir: &Path) -> Result<NuxtProjectInfo> {
    // Check if package.json exists
    let package_json_path = project_dir.join("package.json");
    if !package_json_path.exists() {
        return Err(FugueError::NotNuxtJsProject(
            "package.json not found".to_string(),
        ));
    }

    // Read and parse package.json
    let package_json_content = std::fs::read_to_string(&package_json_path)?;
    let package_json: Value = serde_json::from_str(&package_json_content)?;

    // Check for nuxt dependency
    let has_nuxt = package_json["dependencies"]["nuxt"].is_string()
        || package_json["devDependencies"]["nuxt"].is_string();

    if !has_nuxt {
        return Err(FugueError::NotNuxtJsProject(
            "nuxt dependency not found in package.json".to_string(),
        ));
    }

    // Get nuxt version
    let nuxt_version = package_json["dependencies"]["nuxt"]
        .as_str()
        .or_else(|| package_json["devDependencies"]["nuxt"].as_str())
        .unwrap_or("unknown")
        .to_string();

    // Validate Nuxt version (must be >= 3.0.0)
    if !is_nuxt_3_or_higher(&nuxt_version) {
        return Err(FugueError::UnsupportedNuxtJsVersion(format!(
            "Nuxt version {} is not supported. Only Nuxt 3.x is supported.",
            nuxt_version
        )));
    }

    // Check for nuxt.config file
    let has_config = project_dir.join("nuxt.config.ts").exists()
        || project_dir.join("nuxt.config.js").exists()
        || project_dir.join("nuxt.config.mjs").exists();

    if !has_config {
        return Err(FugueError::NotNuxtJsProject(
            "nuxt.config.ts/js/mjs not found".to_string(),
        ));
    }

    // Check for typical Nuxt directories
    let has_pages = project_dir.join("pages").exists();
    let has_app = project_dir.join("app").exists();
    let has_server = project_dir.join("server").exists();

    // Get Node.js version from package.json engines
    let node_version = package_json["engines"]["node"]
        .as_str()
        .unwrap_or(">=18")
        .to_string();

    Ok(NuxtProjectInfo {
        node_version,
        nuxt_version,
        has_pages,
        has_app,
        has_server,
    })
}

fn is_nuxt_3_or_higher(version: &str) -> bool {
    // Remove common prefixes like ^, ~, >=
    let version = version
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches(">=")
        .trim_start_matches('>')
        .trim();

    // Extract major version
    if let Some(major_str) = version.split('.').next() {
        if let Ok(major) = major_str.parse::<u32>() {
            return major >= 3;
        }
    }

    // If we can't parse, assume it's valid (could be "latest", "next", etc.)
    true
}

pub fn validate_build_output(project_dir: &Path) -> Result<()> {
    let output_dir = project_dir.join(".output");
    if !output_dir.exists() {
        return Err(FugueError::BuildError(
            ".output directory not found. Did the build succeed?".to_string(),
        ));
    }

    let server_dir = output_dir.join("server");
    if !server_dir.exists() {
        return Err(FugueError::BuildError(
            ".output/server directory not found. Ensure Nitro is configured correctly.".to_string(),
        ));
    }

    let index_mjs = server_dir.join("index.mjs");
    if !index_mjs.exists() {
        return Err(FugueError::BuildError(
            ".output/server/index.mjs not found. This is the Nitro server entry point.".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_nuxt_3_or_higher() {
        assert!(is_nuxt_3_or_higher("3.0.0"));
        assert!(is_nuxt_3_or_higher("^3.0.0"));
        assert!(is_nuxt_3_or_higher("~3.5.1"));
        assert!(is_nuxt_3_or_higher(">=3.0.0"));
        assert!(is_nuxt_3_or_higher("4.0.0"));
        assert!(!is_nuxt_3_or_higher("2.17.0"));
        assert!(!is_nuxt_3_or_higher("^2.15.0"));
    }
}
