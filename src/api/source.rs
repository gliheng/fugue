use crate::api::{ApiError, AppState};
use crate::db::crud;
use axum::{
    extract::{Multipart, Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

pub async fn upload_source(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let _app = crud::get_app(&state.db, id).await?;

    let app_dir = crate::config::apps_data_dir().join(id.to_string());
    let source_dir = app_dir.join("source");
    std::fs::create_dir_all(&source_dir)?;

    let mut file_count = 0u64;
    let mut total_size = 0u64;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError(crate::error::FugueError::Other(format!(
            "Multipart error: {}",
            e
        )))
    })? {
        let _name = field.name().unwrap_or("file").to_string();
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let data = field.bytes().await.map_err(|e| {
            ApiError(crate::error::FugueError::Other(format!(
                "Failed to read field: {}",
                e
            )))
        })?;

        // Handle zip file upload
        if file_name.ends_with(".zip") {
            let zip_path = source_dir.join("upload.zip");
            std::fs::write(&zip_path, &data)?;

            // Extract zip
            let zip_file = std::fs::File::open(&zip_path)?;
            let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| {
                crate::error::FugueError::Other(format!("Failed to open zip: {}", e))
            })?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(|e| {
                    crate::error::FugueError::Other(format!("Failed to read zip entry: {}", e))
                })?;

                let outpath = match file.enclosed_name() {
                    Some(path) => source_dir.join(path),
                    None => continue,
                };

                if file.is_dir() {
                    std::fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(parent) = outpath.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    let mut outfile = std::fs::File::create(&outpath)?;
                    std::io::copy(&mut file, &mut outfile)?;
                    total_size += file.size();
                    file_count += 1;
                }
            }

            // Clean up zip
            std::fs::remove_file(&zip_path)?;
        } else {
            // Individual file upload
            let file_path = source_dir.join(&file_name);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            total_size += data.len() as u64;
            std::fs::write(&file_path, &data)?;
            file_count += 1;
        }
    }

    // Update app source_path
    crud::update_app(
        &state.db,
        id,
        None,
        None,
        None,
        None,
        Some(source_dir.to_str().unwrap_or("")),
        None,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "source_path": source_dir.to_string_lossy(),
        "file_count": file_count,
        "total_size": total_size,
    })))
}

pub async fn get_source(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let app = crud::get_app(&state.db, id).await?;

    let source_path = app
        .source_path
        .ok_or_else(|| {
            ApiError(crate::error::FugueError::ValidationError(
                "No source code uploaded for this app".to_string(),
            ))
        })?;

    let source_dir = std::path::Path::new(&source_path);
    if !source_dir.exists() {
        return Err(ApiError(crate::error::FugueError::ValidationError(
            "Source directory not found".to_string(),
        )));
    }

    let mut files = std::collections::HashMap::new();
    read_source_dir(source_dir, source_dir, &mut files)?;

    Ok(Json(serde_json::json!({ "files": files })))
}

fn read_source_dir(
    base: &std::path::Path,
    dir: &std::path::Path,
    files: &mut std::collections::HashMap<String, String>,
) -> Result<(), ApiError> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip node_modules and .git
            let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
            if dir_name == "node_modules" || dir_name == ".git" {
                continue;
            }
            read_source_dir(base, &path, files)?;
        } else if path.is_file() {
            let relative = path.strip_prefix(base).unwrap_or(&path);
            let key = format!("/{}", relative.display());

            // Skip large binary files
            let metadata = std::fs::metadata(&path)?;
            if metadata.len() > 1_000_000 {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(&path) {
                files.insert(key, content);
            }
        }
    }
    Ok(())
}
