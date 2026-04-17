use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployRequest {
    pub name: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployNextJsRequest {
    pub name: String,
    pub source_dir: String,
    pub skip_build: bool,
    pub env_vars: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RebuildRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvokeRequest {
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvokeResponse {
    pub result: serde_json::Value,
}

pub struct DaemonClient {
    base_url: String,
    client: reqwest::Client,
}

impl DaemonClient {
    pub fn new() -> Self {
        let base_url = format!(
            "http://{}:{}",
            crate::config::DAEMON_HOST,
            crate::config::DAEMON_PORT
        );

        // Build client without proxy
        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("Failed to build HTTP client");

        Self {
            base_url,
            client,
        }
    }

    pub async fn deploy(&self, name: &str, code: &str) -> Result<()> {
        let request = DeployRequest {
            name: name.to_string(),
            code: code.to_string(),
        };

        self.client
            .post(&format!("{}/api/deploy", self.base_url))
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn invoke(&self, name: &str, data: serde_json::Value) -> Result<serde_json::Value> {
        let request = InvokeRequest { data };

        let response = self
            .client
            .post(&format!("{}/api/invoke/{}", self.base_url, name))
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        let invoke_response: InvokeResponse = response.json().await?;
        Ok(invoke_response.result)
    }

    pub async fn list(&self) -> Result<Vec<crate::registry::FunctionMetadata>> {
        let response = self
            .client
            .get(&format!("{}/api/functions", self.base_url))
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn delete(&self, name: &str) -> Result<()> {
        self.client
            .delete(&format!("{}/api/functions/{}", self.base_url, name))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn status(&self) -> Result<serde_json::Value> {
        let response = self
            .client
            .get(&format!("{}/api/status", self.base_url))
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.client
            .post(&format!("{}/api/shutdown", self.base_url))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn deploy_nextjs(
        &self,
        name: &str,
        source_dir: &str,
        skip_build: bool,
        env_vars: HashMap<String, String>,
    ) -> Result<()> {
        let request = DeployNextJsRequest {
            name: name.to_string(),
            source_dir: source_dir.to_string(),
            skip_build,
            env_vars,
        };

        self.client
            .post(&format!("{}/api/deploy-nextjs", self.base_url))
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn rebuild(&self, name: &str) -> Result<()> {
        let request = RebuildRequest {
            name: name.to_string(),
        };

        self.client
            .post(&format!("{}/api/rebuild/{}", self.base_url, name))
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn get_url(&self, name: &str) -> Result<String> {
        let response = self
            .client
            .get(&format!("{}/api/url/{}", self.base_url, name))
            .send()
            .await?
            .error_for_status()?;

        let result: serde_json::Value = response.json().await?;
        Ok(result["url"].as_str().unwrap_or("").to_string())
    }
}
