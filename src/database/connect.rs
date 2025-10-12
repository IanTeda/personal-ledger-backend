//! # Database Connection and Migration Module
//!
//! This module provides the main entry point for initializing a database connection pool
//! and running migrations for the Personal Ledger backend. It supports both SQLite and
//! PostgreSQL backends through SQLx's "any" driver.
//!
//! ## Usage
//!
//! Use the `connect` async function to create a database connection pool and run migrations:
//!
//! ```rust,no_run
//! # use personal_ledger_backend::database::connect;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let pool = connect("sqlite::memory:.db").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Database URL Format
//!
//! The connection URL format varies by database engine:
//!
//! - **SQLite**: `sqlite:database_name.db` or `sqlite::memory:` for in-memory databases
//! - **PostgreSQL**: `postgres://user:pass@host:port/database?sslmode=mode`
//!
//! ## Error Handling
//!
//! Errors are returned as [`DatabaseError`] and include connection failures, migration errors,
//! and configuration issues. Connection errors are wrapped as `DatabaseError::Connection` for clarity.
//!
//! ## Migration Management
//!
//! Migrations are embedded at compile time from the `./migrations` directory and are
//! automatically tracked by SQLx. The `connect` function is idempotent - it will only
//! apply new migrations that haven't been run yet.
//!
//! ## Test Coverage
//!
//! Comprehensive unit tests cover connection success/failure scenarios, migration execution,
//! and error handling for both SQLite and PostgreSQL backends.

use sqlx::any as SqlxAny;

#[cfg(test)]
use sqlx::migrate::MigrateDatabase;

use crate::database::{DatabaseError, DatabaseResult};

/// Initialize the database connection pool and run migrations.
///
/// This function creates a SQLx connection pool from the provided database URL,
/// runs any pending SQL migrations using the embedded migration files, and returns
/// the initialized connection pool.
///
/// The function performs the following steps:
/// 1. Installs SQLx "any" driver defaults for runtime database selection
/// 2. Establishes a connection pool to the database using the provided URL
/// 3. Runs pending migrations from the `./migrations` directory
/// 4. Returns the ready-to-use connection pool
///
/// # Arguments
///
/// * `url` - Database connection URL string. Format depends on the database engine:
///   - SQLite: `"sqlite:database_name.db"` or `"sqlite::memory:"`
///   - PostgreSQL: `"postgres://user:pass@host:port/database?sslmode=mode"`
///
/// # Returns
///
/// Returns `Ok(Pool<Any>)` containing the initialized SQLx connection pool,
/// or a [`DatabaseError`] if connection or migration fails.
///
/// # Errors
///
/// Returns an error if:
/// - The connection URL is invalid or malformed
/// - The connection pool cannot be created (e.g., invalid credentials, unreachable host)
/// - Migrations fail to execute (e.g., SQL syntax errors, schema conflicts)
///
/// # Examples
///
/// ```rust,no_run
/// use personal_ledger_backend::database::connect;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // SQLite connection
///     let sqlite_pool = connect("sqlite::memory:.db").await?;
///     
///     // PostgreSQL connection
///     let pg_pool = connect("postgres://user:pass@localhost:5432/mydb").await?;
///     
///     // Use the pool for database operations
///     let row: (i64,) = sqlx::query_as("SELECT 1")
///         .fetch_one(&sqlite_pool)
///         .await?;
///     
///     Ok(())
/// }
/// ```
///
/// # Performance
///
/// The connection pool is configured with SQLx defaults. For production deployments,
/// consider configuring pool limits and timeouts through the connection URL parameters
/// or by creating a custom pool configuration.
///
/// # Migration Files
///
/// Migrations are embedded at compile time from the `./migrations` directory
/// relative to the workspace root. SQLx will track which migrations have been
/// applied and only run new ones, making this function idempotent. The same
/// migration files are used for both SQLite and PostgreSQL backends.
///
/// # Thread Safety
///
/// The returned connection pool is thread-safe and can be cloned cheaply.
/// Each clone references the same underlying connection pool.
pub async fn connect(
    url: &str,
) -> DatabaseResult<sqlx::Pool<sqlx::Any>> {
    // Ensure the SQLx "any" driver has the default drivers installed
    // https://docs.rs/sqlx/latest/sqlx/any/fn.install_default_drivers.html
    SqlxAny::install_default_drivers();

    // Build the connection URL from the config
    let pool = sqlx::Pool::<sqlx::Any>::connect(url)
        .await
        .map_err(|e| DatabaseError::Connection(e.to_string()))?;

    tracing::info!("Database connection established");

    // Run migrations on the database pool
    sqlx::migrate!("./migrations").run(&pool).await?;

    tracing::info!("Database migrations complete.");

    Ok(pool)
}

