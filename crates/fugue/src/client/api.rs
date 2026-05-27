use fugue_common::error::{FugueError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResult {
    pub source_path: String,
    pub file_count: u64,
    pub total_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployResult {
    pub build_id: uuid::Uuid,
    pub status: String,
}

pub struct DaemonClient {
    base_url: String,
    client: reqwest::Client,
}

impl DaemonClient {
    pub fn new_with_port(port: u16) -> Self {
        let base_url = format!("http://127.0.0.1:{}", port);

        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("Failed to build HTTP client");

        Self { base_url, client }
    }

    pub async fn status(&self) -> Result<serde_json::Value> {
        let response = self
            .client
            .get(&format!("{}/api/v1/platform/status", self.base_url))
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn create_app(
        &self,
        name: &str,
        framework: &str,
        description: Option<&str>,
    ) -> Result<crate::db::models::App> {
        let mut body = serde_json::json!({
            "name": name,
            "framework": framework,
        });
        if let Some(desc) = description {
            body["description"] = serde_json::json!(desc);
        }

        let response = self
            .client
            .post(&format!("{}/api/v1/apps", self.base_url))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: serde_json::Value = response.json().await?;
            return Err(FugueError::Other(
                error["error"]
                    .as_str()
                    .unwrap_or("Unknown error")
                    .to_string(),
            ));
        }

        Ok(response.json().await?)
    }

    pub async fn list_apps(&self) -> Result<Vec<crate::db::models::App>> {
        let response = self
            .client
            .get(&format!("{}/api/v1/apps", self.base_url))
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn get_app(&self, id: &uuid::Uuid) -> Result<crate::db::models::App> {
        let response = self
            .client
            .get(&format!("{}/api/v1/apps/{}", self.base_url, id))
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn get_app_by_name(&self, name: &str) -> Result<crate::db::models::App> {
        if let Ok(id) = uuid::Uuid::parse_str(name) {
            return self.get_app(&id).await;
        }

        let apps = self.list_apps().await?;
        apps.into_iter()
            .find(|a| a.slug == name || a.name == name)
            .ok_or_else(|| FugueError::AppNotFound(name.to_string()))
    }

    pub async fn delete_app(&self, id: &uuid::Uuid) -> Result<()> {
        self.client
            .delete(&format!("{}/api/v1/apps/{}", self.base_url, id))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn upload_source_file(
        &self,
        id: &uuid::Uuid,
        path: &std::path::Path,
    ) -> Result<UploadResult> {
        let file_content = std::fs::read(path)?;
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(file_content)
                .file_name(file_name),
        );

        let response = self
            .client
            .post(&format!("{}/api/v1/apps/{}/source", self.base_url, id))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: serde_json::Value = response.json().await?;
            return Err(FugueError::Other(
                error["error"]
                    .as_str()
                    .unwrap_or("Unknown error")
                    .to_string(),
            ));
        }

        Ok(response.json().await?)
    }

    pub async fn upload_source_dir(
        &self,
        id: &uuid::Uuid,
        dir: &std::path::Path,
    ) -> Result<UploadResult> {
        let zip_path = std::env::temp_dir().join(format!("fugue-upload-{}.zip", id));
        create_zip_from_dir(dir, &zip_path)?;

        let result = self.upload_source_file(id, &zip_path).await?;

        let _ = std::fs::remove_file(&zip_path);

        Ok(result)
    }

    pub async fn deploy(&self, id: &uuid::Uuid) -> Result<DeployResult> {
        let response = self
            .client
            .post(&format!("{}/api/v1/apps/{}/deploy", self.base_url, id))
            .json(&serde_json::json!({}))
            .send()
            .await?;

        if !response.status().is_success() {
            let error: serde_json::Value = response.json().await?;
            return Err(FugueError::Other(
                error["error"]
                    .as_str()
                    .unwrap_or("Unknown error")
                    .to_string(),
            ));
        }

        Ok(response.json().await?)
    }

    pub async fn start_app(&self, id: &uuid::Uuid) -> Result<serde_json::Value> {
        let response = self
            .client
            .post(&format!("{}/api/v1/apps/{}/start", self.base_url, id))
            .send()
            .await?;

        if !response.status().is_success() {
            let error: serde_json::Value = response.json().await?;
            return Err(FugueError::Other(
                error["error"]
                    .as_str()
                    .unwrap_or("Unknown error")
                    .to_string(),
            ));
        }

        Ok(response.json().await?)
    }

    pub async fn stop_app(&self, id: &uuid::Uuid) -> Result<()> {
        self.client
            .post(&format!("{}/api/v1/apps/{}/stop", self.base_url, id))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

fn create_zip_from_dir(
    dir: &std::path::Path,
    zip_path: &std::path::Path,
) -> Result<()> {
    use zip::write::SimpleFileOptions;

    let zip_file = std::fs::File::create(zip_path)?;
    let mut zip = zip::ZipWriter::new(zip_file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let walker = walkdir::WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "node_modules" && name != ".git" && name != ".output" && name != "build"
        });

    for entry in walker {
        let entry = entry.map_err(|e| {
            FugueError::Other(format!("Failed to walk directory: {}", e))
        })?;

        let path = entry.path();
        let relative = path.strip_prefix(dir).unwrap_or(path);
        let relative_str = relative.to_string_lossy().to_string();

        if path.is_file() {
            zip.start_file(&relative_str, options)
                .map_err(|e| FugueError::Other(format!("Zip error: {}", e)))?;
            let mut file = std::fs::File::open(path)?;
            std::io::copy(&mut file, &mut zip)?;
        } else if path.is_dir() && !relative_str.is_empty() {
            zip.add_directory(relative_str, options)
                .map_err(|e| FugueError::Other(format!("Zip error: {}", e)))?;
        }
    }

    zip.finish()
        .map_err(|e| FugueError::Other(format!("Zip error: {}", e)))?;
    Ok(())
}
