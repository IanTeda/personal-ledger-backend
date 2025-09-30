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

use crate::database::DatabaseError;
use crate::LedgerConfig;
use crate::config::ConnectionPool;

/// Initialize the database connection pool and run migrations.
///
/// This function creates a connection pool based on the provided configuration,
/// runs any pending SQL migrations, and returns the ready-to-use pool.
///
/// # Errors
///
/// Returns an error if:
/// - The connection pool cannot be created (e.g., invalid config or connection failure)
/// - Migrations fail to run (e.g., SQL syntax errors or schema conflicts)
pub async fn connect(
    config: &LedgerConfig,
) -> Result<ConnectionPool, DatabaseError> {
    // Create the connection pool
    let pool = config.database.connection_pool()
        .await.map_err(|e| DatabaseError::Connection(e.to_string()))?;

    // TODO: Add Display to Connection pool so we can trace the database type connected to
    tracing::info!("Database connection established");

    // Run migrations on the appropriate database pool
    // Each database engine has its own migration runner because each database engine
    // has unique SQL dialects and migration requirements.
    match &pool {
        ConnectionPool::Sqlite(p) => {
            sqlx::migrate!("./migrations").run(p).await?;
            tracing::info!("SQLite datbase migration complete.");
        }
        ConnectionPool::Postgres(p) => {
            sqlx::migrate!("./migrations").run(p).await?;
            tracing::info!("Postgres datbase migration complete.");
        }
    }

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DatabaseConfig, DbEngine};

    #[tokio::test]
    async fn connect_sqlite_success() {
        let config = LedgerConfig {
            server: Default::default(),
            database: DatabaseConfig {
                kind: DbEngine::Sqlite,
                database: ":memory:".to_string(),
                postgres: None,
            },
        };

        let result = connect(&config).await;
        assert!(result.is_ok(), "SQLite connection and migration should succeed");

        let pool = result.unwrap();
        match pool {
            ConnectionPool::Sqlite(p) => {
                // Verify the pool works by running a simple query
                let row: (i64,) = sqlx::query_as("SELECT 1")
                    .fetch_one(&p)
                    .await
                    .expect("Should be able to query the SQLite database");
                assert_eq!(row.0, 1);
            }
            ConnectionPool::Postgres(_) => panic!("Expected SQLite pool"),
        }
    }

    #[tokio::test]
    async fn connect_sqlite_connection_failure() {
        // Use an invalid path that should fail
        let config = LedgerConfig {
            server: Default::default(),
            database: DatabaseConfig {
                kind: DbEngine::Sqlite,
                database: "/invalid/path/that/does/not/exist/database".to_string(),
                postgres: None,
            },
        };

        let result = connect(&config).await;
        assert!(result.is_err(), "SQLite connection should fail for invalid path");

        let err = result.unwrap_err();
        match err {
            DatabaseError::Connection(msg) => {
                assert!(msg.contains("SQLite connection failed"), 
                       "Error message should indicate SQLite connection failure: {}", msg);
            }
            other => panic!("Expected DatabaseError::Connection, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn connect_postgres_missing_config() {
        let config = LedgerConfig {
            server: Default::default(),
            database: DatabaseConfig {
                kind: DbEngine::Postgres,
                database: "test_db".to_string(),
                postgres: None,
            },
        };

        let result = connect(&config).await;
        assert!(result.is_err(), "Postgres connection should fail without config");

        let err = result.unwrap_err();
        match err {
            DatabaseError::Connection(msg) => {
                assert!(msg.contains("postgres configuration missing"), 
                       "Error message should indicate missing postgres config: {}", msg);
            }
            other => panic!("Expected DatabaseError::Connection, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn connect_postgres_invalid_connection() {
        use crate::config::PostgresConfig;
        use secrecy::SecretString;

        let config = LedgerConfig {
            server: Default::default(),
            database: DatabaseConfig {
                kind: DbEngine::Postgres,
                database: "test_db".to_string(),
                postgres: Some(PostgresConfig {
                    host: "127.0.0.1".to_string(),
                    port: 1, // Invalid port
                    user: "user".to_string(),
                    password: SecretString::new("pass".to_string().into()),
                    url: Some("postgres://user:pass@127.0.0.1:1/test_db".to_string()),
                    ssl_mode: None,
                    max_pool_size: None,
                    connect_timeout_secs: Some(1),
                }),
            },
        };

        let result = connect(&config).await;
        assert!(result.is_err(), "Postgres connection should fail for invalid/unreachable server");

        let err = result.unwrap_err();
        match err {
            DatabaseError::Connection(msg) => {
                assert!(msg.contains("Postgres connection failed"), 
                       "Error message should indicate Postgres connection failure: {}", msg);
            }
            other => panic!("Expected DatabaseError::Connection, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn connect_returns_correct_pool_type() {
        // Test SQLite pool type
        let sqlite_config = LedgerConfig {
            server: Default::default(),
            database: DatabaseConfig {
                kind: DbEngine::Sqlite,
                database: ":memory:".to_string(),
                postgres: None,
            },
        };

        let result = connect(&sqlite_config).await;
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ConnectionPool::Sqlite(_)));

        // Test Postgres pool type (will fail due to missing config, but we can check the error type)
        let postgres_config = LedgerConfig {
            server: Default::default(),
            database: DatabaseConfig {
                kind: DbEngine::Postgres,
                database: "test_db".to_string(),
                postgres: None,
            },
        };

        let result = connect(&postgres_config).await;
        assert!(result.is_err()); // Should fail due to missing config
    }

    #[test]
    fn connect_function_signature() {
        // This test ensures the function signature hasn't changed unexpectedly
        // and that it returns the expected types
        use std::future::Future;
        use std::pin::Pin;

        let config = LedgerConfig::default();
        
        // Check that connect returns a Future by checking the type
        let future = connect(&config);
        let boxed_future: Pin<Box<dyn Future<Output = Result<ConnectionPool, DatabaseError>>>> = Box::pin(future);
        std::mem::drop(boxed_future); // Explicitly drop to satisfy clippy
        
        // If we get here, the types are correct
    }
}