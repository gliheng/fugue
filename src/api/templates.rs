use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct TemplateInfo {
    id: String,
    name: String,
    framework: String,
    description: String,
}

#[derive(Serialize)]
pub struct TemplateDetail {
    id: String,
    name: String,
    framework: String,
    description: String,
    files: std::collections::HashMap<String, String>,
}

pub async fn list_templates() -> impl IntoResponse {
    let templates = vec![
        TemplateInfo {
            id: "worker".to_string(),
            name: "Worker".to_string(),
            framework: "worker".to_string(),
            description: "Simple Cloudflare Worker with a fetch handler".to_string(),
        },
        TemplateInfo {
            id: "nuxtjs".to_string(),
            name: "Nuxt.js".to_string(),
            framework: "nuxtjs".to_string(),
            description: "Full-stack Nuxt.js application with SSR".to_string(),
        },
        TemplateInfo {
            id: "react-router".to_string(),
            name: "React Router".to_string(),
            framework: "react-router".to_string(),
            description: "React Router v7 application with SSR".to_string(),
        },
    ];
    Json(templates)
}

pub async fn get_template(
    axum::extract::Path(framework): axum::extract::Path<String>,
) -> impl IntoResponse {
    let raw_files = match crate::templates::get_template_files(&framework) {
        Ok(files) => files,
        Err(e) => {
            return Err((
                axum::http::StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ));
        }
    };

    let mut files = std::collections::HashMap::new();
    for (path, content) in raw_files {
        match String::from_utf8(content) {
            Ok(text) => { files.insert(path, text); }
            Err(e) => {
                let len = e.into_bytes().len();
                files.insert(path, format!("<binary file, {} bytes>", len));
            }
        }
    }

    let (name, description) = match framework.as_str() {
        "worker" => ("Worker", "Simple Cloudflare Worker with a fetch handler"),
        "nuxtjs" => ("Nuxt.js", "Full-stack Nuxt.js application with SSR"),
        "react-router" => ("React Router", "React Router v7 application with SSR"),
        _ => ("Unknown", "Unknown template"),
    };

    Ok(Json(TemplateDetail {
        id: framework.clone(),
        name: name.to_string(),
        framework: framework.clone(),
        description: description.to_string(),
        files,
    }))
}