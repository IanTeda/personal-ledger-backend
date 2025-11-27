//! Database connection pool management.
//!
//! This module provides a lightweight wrapper around SQLx's `Sqlite` pool that
//! centralizes error handling, logging, and connection lifecycle management.
//!
//! # Overview
//!
//! The [`DatabasePool`] struct manages SQLite database connections in a type-safe way,
//! mapping SQLx errors to domain-specific [`DatabaseError`] variants.
//!
//! # Examples
//!
//! ## Basic usage with SQLite
//!
//! ```rust,no_run
//! use personal_ledger_backend::database::pool::DatabasePool;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create and connect to an in-memory SQLite database
//! let db = DatabasePool::new("sqlite::memory:")
//!     .connect()
//!     .await?;
//!
//! // Get a reference to the pool for queries
//! let pool = db.get_pool()?;
//!
//! // Use the pool...
//! # Ok(())
//! # }
//! ```
//!
//! ## SQLite with cleanup
//!
//! ```rust,no_run
//! use personal_ledger_backend::database::pool::DatabasePool;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let db = DatabasePool::new("sqlite:test.db")
//!     .connect()
//!     .await?;
//!
//! // Do work with the database...
//! let pool = db.into_pool()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Graceful shutdown
//!
//! ```rust,no_run
//! use personal_ledger_backend::database::pool::DatabasePool;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut db = DatabasePool::new("sqlite::memory:")
//!     .connect()
//!     .await?;
//!
//! // Close the pool gracefully
//! db.close().await?;
//! # Ok(())
//! # }
//! ```
//!
use crate::database::{DatabaseError, DatabaseResult};

/// Lightweight wrapper around a SQLx `Sqlite` pool that carries its URL and
/// provides convenience helpers for connecting and extracting the pool.
///
/// The primary goal here is to centralise error mapping and logging so callers
/// get consistent `DatabaseError` variants instead of ad-hoc string conversions.
///
/// # Thread Safety
///
/// The underlying SQLx pool is thread-safe and can be cloned cheaply. Each clone
/// references the same connection pool.
///
/// # Lifecycle
///
/// 1. Create with [`DatabasePool::new`]
/// 2. Connect with [`DatabasePool::connect`]
/// 3. Access pool via [`DatabasePool::get_pool`] or [`DatabasePool::into_pool`]
/// 4. Optionally close with [`DatabasePool::close`]
///
/// # Examples
///
/// ```rust,no_run
/// use personal_ledger_backend::database::pool::DatabasePool;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = DatabasePool::new("sqlite::memory:")
///     .connect()
///     .await?;
///
/// let pool = db.get_pool()?;
/// // Use pool for queries...
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct DatabasePool {
  url: String,
  pool: Option<sqlx::SqlitePool>,
}

impl DatabasePool {
  /// Create an unconnected `DatabasePool` which stores the URL to be used
  /// when [`connect`](Self::connect) is invoked.
  ///
  /// This is a lightweight operation that only stores the URL string. No
  /// network operations or validations are performed until [`connect`](Self::connect)
  /// is called.
  ///
  /// # Arguments
  ///
  /// * `url` - Database connection URL. Format depends on the database engine:
  ///   - SQLite: `"sqlite:database_name.db"` or `"sqlite::memory:"` for in-memory
  ///
  /// # Examples
  ///
  /// ```rust
  /// use personal_ledger_backend::database::pool::DatabasePool;
  ///
  /// // SQLite in-memory database
  /// let db = DatabasePool::new("sqlite::memory:");
  ///
  /// // SQLite file database
  /// let db = DatabasePool::new("sqlite:test.db");
  /// ```
  pub fn new(url: &str) -> Self {
    tracing::info!(url = %url, "New database pool created");

    DatabasePool {
      url: url.to_string(),
      pool: None,
    }
  }

