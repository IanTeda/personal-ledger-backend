//! # Database Connection and Migration Module
//!
//! This module provides the main entry point for initializing a database connection pool
//! and running migrations for the Personal Ledger backend. It supports both SQLite and
//! PostgreSQL backends, with configuration provided via the `LedgerConfig` type.
//!
//! ## Usage
//!
//! Use the `connect` async function to create a connection pool and run migrations:
//!
//! ```rust
//! let pool = connect(&config).await?;
//! ```
//!
//! ## Error Handling
//!
//! Errors are returned as `DatabaseError` and include connection failures, migration errors,
//! and configuration issues. Connection errors are wrapped as `DatabaseError::Connection` for clarity.
//!
//! ## Supported Backends
//!
//! - **SQLite**: Use `:memory:` for in-memory DB, or a file name for file-based DB.
//! - **Postgres**: Requires a valid `PostgresConfig` in `LedgerConfig`.
//!
//! ## Test Coverage
//!
//! Comprehensive unit tests cover connection success/failure, migration execution, and pool type detection.

use sqlx::any as SqlxAny;

use crate::database::DatabaseError;

/// Initialize the database connection pool and run migrations.
///
/// This function creates a SQLx connection pool based on the provided configuration,
/// runs any pending SQL migrations using the embedded migration files, and returns
/// a [`ConnectionPool`] wrapper containing both the pool and engine type information.
///
/// The function performs the following steps:
/// 1. Installs SQLx "any" driver defaults for runtime database selection
/// 2. Constructs a connection URL from the database configuration
/// 3. Establishes a connection pool to the database
/// 4. Wraps the pool in a [`ConnectionPool`] with engine type metadata
/// 5. Runs pending migrations from the `./migrations` directory
///
/// # Arguments
///
/// * `config` - Reference to the application's [`LedgerConfig`](crate::config::LedgerConfig)
///              containing database connection parameters
///
/// # Returns
///
/// Returns `Ok(ConnectionPool)` containing the initialized connection pool and engine type,
/// or a [`DatabaseError`] if connection or migration fails.
///
/// # Errors
///
/// Returns an error if:
/// - The database connection URL cannot be constructed (e.g., missing PostgreSQL config)
/// - The connection pool cannot be created (e.g., invalid credentials, unreachable host)
/// - Migrations fail to execute (e.g., SQL syntax errors, schema conflicts)
///
/// # Examples
///
/// ```rust,no_run
/// use personal_ledger_backend::config::LedgerConfig;
/// use personal_ledger_backend::database::connect;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Load configuration from environment and config files
///     let config = LedgerConfig::load()?;
///     
///     // Initialize database connection pool and run migrations
///     let connection_pool = connect(&config).await?;
///     
///     // Use the pool for database operations
///     println!("Connected to {:?} database", connection_pool.kind);
///     
///     Ok(())
/// }
/// ```
///
/// # Performance
///
/// The connection pool is configured with sensible defaults for connection limits
/// and timeouts. For production deployments, consider tuning these values through
/// the PostgreSQL configuration options (`max_pool_size`, `connect_timeout_secs`).
///
/// # Migration Files
///
/// Migrations are embedded at compile time from the `./migrations` directory
/// relative to the workspace root. SQLx will track which migrations have been
/// applied and only run new ones, making this function idempotent.
///
/// This function creates a connection pool based on the provided configuration,
/// runs any pending SQL migrations, and returns the ready-to-use pool.
/// # Arguments
///
/// * `config` - Reference to the application's [`LedgerConfig`](crate::config::LedgerConfig)
///              containing database connection parameters
///
/// # Returns
///
/// Returns `Ok(ConnectionPool)` containing the initialized connection pool and engine type,
/// or a [`DatabaseError`] if connection or migration fails.
///
/// # Errors
///
/// Returns an error if:
/// - The database connection URL cannot be constructed (e.g., missing PostgreSQL config)
/// - The connection pool cannot be created (e.g., invalid credentials, unreachable host)
/// - Migrations fail to execute (e.g., SQL syntax errors, schema conflicts)
///
/// # Examples
///
/// ```rust,no_run
/// use personal_ledger_backend::config::LedgerConfig;
/// use personal_ledger_backend::database::connect;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Load configuration from environment and config files
///     let config = LedgerConfig::load()?;
///     
///     // Initialize database connection pool and run migrations
///     let connection_pool = connect(&config).await?;
///     
///     // Use the pool for database operations
///     println!("Connected to {:?} database", connection_pool.kind);
///     
///     Ok(())
/// }
/// ```
///
/// # Performance
///
/// The connection pool is configured with sensible defaults for connection limits
/// and timeouts. For production deployments, consider tuning these values through
/// the PostgreSQL configuration options (`max_pool_size`, `connect_timeout_secs`).
///
/// # Migration Files
///
/// Migrations are embedded at compile time from the `./migrations` directory
/// relative to the workspace root. SQLx will track which migrations have been
/// applied and only run new ones, making this function idempotent.
pub async fn connect(
    url: &str,
) -> Result<sqlx::Pool<sqlx::Any>, DatabaseError> {
    // Ensure the SQLx "any" driver has the default drivers installed
    // https://docs.rs/sqlx/latest/sqlx/any/fn.install_default_drivers.html
    SqlxAny::install_default_drivers();

    // Build the connection URL from the config
    let pool = sqlx::Pool::<sqlx::Any>::connect(&url)
        .await
        .map_err(|e| DatabaseError::Connection(e.to_string()))?;

    tracing::info!("Database connection established");

    // Run migrations on the appropriate database pool
    // Each database engine has its own migration runner because each database engine
    // has unique SQL dialects and migration requirements.
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database migrations complete.");

    Ok(pool)
}
