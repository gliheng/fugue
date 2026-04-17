use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeploymentType {
    SingleFile,
    NextJs {
        build_output_path: String,
        node_version: String,
    },
    NuxtJs {
        build_output_path: String,
        node_version: String,
    },
}

impl Default for DeploymentType {
    fn default() -> Self {
        DeploymentType::SingleFile
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub updated_at: DateTime<Utc>,
    pub timeout_ms: u64,
    pub handler: String,
    #[serde(default)]
    pub deployment_type: DeploymentType,
    #[serde(default)]
    pub environment_vars: HashMap<String, String>,
}

impl FunctionMetadata {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            created_at: now,
            updated_at: now,
            timeout_ms: crate::config::DEFAULT_TIMEOUT_MS,
            handler: "handler".to_string(),
            deployment_type: DeploymentType::SingleFile,
            environment_vars: HashMap::new(),
        }
    }
}