  /// Establish a connection pool and run basic setup.
  ///
  /// This method performs several initialization steps:
  /// 1. Ensures SQLx "sqlite" drivers are installed (prevents runtime panics)
  /// 2. Establishes the connection pool
  /// 3. Returns `self` with the pool populated
  ///
  /// # Returns
  ///
  /// Returns `Ok(Self)` with an active connection pool on success.
  ///
  /// # Errors
  ///
  /// Returns [`DatabaseError::Connection`] if:
  /// - The connection URL is invalid or malformed
  /// - The database file cannot be accessed
  /// - The database doesn't exist and cannot be created
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use personal_ledger_backend::database::pool::DatabasePool;
  ///
  /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
  /// let db = DatabasePool::new("sqlite::memory:")
  ///     .connect()
  ///     .await?;
  ///
  /// // Pool is now ready for use
  /// let pool = db.get_pool()?;
  /// # Ok(())
  /// # }
  /// ```
  pub async fn connect(mut self) -> DatabaseResult<Self> {
    // Ensure the SQLx "sqlite" driver is installed.
    // This prevents the "No drivers installed" panic from sqlx.
    sqlx::sqlite::SqlitePoolOptions::new();

    // Build the connection pool. Map any sqlx error to DatabaseError::Sqlx
    // to preserve the original error and make higher-level handling easier.
    let pool = sqlx::SqlitePool::connect(&self.url)
      .await
      .map_err(|e| {
        // Connection failures are mapped to `Connection` to make it clear
        // the error occurred while establishing a connection.
        tracing::error!(error = %e, url = %self.url, "Connection error: failed to establish database pool");
        DatabaseError::Connection(e.to_string())
      })?;

    self.pool = Some(pool);

    tracing::info!("Database connection established");

    
    Ok(self)
  }

  /// Borrow the inner connection pool if connected.
  ///
  /// This method provides a reference to the underlying SQLx pool without
  /// transferring ownership, allowing multiple queries against the same pool.
  ///
  /// # Returns
  ///
  /// Returns `Ok(&Pool<Any>)` if the pool is connected and available.
  ///
  /// # Errors
  ///
  /// Returns [`DatabaseError::Connection`] with message "Database pool is not connected"
  /// if [`connect`](Self::connect) hasn't been called or the pool has been consumed
  /// by [`into_pool`](Self::into_pool).
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use personal_ledger_backend::database::pool::DatabasePool;
  ///
  /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
  /// let db = DatabasePool::new("sqlite::memory:")
  ///     .connect()
  ///     .await?;
  ///
  /// // Get a reference to the pool
  /// let pool = db.get_pool()?;
  ///
  /// // Use pool for queries
  /// let row: (i64,) = sqlx::query_as("SELECT 1")
  ///     .fetch_one(pool)
  ///     .await?;
  /// # Ok(())
  /// # }
  /// ```
  pub fn get_pool(&self) -> DatabaseResult<&sqlx::SqlitePool> {
    self.pool
      .as_ref()
      .ok_or_else(|| DatabaseError::Connection("Database pool is not connected".into()))
  }

  /// Take ownership of the inner connection pool.
  ///
  /// This method consumes the `DatabasePool` wrapper and returns the underlying
  /// SQLx pool, transferring ownership to the caller. Useful for passing the pool
  /// into application code after setup.
  ///
  /// # Returns
  ///
  /// Returns `Ok(Pool<Any>)` containing the underlying connection pool.
  ///
  /// # Errors
  ///
  /// Returns [`DatabaseError::Connection`] with message "Database pool is not connected"
  /// if [`connect`](Self::connect) hasn't been called or the pool has already been
  /// taken by a previous call to this method or [`close`](Self::close).
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use personal_ledger_backend::database::pool::DatabasePool;
  ///
  /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
  /// let db = DatabasePool::new("sqlite::memory:")
  ///     .connect()
  ///     .await?;
  ///
  /// // Take ownership of the pool
  /// let pool = db.into_pool()?;
  ///
  /// // Pool can now be stored in application state, passed to services, etc.
  /// // let app_state = AppState { pool };
  /// # Ok(())
  /// # }
  /// ```
  pub fn into_pool(self) -> DatabaseResult<sqlx::SqlitePool> {
    println!("Into pool called");
    self.pool
      .ok_or_else(|| DatabaseError::Connection("Database pool is not connected".into()))
  }

