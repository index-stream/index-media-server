use sqlx::{sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous}, SqlitePool};
use std::str::FromStr;
use anyhow::Result;

/// Create a SQLite connection pool with optimized settings for desktop apps
pub async fn connect_pool(db_path: &std::path::Path) -> Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(
        &format!("sqlite://{}", db_path.to_string_lossy())
    )?
    .create_if_missing(true)
    // Performance & durability tuning for desktop apps:
    .journal_mode(SqliteJournalMode::Wal)
    .synchronous(SqliteSynchronous::Normal) // Balance between performance and durability
    .foreign_keys(true);

    let pool = SqlitePool::connect_with(opts).await?;
    
    // PRAGMA tuning that requires a connection:
    sqlx::query("PRAGMA journal_size_limit = 67108864;").execute(&pool).await?;
    
    Ok(pool)
}

/// Initialize the database schema
pub async fn init_schema(pool: &SqlitePool) -> Result<()> {
    // Read the schema.sql file
    let schema_sql = include_str!("../../../schema.sql");
    
    // Execute the schema
    sqlx::query(schema_sql)
        .execute(pool)
        .await?;
    
    Ok(())
}
