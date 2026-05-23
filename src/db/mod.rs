pub mod crud;
pub mod models;

use crate::error::{FugueError, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn init_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|e| FugueError::ConfigError(format!("Failed to connect to PostgreSQL: {}", e)))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| FugueError::ConfigError(format!("Failed to run migrations: {}", e)))?;

    Ok(pool)
}
