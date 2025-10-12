//! Test utilities for database operations.
//!
//! This module provides helper functions for setting up and tearing down
//! test databases, particularly for handling random test database names
//! created by the `test_connection_url` function.

use sqlx::migrate::MigrateDatabase;
use sqlx::ConnectOptions;
use crate::database::{DatabaseError, DatabaseResult};

/// Drop a test database by URL using SQLx's force_drop_database.
///
/// This function attempts to drop a database specified in the connection URL.
/// It will forcefully terminate any active connections before dropping.
/// For SQLite in-memory databases, this is a no-op as they are automatically
/// cleaned up when connections close.
///
/// # Arguments
///
/// * `url` - The database connection URL to drop
///
/// # Returns
///
/// Returns `Ok(())` on success, or a [`DatabaseError`] if the drop operation fails.
///
/// # Examples
///
/// ```rust,no_run
/// # use personal_ledger_backend::database::test::drop_test_database;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Drop a test database
/// drop_test_database("postgres://postgres:password@localhost:5432/test_abc123").await?;
/// # Ok(())
/// # }
/// ```
pub async fn drop_test_database(url: &str) -> DatabaseResult<()> {
    // For SQLite in-memory databases, no cleanup needed
    if url.contains("::memory:") {
        return Ok(());
    }

    sqlx::any::Any::force_drop_database(url)
        .await
        .map_err(|e| DatabaseError::Other(format!("Failed to drop test database: {}", e)))?;

    tracing::info!("Dropped test database: {}", url);

    Ok(())
}

/// Clean up test databases using a pool connection.
///
/// This function extracts the database URL from the pool and drops it.
///
/// # Arguments
///
/// * `pool` - The database connection pool for the database to drop
///
/// # Returns
///
/// Returns `Ok(())` on success, or a [`DatabaseError`] if the drop operation fails.
pub async fn cleanup_test_databases(pool: &sqlx::Pool<sqlx::Any>) -> DatabaseResult<()> {
    let url = pool.connect_options().to_url_lossy().to_string();
    drop_test_database(&url).await
}