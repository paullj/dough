pub mod models;
pub mod repository;
pub mod backup;

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

/// Initialize database connection pool
pub async fn init_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Create database file if it doesn't exist
    let path = database_url.trim_start_matches("sqlite://");
    let db_path = Path::new(path);

    // Create parent directories if they don't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    // Use SQLx's create_if_missing option to properly create the database
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            database_url.parse::<sqlx::sqlite::SqliteConnectOptions>()?
                .create_if_missing(true)
        )
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

/// Get default database path
pub fn default_db_path() -> String {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("dough");

    std::fs::create_dir_all(&data_dir).ok();

    format!("sqlite://{}/dough.db", data_dir.display())
}