  /// Close the inner connection pool if present.
  ///
  /// This method gracefully shuts down the connection pool, closing all active
  /// connections and releasing resources. After calling this method, the pool
  /// is no longer available and subsequent calls to [`get_pool`](Self::get_pool)
  /// will return an error.
  ///
  /// # Returns
  ///
  /// Returns `Ok(())` if the pool was successfully closed.
  ///
  /// # Errors
  ///
  /// Returns [`DatabaseError::Connection`] if:
  /// - The pool has not been connected via [`connect`](Self::connect)
  /// - The pool has already been closed
  /// - The pool was consumed by [`into_pool`](Self::into_pool)
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use personal_ledger_backend::database::pool::DatabasePool;
  ///
  /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
  /// let mut db = DatabasePool::new("sqlite::memory:")
  ///     .connect()
  ///     .await?;
  ///
  /// // Do work...
  ///
  /// // Gracefully close the pool
  /// db.close().await?;
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// # Idempotence
  ///
  /// This method is **not** idempotent. Calling it multiple times will return
  /// an error on subsequent calls:
  ///
  /// ```rust,no_run
  /// # use personal_ledger_backend::database::pool::DatabasePool;
  /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
  /// # let mut db = DatabasePool::new("sqlite::memory:").connect().await?;
  /// db.close().await?; // Ok
  /// db.close().await?; // Error: pool not connected
  /// # Ok(())
  /// # }
  /// ```
  pub async fn close(&mut self) -> DatabaseResult<()> {
    if let Some(pool) = self.pool.take() {
      // `close` does not return a Result; it performs an orderly shutdown.
      pool.close().await;
      tracing::info!(url = %self.url, "Database pool closed");
      Ok(())
    } else {
      Err(DatabaseError::Connection("Database pool is not connected".into()))
    }
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;

    #[test]
    fn test_database_pool_new() {
        let url = "sqlite::memory:";
        let pool = DatabasePool::new(url);
        
        assert_eq!(pool.url, url);
        assert!(pool.pool.is_none());
    }

    #[test]
    fn test_database_pool_new_with_various_urls() {
        // Test with SQLite URL
        let sqlite_pool = DatabasePool::new("sqlite:test.db");
        assert_eq!(sqlite_pool.url, "sqlite:test.db");

        // Test with SQLite in-memory
        let memory_pool = DatabasePool::new("sqlite::memory:");
        assert_eq!(memory_pool.url, "sqlite::memory:");

        // Test with empty string (edge case)
        let empty_pool = DatabasePool::new("");
        assert_eq!(empty_pool.url, "");
    }

    #[test]
    fn test_get_pool_when_not_connected() {
        let pool = DatabasePool::new("sqlite::memory:");
        let result = pool.get_pool();
        
        assert!(result.is_err());
        match result {
            Err(DatabaseError::Connection(msg)) => {
                assert_eq!(msg, "Database pool is not connected");
            }
            _ => panic!("Expected DatabaseError::Connection"),
        }
    }

    #[test]
    fn test_into_pool_when_not_connected() {
        let pool = DatabasePool::new("sqlite::memory:");
        let result = pool.into_pool();
        
        assert!(result.is_err());
        match result {
            Err(DatabaseError::Connection(msg)) => {
                assert_eq!(msg, "Database pool is not connected");
            }
            _ => panic!("Expected DatabaseError::Connection"),
        }
    }

    #[tokio::test]
    async fn test_close_when_not_connected() {
        let mut pool = DatabasePool::new("sqlite::memory:");
        let result = pool.close().await;
        
        assert!(result.is_err());
        match result {
            Err(DatabaseError::Connection(msg)) => {
                assert_eq!(msg, "Database pool is not connected");
            }
            _ => panic!("Expected DatabaseError::Connection"),
        }
    }

    #[tokio::test]
    async fn test_connect_with_sqlite_memory() {
        let pool = DatabasePool::new("sqlite::memory:");
        let result = pool.connect().await;
        
        assert!(result.is_ok());
        let connected_pool = result.unwrap();
        assert!(connected_pool.pool.is_some());
        
        // Verify we can get the pool
        let pool_ref = connected_pool.get_pool();
        assert!(pool_ref.is_ok());
    }

    #[tokio::test]
    async fn test_connect_with_invalid_url() {
        let pool = DatabasePool::new("invalid://bad_url");
        let result = pool.connect().await;
        
        assert!(result.is_err());
        match result {
            Err(DatabaseError::Connection(_)) => {
                // Expected error type
            }
            other => panic!("Expected DatabaseError::Connection, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_get_pool_after_connect() {
        let pool = DatabasePool::new("sqlite::memory:");
        let connected_pool = pool.connect().await.unwrap();
        
        let pool_ref = connected_pool.get_pool();
        assert!(pool_ref.is_ok());
        
        // Verify the pool is functional
        let pool = pool_ref.unwrap();
        let row: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(row.0, 1);
    }

    #[tokio::test]
    async fn test_into_pool_after_connect() {
        let pool = DatabasePool::new("sqlite::memory:");
        let connected_pool = pool.connect().await.unwrap();
        
        let pool = connected_pool.into_pool();
        assert!(pool.is_ok());
        
        // Verify the pool is functional
        let pool = pool.unwrap();
        let row: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(row.0, 1);
    }

    #[tokio::test]
    async fn test_close_after_connect() {
        let pool = DatabasePool::new("sqlite::memory:");
        let mut connected_pool = pool.connect().await.unwrap();
        
        // Pool should be present before close
        assert!(connected_pool.pool.is_some());
        
        // Close should succeed
        let result = connected_pool.close().await;
        assert!(result.is_ok());
        
        // Pool should be None after close
        assert!(connected_pool.pool.is_none());
        
        // Subsequent operations should fail
        let get_result = connected_pool.get_pool();
        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_close_idempotence() {
        let pool = DatabasePool::new("sqlite::memory:");
        let mut connected_pool = pool.connect().await.unwrap();
        
        // First close should succeed
        let result1 = connected_pool.close().await;
        assert!(result1.is_ok());
        
        // Second close should fail with appropriate error
        let result2 = connected_pool.close().await;
        assert!(result2.is_err());
        match result2 {
            Err(DatabaseError::Connection(_)) => {
                // Expected
            }
            _ => panic!("Expected DatabaseError::Connection"),
        }
    }

    #[test]
    fn test_database_pool_with_fake_urls() {
        // Use fake crate to generate random test URLs
        for _ in 0..10 {
            let random_name: String = (5..20).fake();
            let url = format!("sqlite:{}.db", random_name);
            let pool = DatabasePool::new(&url);
            
            assert_eq!(pool.url, url);
            assert!(pool.pool.is_none());
        }
    }

    #[tokio::test]
    async fn test_multiple_connections_are_independent() {
        let pool1 = DatabasePool::new("sqlite::memory:");
        let pool2 = DatabasePool::new("sqlite::memory:");
        
        let connected1 = pool1.connect().await.unwrap();
        let connected2 = pool2.connect().await.unwrap();
        
        // Both should have independent pools
        assert!(connected1.get_pool().is_ok());
        assert!(connected2.get_pool().is_ok());
        
        // Verify independence by creating different tables
        let p1 = connected1.get_pool().unwrap();
        let p2 = connected2.get_pool().unwrap();
        
        sqlx::query("CREATE TABLE test1 (id INTEGER)")
            .execute(p1)
            .await
            .unwrap();
        
        sqlx::query("CREATE TABLE test2 (id INTEGER)")
            .execute(p2)
            .await
            .unwrap();
        
        // test1 should exist in pool1 but not pool2
        let result1 = sqlx::query("SELECT * FROM test1")
            .fetch_optional(p1)
            .await;
        assert!(result1.is_ok());
        
        let result2 = sqlx::query("SELECT * FROM test1")
            .fetch_optional(p2)
            .await;
        assert!(result2.is_err()); // Table doesn't exist in pool2
    }

    #[tokio::test]
    async fn test_pool_url_is_preserved() {
        let original_url = "sqlite::memory:";
        let pool = DatabasePool::new(original_url);
        
        assert_eq!(pool.url, original_url);
        
        let connected_pool = pool.connect().await.unwrap();
        assert_eq!(connected_pool.url, original_url);
        
        // URL should still be accessible after taking the pool
        let url_before = connected_pool.url.clone();
        let _extracted_pool = connected_pool.into_pool().unwrap();
        // Can't check url after into_pool since it consumes self
        assert_eq!(url_before, original_url);
    }

    #[tokio::test]
    async fn test_error_messages_are_informative() {
        let pool = DatabasePool::new("sqlite::memory:");
        
        // Test get_pool error message
        let err = pool.get_pool().unwrap_err();
        assert!(matches!(err, DatabaseError::Connection(_)));
        assert_eq!(format!("{}", err), "Error connecting to the database: Database pool is not connected");
        
        // Test into_pool error message
        let pool2 = DatabasePool::new("sqlite::memory:");
        let err2 = pool2.into_pool().unwrap_err();
        assert!(matches!(err2, DatabaseError::Connection(_)));
        assert_eq!(format!("{}", err2), "Error connecting to the database: Database pool is not connected");
    }

    #[tokio::test]
    async fn test_connect_creates_sqlite_database_if_not_exists() {
        // This test verifies that SQLite databases are created automatically
        // when they don't exist (which is the default SQLite behavior)
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_auto_create.db");
    // Use the SQLx-compatible absolute-file URL form `sqlite://<path>` so
    // the driver can open/create the file at the absolute temp path.
    // Use the SQLx-compatible absolute-file URL form `sqlite://<path>` and
    // request read-write-create mode via URI query so SQLite will create
    // the file if it does not exist in environments where URI handling
    // is enabled.
    let url = format!("sqlite://{}?mode=rwc", db_path.display());
        
        // Ensure the file doesn't exist initially
        if db_path.exists() {
            std::fs::remove_file(&db_path).unwrap();
        }
        
    let pool = DatabasePool::new(&url);
    let result = pool.connect().await;
        
        // Should succeed - SQLite creates the database file automatically
        assert!(result.is_ok());
        
        // Clean up
        let _ = std::fs::remove_file(&db_path);
    }

    #[tokio::test]
    async fn test_connect_is_idempotent_for_existing_sqlite_db() {
        // This test documents the behavior when connecting to an existing SQLite DB
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_idempotent.db");
    // Use SQLx-compatible absolute-file URL form for existing DB path.
    // Use SQLx-compatible absolute-file URL with create mode for existing DB path.
    let url = format!("sqlite://{}?mode=rwc", db_path.display());
        
        // Create the database first
        {
            let pool = DatabasePool::new(&url);
            let connected = pool.connect().await.unwrap();
            // Do some work to ensure it's created
            let pool_ref = connected.get_pool().unwrap();
            sqlx::query("CREATE TABLE test (id INTEGER)")
                .execute(pool_ref)
                .await
                .unwrap();
        }
        
        // Now connect again - should work fine
        let pool = DatabasePool::new(&url);
        let result = pool.connect().await;
        
        // Should succeed
        assert!(result.is_ok());
        
        // Clean up
        let _ = std::fs::remove_file(&db_path);
    }
}