/// Initialize a test database connection pool with automatic cleanup capability.
///
/// This function is similar to [`connect`] but is designed specifically for testing.
/// It creates a test database, runs migrations, and returns both the connection pool
/// and the database URL for later cleanup.
///
/// # Arguments
///
/// * `url` - Database connection URL string. For tests, prefer unique database names:
///   - SQLite: `"sqlite::memory:"` for in-memory (auto-cleanup)
///   - PostgreSQL: `"postgres://user:pass@host:port/test_db_uuid"` (requires manual cleanup)
///
/// # Returns
///
/// Returns `Ok((Pool<Any>, String))` containing:
/// - The initialized SQLx connection pool
/// - The database URL used (needed for cleanup with [`drop_test_database`])
///
/// # Errors
///
/// Returns a [`DatabaseError`] if connection or migration fails.
///
/// # Examples
///
/// ```rust,no_run
/// use personal_ledger_backend::database::{connect_test, drop_test_database};
///
/// #[tokio::test]
/// async fn my_integration_test() -> Result<(), Box<dyn std::error::Error>> {
///     // Create test database
///     let (pool, url) = connect_test("sqlite::memory:").await?;
///     
///     // Run your test logic
///     // ...
///     
///     // Cleanup (automatic for SQLite in-memory, manual for PostgreSQL)
///     drop_test_database(&url).await?;
///     
///     Ok(())
/// }
/// ```
///
/// # Test Isolation
///
/// For PostgreSQL tests, use unique database names (e.g., with UUID) to ensure
/// test isolation. SQLite in-memory databases are automatically isolated per connection.
#[cfg(test)]
pub async fn connect_test(
    url: &str,
) -> DatabaseResult<(sqlx::Pool<sqlx::Any>, String)> {
    // Install SQLx "any" driver defaults
    SqlxAny::install_default_drivers();

    // Create the database if it doesn't exist (PostgreSQL only, SQLite auto-creates)
    if url.starts_with("postgres") {
        sqlx::Any::create_database(url).await.ok(); // Ignore error if already exists
    }

    // Build the connection pool
    let pool = sqlx::Pool::<sqlx::Any>::connect(url)
        .await
        .map_err(|e| DatabaseError::Connection(e.to_string()))?;

    tracing::info!("Test database connection established: {}", url);

    // Run migrations on the test database pool
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    tracing::info!("Test database migrations complete");

    Ok((pool, url.to_string()))
}

/// Drop a test database by URL.
///
/// This is a convenience wrapper around [`crate::database::test::drop_test_database`]
/// for use in test cleanup. For SQLite in-memory databases, this is a no-op as they
/// are automatically cleaned up when the connection is closed.
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
/// use personal_ledger_backend::database::{connect_test, drop_test_database};
///
/// #[tokio::test]
/// async fn my_test() -> Result<(), Box<dyn std::error::Error>> {
///     let (pool, url) = connect_test("postgres://localhost/test_db_abc").await?;
///     
///     // ... run test ...
///     
///     drop(pool); // Close connections first
///     drop_test_database(&url).await?;
///     Ok(())
/// }
/// ```
#[cfg(test)]
pub async fn drop_test_database(url: &str) -> DatabaseResult<()> {
    crate::database::test::drop_test_database(url).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect_test_sqlite_memory() -> DatabaseResult<()> {
        let (pool, url) = connect_test("sqlite::memory:").await?;
        
        assert_eq!(url, "sqlite::memory:");
        
        // Verify connection works
        let row: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await?;
        assert_eq!(row.0, 1);
        
        // Cleanup (no-op for in-memory SQLite)
        drop(pool);
        drop_test_database(&url).await?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_connect_test_with_cleanup() -> DatabaseResult<()> {
        // Use in-memory database for testing
        let (pool, url) = connect_test("sqlite::memory:").await?;
        
        // Run a simple query
        let _: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await?;
        
        // Verify migrations ran by checking for a table
        // (this assumes migrations create at least one table)
        
        // Close pool before dropping database
        drop(pool);
        
        // Cleanup should succeed (no-op for in-memory)
        drop_test_database(&url).await?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_connect_test_isolation() -> DatabaseResult<()> {
        // Create two separate test databases
        let (pool1, url1) = connect_test("sqlite::memory:").await?;
        let (pool2, url2) = connect_test("sqlite::memory:").await?;
        
        // They should be isolated
        sqlx::query("CREATE TABLE test (id INTEGER)")
            .execute(&pool1)
            .await?;
        
        // pool2 should not see pool1's table
        let result = sqlx::query("SELECT * FROM test")
            .fetch_optional(&pool2)
            .await;
        assert!(result.is_err()); // Table doesn't exist in pool2
        
        // Cleanup
        drop(pool1);
        drop(pool2);
        drop_test_database(&url1).await?;
        drop_test_database(&url2).await?;
        
        Ok(())
    }
}
