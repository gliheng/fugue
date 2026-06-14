#![allow(dead_code)]

use fugue_common::error::{FugueError, Result};
use fugue_common::project_config::ProjectConfig;
use serde_json::Value;
use std::path::Path;

#[derive(Debug)]
pub struct ViteProjectInfo {
    pub node_version: String,
    pub vite_version: String,
}

pub fn detect_vite_project(project_dir: &Path) -> Result<ViteProjectInfo> {
    let package_json_path = project_dir.join("package.json");
    if !package_json_path.exists() {
        return Err(FugueError::NotViteProject(
            "package.json not found".to_string(),
        ));
    }

    let package_json_content = std::fs::read_to_string(&package_json_path)?;
    let package_json: Value = serde_json::from_str(&package_json_content)?;

    let has_vite = package_json["dependencies"]["vite"].is_string()
        || package_json["devDependencies"]["vite"].is_string();

    if !has_vite {
        return Err(FugueError::NotViteProject(
            "vite dependency not found in package.json".to_string(),
        ));
    }

    let has_cloudflare_vite_plugin = package_json["dependencies"]["@cloudflare/vite-plugin"]
        .is_string()
        || package_json["devDependencies"]["@cloudflare/vite-plugin"]
            .is_string();

    if !has_cloudflare_vite_plugin {
        return Err(FugueError::NotViteProject(
            "@cloudflare/vite-plugin not found in package.json".to_string(),
        ));
    }

    let vite_version = package_json["dependencies"]["vite"]
        .as_str()
        .or_else(|| package_json["devDependencies"]["vite"].as_str())
        .unwrap_or("unknown")
        .to_string();

    let has_wrangler = project_dir.join("wrangler.jsonc").exists()
        || project_dir.join("wrangler.json").exists();

    if !has_wrangler {
        return Err(FugueError::NotViteProject(
            "wrangler.jsonc or wrangler.json not found".to_string(),
        ));
    }

    let node_version = package_json["engines"]["node"]
        .as_str()
        .unwrap_or(">=18")
        .to_string();

    Ok(ViteProjectInfo {
        node_version,
        vite_version,
    })
}

pub fn validate_build_output(project_dir: &Path) -> Result<()> {
    let config = ProjectConfig::load(project_dir, "vite")?;
    let output_dir = project_dir.join(&config.build_output_dir);

    if !output_dir.exists() {
        return Err(FugueError::BuildError(format!(
            "{} directory not found. Did the build succeed?",
            config.build_output_dir
        )));
    }

    let worker_dir = output_dir.join("vite_app");
    if !worker_dir.exists() {
        return Err(FugueError::BuildError(format!(
            "{}/vite_app directory not found",
            config.build_output_dir
        )));
    }

    let server_entry = config
        .server_entry
        .as_deref()
        .unwrap_or("vite_app/index.js");
    let index_file = output_dir.join(server_entry);
    if !index_file.exists() {
        return Err(FugueError::BuildError(format!(
            "{}/{} not found",
            config.build_output_dir, server_entry
        )));
    }

    let client_dir = project_dir.join(&config.assets_dir);
    if !client_dir.exists() {
        return Err(FugueError::BuildError(format!(
            "{} directory not found",
            config.assets_dir
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_detect_vite_project() {
        let dir = std::env::temp_dir().join("fugue-test-vite-detect");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            dir.join("package.json"),
            r#"{
                "dependencies": { "react": "^19.0.0" },
                "devDependencies": { "vite": "^8.0.0", "@cloudflare/vite-plugin": "^1.0.0" }
            }"#,
        )
        .unwrap();
        fs::write(dir.join("wrangler.jsonc"), "{}").unwrap();

        let info = detect_vite_project(&dir).unwrap();
        assert_eq!(info.vite_version, "^8.0.0");
        assert_eq!(info.node_version, ">=18");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_missing_vite() {
        let dir = std::env::temp_dir().join("fugue-test-vite-missing-vite");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            dir.join("package.json"),
            r#"{ "dependencies": { "react": "^19.0.0" }, "devDependencies": { "@cloudflare/vite-plugin": "^1.0.0" } }"#,
        )
        .unwrap();
        fs::write(dir.join("wrangler.jsonc"), "{}").unwrap();

        assert!(detect_vite_project(&dir).is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_vue_project() {
        let dir = std::env::temp_dir().join("fugue-test-vite-vue");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            dir.join("package.json"),
            r#"{
                "dependencies": { "vue": "^3.0.0" },
                "devDependencies": { "vite": "^8.0.0", "@cloudflare/vite-plugin": "^1.0.0" }
            }"#,
        )
        .unwrap();
        fs::write(dir.join("wrangler.jsonc"), "{}").unwrap();

        let info = detect_vite_project(&dir).unwrap();
        assert_eq!(info.vite_version, "^8.0.0");

        let _ = fs::remove_dir_all(&dir);
    }
}
