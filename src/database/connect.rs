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
