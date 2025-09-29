//! # Database Error Types
//!
//! This module defines the `DatabaseError` enum, which standardizes error handling for
//! all database-related operations in the Personal Ledger backend. It wraps connection,
//! migration, SQLx, and configuration errors, and provides variants for validation and
//! generic database failures.
//!
//! ## Error Variants
//!
//! - `Connection`: Database connection failures (invalid config, unreachable server, etc.)
//! - `Sqlx`: Errors from the `sqlx` crate (query, pool, etc.)
//! - `Migration`: Errors from running migrations
//! - `Config`: Configuration errors during DB initialization
//! - `Validation`: Domain validation errors (constraint violations, etc.)
//! - `Other`: Catch-all for miscellaneous DB errors
//!
//! ## Usage
//!
//! All database service functions should return `Result<T, DatabaseError>` for consistent error propagation.
//!
//! Example:
//! ```rust
//! fn do_db_work() -> Result<(), DatabaseError> {
//!     // ...
//! }
//! ```
//!
//! ## Integration
//!
//! Errors are convertible to `LedgerError` for unified error handling across the backend.

#[derive(thiserror::Error, Debug)]
/// Errors produced while loading or validating configuration.
///
/// This enum wraps errors from the underlying `config` crate and adds
/// domain-specific validation variants such as an invalid server address.
pub enum DatabaseError {
    /// Connection error
    #[error("Error connecting to the database: {0}")]
    Connection(String),

    /// Wrap underlying sqlx errors
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Database migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    /// Wrap config errors that occur during database initialization
    #[error("Config error: {0}")]
    Config(#[from] crate::config::ConfigError),

    /// Validation errors originating from the DB layer (e.g. constraint violations)
    #[error("Validation: {0}")]
    Validation(String),

    /// Generic catch-all for other database related errors
    #[error("Other database error: {0}")]
    Other(String),
}

