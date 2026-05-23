use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::Value;
use tower::util::ServiceExt;

async fn setup_app() -> Router {
    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://fugue:fugue@localhost:5433/fugue".to_string());

    let db = fugue::db::init_pool(&db_url).await.expect("Failed to connect to database");

    let config = fugue::config::PlatformConfig::default();
    let pm = fugue::process::ProcessManager::new(&config).expect("Failed to create ProcessManager");
    let process = std::sync::Arc::new(tokio::sync::RwLock::new(pm));

    let state = fugue::api::AppState {
        db,
        process,
        config,
    };

    fugue::api::api_router(state)
}

async fn parse_json(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
#[ignore]
async fn test_platform_status() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/platform/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json(response).await;
    assert_eq!(body["version"], "0.1.0");
    assert!(body["apps_total"].is_number());
    assert!(body["apps_running"].is_number());
}

#[tokio::test]
#[ignore]
async fn test_create_and_list_app() {
    let app = setup_app().await;

    // Create app
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/apps")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": "test-integration",
                        "framework": "single-file",
                        "description": "Integration test app"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let created = parse_json(response).await;
    assert_eq!(created["name"], "test-integration");
    assert_eq!(created["slug"], "test-integration");
    assert_eq!(created["subdomain"], "test-integration");
    assert_eq!(created["framework"], "single-file");
    assert_eq!(created["status"], "created");
    assert_eq!(created["description"], "Integration test app");

    let app_id = created["id"].as_str().unwrap();

    // List apps
    let response = setup_app()
        .await
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let apps = parse_json(response).await;
    assert!(apps.as_array().unwrap().len() >= 1);

    // Get app by ID
    let response = setup_app()
        .await
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/apps/{}", app_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let fetched = parse_json(response).await;
    assert_eq!(fetched["id"].as_str().unwrap(), app_id);

    // Cleanup
    let pool = {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://fugue:fugue@localhost:5433/fugue".to_string());
        fugue::db::init_pool(&db_url).await.unwrap()
    };
    let uuid = uuid::Uuid::parse_str(app_id).unwrap();
    let _ = fugue::db::crud::delete_app(&pool, uuid).await;
}

#[tokio::test]
#[ignore]
async fn test_create_app_invalid_framework() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/apps")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": "bad-app",
                        "framework": "invalid-framework"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_get_nonexistent_app() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore]
async fn test_delete_nonexistent_app() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/apps/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore]
async fn test_update_app() {
    let pool = {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://fugue:fugue@localhost:5433/fugue".to_string());
        fugue::db::init_pool(&db_url).await.unwrap()
    };

    let app = fugue::db::crud::create_app(&pool, "update-test", "single-file", Some("original"))
        .await
        .unwrap();

    let router = setup_app().await;

    // Update description
    let response = router
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/apps/{}", app.id))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "description": "updated description"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let updated = parse_json(response).await;
    assert_eq!(updated["description"], "updated description");

    // Cleanup
    let _ = fugue::db::crud::delete_app(&pool, app.id).await;
}

#[tokio::test]
#[ignore]
async fn test_list_apps_filter_by_framework() {
    let pool = {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://fugue:fugue@localhost:5433/fugue".to_string());
        fugue::db::init_pool(&db_url).await.unwrap()
    };

    let _ = fugue::db::crud::create_app(&pool, "filter-sf", "single-file", None).await;
    let _ = fugue::db::crud::create_app(&pool, "filter-rr", "react-router", None).await;

    let router = setup_app().await;

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps?framework=single-file")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let apps = parse_json(response).await;
    let arr = apps.as_array().unwrap();
    assert!(arr.iter().all(|a| a["framework"] == "single-file"));

    // Cleanup
    let _ = fugue::db::crud::delete_app(&pool, uuid::Uuid::parse_str(arr[0]["id"].as_str().unwrap()).unwrap()).await;
    let rr = fugue::db::crud::list_apps(&pool, None, Some("react-router")).await.unwrap();
    for a in rr {
        let _ = fugue::db::crud::delete_app(&pool, a.id).await;
    }
}

#[tokio::test]
#[ignore]
async fn test_deploy_creates_build() {
    let pool = {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://fugue:fugue@localhost:5433/fugue".to_string());
        fugue::db::init_pool(&db_url).await.unwrap()
    };

    let app = fugue::db::crud::create_app(&pool, "deploy-test", "single-file", None)
        .await
        .unwrap();

    // Upload source first
    let source_dir = fugue::config::apps_data_dir().join(app.id.to_string()).join("source");
    std::fs::create_dir_all(&source_dir).unwrap();
    std::fs::write(
        source_dir.join("index.js"),
        "export default { async fetch() { return new Response('ok'); } };",
    )
    .unwrap();

    let router = setup_app().await;

    // Deploy
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/apps/{}/deploy", app.id))
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let result = parse_json(response).await;
    assert_eq!(result["status"], "building");
    assert!(result["build_id"].is_string());

    // Wait for build
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Check builds list
    let router2 = setup_app().await;
    let response = router2
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/apps/{}/builds", app.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let builds = parse_json(response).await;
    assert!(builds.as_array().unwrap().len() >= 1);

    // Cleanup
    let _ = fugue::db::crud::delete_app(&pool, app.id).await;
}

#[tokio::test]
#[ignore]
async fn test_start_stop_app() {
    let pool = {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://fugue:fugue@localhost:5433/fugue".to_string());
        fugue::db::init_pool(&db_url).await.unwrap()
    };

    let app = fugue::db::crud::create_app(&pool, "startstop-test", "single-file", None)
        .await
        .unwrap();

    // Upload source and deploy
    let source_dir = fugue::config::apps_data_dir().join(app.id.to_string()).join("source");
    std::fs::create_dir_all(&source_dir).unwrap();
    std::fs::write(
        source_dir.join("index.js"),
        "export default { async fetch() { return new Response('ok'); } };",
    )
    .unwrap();

    let router = setup_app().await;

    // Deploy first
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/apps/{}/deploy", app.id))
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Wait for deploy
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Stop
    let router2 = setup_app().await;
    let response = router2
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/apps/{}/stop", app.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let result = parse_json(response).await;
    assert_eq!(result["status"], "stopped");

    // Start
    let router3 = setup_app().await;
    let response = router3
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/apps/{}/start", app.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let result = parse_json(response).await;
    assert_eq!(result["status"], "running");
    assert!(result["url"].as_str().unwrap().contains("startstop-test"));

    // Cleanup
    let _ = fugue::db::crud::delete_app(&pool, app.id).await;
}
