use crate::error::{FugueError, Result};
use crate::registry::metadata::{DeploymentType, FunctionMetadata};
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct FunctionRegistry {
    functions_dir: PathBuf,
}

impl FunctionRegistry {
    pub fn new(functions_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&functions_dir)?;
        Ok(Self { functions_dir })
    }

    pub fn deploy_function(&self, name: &str, code: &str) -> Result<FunctionMetadata> {
        let metadata = FunctionMetadata::new(name.to_string());
        let function_dir = self.functions_dir.join(name);

        fs::create_dir_all(&function_dir)?;

        // Save metadata
        let metadata_path = function_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        fs::write(metadata_path, metadata_json)?;

        // Save code
        let code_path = function_dir.join("code.js");
        fs::write(code_path, code)?;

        Ok(metadata)
    }

    pub fn get_function(&self, name: &str) -> Result<(FunctionMetadata, String)> {
        let function_dir = self.functions_dir.join(name);

        if !function_dir.exists() {
            return Err(FugueError::FunctionNotFound(name.to_string()));
        }

        // Load metadata
        let metadata_path = function_dir.join("metadata.json");
        let metadata_json = fs::read_to_string(metadata_path)?;
        let metadata: FunctionMetadata = serde_json::from_str(&metadata_json)?;

        // Load code (only for single-file functions)
        let code = match metadata.deployment_type {
            DeploymentType::SingleFile => {
                let code_path = function_dir.join("code.js");
                fs::read_to_string(code_path)?
            }
            DeploymentType::NuxtJs { .. } | DeploymentType::ReactRouter { .. } => {
                // Framework functions don't have a code.js file
                String::new()
            }
        };

        Ok((metadata, code))
    }

    pub fn list_functions(&self) -> Result<Vec<FunctionMetadata>> {
        let mut functions = Vec::new();

        if !self.functions_dir.exists() {
            return Ok(functions);
        }

        for entry in fs::read_dir(&self.functions_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let metadata_path = entry.path().join("metadata.json");
                if metadata_path.exists() {
                    let metadata_json = fs::read_to_string(metadata_path)?;
                    let metadata: FunctionMetadata = serde_json::from_str(&metadata_json)?;
                    functions.push(metadata);
                }
            }
        }

        Ok(functions)
    }

    pub fn delete_function(&self, name: &str) -> Result<()> {
        let function_dir = self.functions_dir.join(name);

        if !function_dir.exists() {
            return Err(FugueError::FunctionNotFound(name.to_string()));
        }

        fs::remove_dir_all(function_dir)?;
        Ok(())
    }

    pub fn deploy_nuxtjs_function(
        &self,
        name: &str,
        source_dir: &Path,
        build_output: &Path,
        env_vars: HashMap<String, String>,
        node_version: String,
    ) -> Result<FunctionMetadata> {
        let function_dir = self.functions_dir.join(name);

        // Create directory structure
        fs::create_dir_all(&function_dir)?;
        let source_dest = function_dir.join("source");
        let build_dest = function_dir.join("build");

        // Copy source directory
        println!("Copying source files...");
        copy_dir_recursive(source_dir, &source_dest)?;

        // Copy build output (.output directory)
        println!("Copying build output...");
        copy_dir_recursive(build_output, &build_dest)?;

        // Create metadata
        let now = Utc::now();
        let metadata = FunctionMetadata {
            id: Uuid::new_v4(),
            name: name.to_string(),
            created_at: now,
            updated_at: now,
            timeout_ms: crate::config::DEFAULT_TIMEOUT_MS,
            handler: "default".to_string(),
            deployment_type: DeploymentType::NuxtJs {
                build_output_path: "build/server".to_string(),
                node_version,
            },
            environment_vars: env_vars,
        };

        // Save metadata
        let metadata_path = function_dir.join("metadata.json");
        fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;

        println!("Deployment complete!");

        Ok(metadata)
    }

    pub fn rebuild_nuxtjs_function(&self, name: &str) -> Result<FunctionMetadata> {
        let function_dir = self.functions_dir.join(name);

        if !function_dir.exists() {
            return Err(FugueError::FunctionNotFound(name.to_string()));
        }

        // Load existing metadata
        let metadata_path = function_dir.join("metadata.json");
        let metadata_json = fs::read_to_string(&metadata_path)?;
        let mut metadata: FunctionMetadata = serde_json::from_str(&metadata_json)?;

        // Verify it's a Nuxt.js deployment
        if !matches!(metadata.deployment_type, DeploymentType::NuxtJs { .. }) {
            return Err(FugueError::ValidationError(
                "Function is not a Nuxt.js deployment".to_string(),
            ));
        }

        // Source directory should exist
        let source_dir = function_dir.join("source");
        if !source_dir.exists() {
            return Err(FugueError::Other(
                "Source directory not found for rebuild".to_string(),
            ));
        }

        // Build the project
        println!("Building Nuxt.js project...");
        let build_result = crate::nuxtjs::build_nuxt_project(&source_dir, false)?;

        // Copy new build output
        let build_dest = function_dir.join("build");
        if build_dest.exists() {
            fs::remove_dir_all(&build_dest)?;
        }
        copy_dir_recursive(&source_dir.join(".output"), &build_dest)?;

        // Update metadata timestamp
        metadata.updated_at = Utc::now();
        fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;

        println!("Rebuild complete!");

        Ok(metadata)
    }

    pub fn deploy_reactrouter_function(
        &self,
        name: &str,
        source_dir: &Path,
        build_output: &Path,
        env_vars: HashMap<String, String>,
        node_version: String,
    ) -> Result<FunctionMetadata> {
        let function_dir = self.functions_dir.join(name);

        // Create directory structure
        fs::create_dir_all(&function_dir)?;
        let source_dest = function_dir.join("source");
        let build_dest = function_dir.join("build");

        // Copy source directory
        println!("Copying source files...");
        copy_dir_recursive(source_dir, &source_dest)?;

        // Copy build output (build/ directory)
        println!("Copying build output...");
        copy_dir_recursive(build_output, &build_dest)?;

        // Create metadata
        let now = Utc::now();
        let metadata = FunctionMetadata {
            id: Uuid::new_v4(),
            name: name.to_string(),
            created_at: now,
            updated_at: now,
            timeout_ms: crate::config::DEFAULT_TIMEOUT_MS,
            handler: "default".to_string(),
            deployment_type: DeploymentType::ReactRouter {
                build_output_path: "build/server".to_string(),
                node_version,
            },
            environment_vars: env_vars,
        };

        // Save metadata
        let metadata_path = function_dir.join("metadata.json");
        fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;

        println!("Deployment complete!");

        Ok(metadata)
    }
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in walkdir::WalkDir::new(src).follow_links(true) {
        let entry = entry.map_err(|e| FugueError::Other(format!("Failed to walk directory: {}", e)))?;
        let path = entry.path();
        let relative_path = path.strip_prefix(src).unwrap();
        let target_path = dst.join(relative_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target_path)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &target_path)?;
        }
        // Skip other file types (symlinks are followed by walkdir)
    }

    Ok(())
}
