#![allow(dead_code)]

use crate::db::models::*;
use crate::error::{FugueError, Result};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_app(
    pool: &PgPool,
    name: &str,
    framework: &str,
    description: Option<&str>,
) -> Result<App> {
    let slug = slugify(name);
    let subdomain = slug.clone();
    let id = Uuid::new_v4();
    let now = Utc::now();

    let app = sqlx::query_as::<_, App>(
        r#"
        INSERT INTO apps (id, name, slug, subdomain, framework, status, description, env_vars, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, 'created', $6, '{}', $7, $7)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(&slug)
    .bind(&subdomain)
    .bind(framework)
    .bind(description)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db_err) if db_err.constraint().is_some() => {
            FugueError::AppAlreadyExists(name.to_string())
        }
        _ => FugueError::DatabaseError(format!("Failed to create app: {}", e)),
    })?;

    Ok(app)
}

pub async fn get_app(pool: &PgPool, id: Uuid) -> Result<App> {
    sqlx::query_as::<_, App>("SELECT * FROM apps WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| FugueError::DatabaseError(format!("Failed to get app: {}", e)))?
        .ok_or_else(|| FugueError::AppNotFound(id.to_string()))
}

pub async fn get_app_by_slug(pool: &PgPool, slug: &str) -> Result<App> {
    sqlx::query_as::<_, App>("SELECT * FROM apps WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await
        .map_err(|e| FugueError::DatabaseError(format!("Failed to get app: {}", e)))?
        .ok_or_else(|| FugueError::AppNotFound(slug.to_string()))
}

pub async fn list_apps(
    pool: &PgPool,
    status: Option<&str>,
    framework: Option<&str>,
) -> Result<Vec<App>> {
    let mut query = String::from("SELECT * FROM apps WHERE 1=1");
    let mut binds: Vec<String> = Vec::new();

    if let Some(s) = status {
        binds.push(s.to_string());
        query.push_str(&format!(" AND status = ${}", binds.len()));
    }
    if let Some(f) = framework {
        binds.push(f.to_string());
        query.push_str(&format!(" AND framework = ${}", binds.len()));
    }

    query.push_str(" ORDER BY created_at DESC");

    let mut q = sqlx::query_as::<_, App>(&query);
    for bind in &binds {
        q = q.bind(bind);
    }

    q.fetch_all(pool)
        .await
        .map_err(|e| FugueError::DatabaseError(format!("Failed to list apps: {}", e)))
}

pub async fn update_app(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    env_vars: Option<&serde_json::Value>,
    status: Option<&str>,
    source_path: Option<&str>,
    build_path: Option<&str>,
) -> Result<App> {
    let now = Utc::now();

    let app = sqlx::query_as::<_, App>(
        r#"
        UPDATE apps SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            env_vars = COALESCE($4, env_vars),
            status = COALESCE($5, status),
            source_path = COALESCE($6, source_path),
            build_path = COALESCE($7, build_path),
            updated_at = $8
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(env_vars)
    .bind(status)
    .bind(source_path)
    .bind(build_path)
    .bind(now)
    .fetch_optional(pool)
    .await
    .map_err(|e| FugueError::DatabaseError(format!("Failed to update app: {}", e)))?
    .ok_or_else(|| FugueError::AppNotFound(id.to_string()))?;

    Ok(app)
}

pub async fn delete_app(pool: &PgPool, id: Uuid) -> Result<()> {
    let result = sqlx::query("DELETE FROM apps WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| FugueError::DatabaseError(format!("Failed to delete app: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(FugueError::AppNotFound(id.to_string()));
    }

    Ok(())
}

pub async fn create_build(pool: &PgPool, app_id: Uuid) -> Result<Build> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let build = sqlx::query_as::<_, Build>(
        r#"
        INSERT INTO builds (id, app_id, status, created_at)
        VALUES ($1, $2, 'pending', $3)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(app_id)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| FugueError::DatabaseError(format!("Failed to create build: {}", e)))?;

    Ok(build)
}

pub async fn update_build(
    pool: &PgPool,
    id: Uuid,
    status: &str,
    log: Option<&str>,
    error: Option<&str>,
) -> Result<Build> {
    let finished_at = if status == "success" || status == "failed" {
        Some(Utc::now())
    } else {
        None
    };

    let build = sqlx::query_as::<_, Build>(
        r#"
        UPDATE builds SET
            status = $2,
            log = COALESCE($3, log),
            error = COALESCE($4, error),
            finished_at = COALESCE($5, finished_at)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(log)
    .bind(error)
    .bind(finished_at)
    .fetch_optional(pool)
    .await
    .map_err(|e| FugueError::DatabaseError(format!("Failed to update build: {}", e)))?
    .ok_or_else(|| FugueError::Other(format!("Build {} not found", id)))?;

    Ok(build)
}

pub async fn get_build(pool: &PgPool, id: Uuid) -> Result<Build> {
    sqlx::query_as::<_, Build>("SELECT * FROM builds WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| FugueError::DatabaseError(format!("Failed to get build: {}", e)))?
        .ok_or_else(|| FugueError::Other(format!("Build {} not found", id)))
}

pub async fn list_builds(pool: &PgPool, app_id: Uuid) -> Result<Vec<Build>> {
    sqlx::query_as::<_, Build>("SELECT * FROM builds WHERE app_id = $1 ORDER BY created_at DESC")
        .bind(app_id)
        .fetch_all(pool)
        .await
        .map_err(|e| FugueError::DatabaseError(format!("Failed to list builds: {}", e)))
}

pub async fn create_deployment(pool: &PgPool, app_id: Uuid, build_id: Uuid) -> Result<Deployment> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let max_version: Option<(i32,)> =
        sqlx::query_as("SELECT COALESCE(MAX(version), 0) FROM deployments WHERE app_id = $1")
            .bind(app_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| FugueError::DatabaseError(format!("Failed to get max version: {}", e)))?;

    let version = max_version.map(|(v,)| v).unwrap_or(0) + 1;

    let deployment = sqlx::query_as::<_, Deployment>(
        r#"
        INSERT INTO deployments (id, app_id, build_id, version, status, created_at)
        VALUES ($1, $2, $3, $4, 'starting', $5)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(app_id)
    .bind(build_id)
    .bind(version)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| FugueError::DatabaseError(format!("Failed to create deployment: {}", e)))?;

    Ok(deployment)
}

pub async fn update_deployment_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
) -> Result<Deployment> {
    let now = Utc::now();

    let deployment = sqlx::query_as::<_, Deployment>(
        r#"
        UPDATE deployments SET
            status = $2,
            started_at = CASE WHEN $2 = 'running' THEN $3 ELSE started_at END,
            stopped_at = CASE WHEN $2 = 'stopped' THEN $3 ELSE stopped_at END
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(now)
    .fetch_optional(pool)
    .await
    .map_err(|e| FugueError::DatabaseError(format!("Failed to update deployment: {}", e)))?
    .ok_or_else(|| FugueError::Other(format!("Deployment {} not found", id)))?;

    Ok(deployment)
}

pub async fn get_active_deployment(pool: &PgPool, app_id: Uuid) -> Result<Option<Deployment>> {
    let deployment = sqlx::query_as::<_, Deployment>(
        "SELECT * FROM deployments WHERE app_id = $1 AND status = 'running' ORDER BY version DESC LIMIT 1",
    )
    .bind(app_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| FugueError::DatabaseError(format!("Failed to get active deployment: {}", e)))?;

    Ok(deployment)
}

pub async fn count_apps(pool: &PgPool) -> Result<(i64, i64)> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM apps")
        .fetch_one(pool)
        .await
        .map_err(|e| FugueError::DatabaseError(format!("Failed to count apps: {}", e)))?;

    let running: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM apps WHERE status = 'running'")
        .fetch_one(pool)
        .await
        .map_err(|e| FugueError::DatabaseError(format!("Failed to count running apps: {}", e)))?;

    Ok((total.0, running.0))
}

pub fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_simple() {
        assert_eq!(slugify("My Blog"), "my-blog");
        assert_eq!(slugify("hello"), "hello");
        assert_eq!(slugify("My App"), "my-app");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("My Blog!"), "my-blog");
        assert_eq!(slugify("app@home"), "app-home");
        assert_eq!(slugify("test.app"), "test-app");
    }

    #[test]
    fn test_slugify_multiple_spaces() {
        assert_eq!(slugify("my   blog"), "my-blog");
        assert_eq!(slugify("a  b  c"), "a-b-c");
    }

    #[test]
    fn test_slugify_hyphens() {
        assert_eq!(slugify("my-blog"), "my-blog");
        assert_eq!(slugify("my--blog"), "my-blog");
    }

    #[test]
    fn test_slugify_leading_trailing() {
        assert_eq!(slugify(" my-blog "), "my-blog");
        assert_eq!(slugify("--my-blog--"), "my-blog");
    }

    #[test]
    fn test_slugify_empty() {
        assert_eq!(slugify(""), "");
        assert_eq!(slugify("   "), "");
        assert_eq!(slugify("---"), "");
    }

    #[test]
    fn test_slugify_uppercase() {
        assert_eq!(slugify("MY-BLOG"), "my-blog");
        assert_eq!(slugify("MyBlog"), "myblog");
    }

    #[test]
    fn test_slugify_underscores() {
        assert_eq!(slugify("my_blog"), "my-blog");
        assert_eq!(slugify("my___blog"), "my-blog");
    }
}
