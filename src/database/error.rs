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
//! - `NotFound`: Resource not found errors
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

/// Result type alias used across database modules.
///
/// Use `DatabaseResult<T>` for functions that return `T` or a `DatabaseError`.
/// This keeps signatures concise and makes it clear the function is database-related.
///
/// Example:
///
/// ```rust
/// fn get_categories() -> DatabaseResult<Vec<Category>> {
///     // ...
/// }
/// ```
pub type DatabaseResult<T> = std::result::Result<T, DatabaseError>;

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

    /// Resource not found errors
    #[error("Not found: {0}")]
    NotFound(String),

    /// Generic catch-all for other database related errors
    #[error("Other database error: {0}")]
    Other(String),
}

impl PartialEq for DatabaseError {
    fn eq(&self, other: &Self) -> bool {
        // Compare by their Display representation to avoid requiring PartialEq on wrapped types
        format!("{}", self) == format!("{}", other)
    }
}

impl Eq for DatabaseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_result_type_alias() {
        // Test that DatabaseResult<T> is equivalent to Result<T, DatabaseError>
        let ok_result: DatabaseResult<i32> = Ok(42);
        assert_eq!(ok_result, Ok(42));

        let err_result: DatabaseResult<i32> = Err(DatabaseError::Validation("test".to_string()));
        assert!(err_result.is_err());
        assert!(matches!(err_result, Err(DatabaseError::Validation(_))));
    }

    #[test]
    fn test_database_error_variants() {
        // Test Connection variant
        let conn_err = DatabaseError::Connection("connection failed".to_string());
        assert!(matches!(conn_err, DatabaseError::Connection(_)));

        // Test Sqlx variant (via From)
        let sqlx_err = sqlx::Error::RowNotFound;
        let db_err: DatabaseError = sqlx_err.into();
        assert!(matches!(db_err, DatabaseError::Sqlx(_)));

        // Test Migration variant (via From)
        let migrate_err = sqlx::migrate::MigrateError::Execute(sqlx::Error::RowNotFound);
        let db_err: DatabaseError = migrate_err.into();
        assert!(matches!(db_err, DatabaseError::Migration(_)));

        // Test Config variant (via From)
        let config_err = crate::config::ConfigError::Validation("config error".to_string());
        let db_err: DatabaseError = config_err.into();
        assert!(matches!(db_err, DatabaseError::Config(_)));

        // Test Validation variant
        let val_err = DatabaseError::Validation("validation failed".to_string());
        assert!(matches!(val_err, DatabaseError::Validation(_)));

        // Test NotFound variant
        let not_found_err = DatabaseError::NotFound("record not found".to_string());
        assert!(matches!(not_found_err, DatabaseError::NotFound(_)));

        // Test Other variant
        let other_err = DatabaseError::Other("other error".to_string());
        assert!(matches!(other_err, DatabaseError::Other(_)));
    }

    #[test]
    fn test_database_error_display() {
        let conn_err = DatabaseError::Connection("test connection".to_string());
        assert_eq!(format!("{}", conn_err), "Error connecting to the database: test connection");

        let sqlx_err = DatabaseError::Sqlx(sqlx::Error::RowNotFound);
        assert!(format!("{}", sqlx_err).contains("Database error:"));

        let migrate_err = DatabaseError::Migration(sqlx::migrate::MigrateError::Execute(sqlx::Error::RowNotFound));
        assert!(format!("{}", migrate_err).contains("Database migration error:"));

        let config_err = DatabaseError::Config(crate::config::ConfigError::Validation("test config".to_string()));
        assert!(format!("{}", config_err).contains("Config error:"));

        let val_err = DatabaseError::Validation("test validation".to_string());
        assert_eq!(format!("{}", val_err), "Validation: test validation");

        let not_found_err = DatabaseError::NotFound("test record".to_string());
        assert_eq!(format!("{}", not_found_err), "Not found: test record");

        let other_err = DatabaseError::Other("test other".to_string());
        assert_eq!(format!("{}", other_err), "Other database error: test other");
    }

    #[test]
    fn test_database_error_debug() {
        let err = DatabaseError::Connection("debug test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Connection"));
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_database_error_from_conversions() {
        // Test From<sqlx::Error>
        let sqlx_err = sqlx::Error::RowNotFound;
        let db_err: DatabaseError = sqlx_err.into();
        assert!(matches!(db_err, DatabaseError::Sqlx(_)));

        // Test From<sqlx::migrate::MigrateError>
        let migrate_err = sqlx::migrate::MigrateError::Execute(sqlx::Error::RowNotFound);
        let db_err: DatabaseError = migrate_err.into();
        assert!(matches!(db_err, DatabaseError::Migration(_)));

        // Test From<crate::config::ConfigError>
        let config_err = crate::config::ConfigError::Validation("test".to_string());
        let db_err: DatabaseError = config_err.into();
        assert!(matches!(db_err, DatabaseError::Config(_)));
    }

    #[test]
    fn test_database_error_to_ledger_error_conversion() {
        use crate::LedgerError;

        let conn_err = DatabaseError::Connection("test".to_string());
        let ledger_err: LedgerError = conn_err.into();
        assert!(matches!(ledger_err, LedgerError::Database(_)));

        let val_err = DatabaseError::Validation("test".to_string());
        let ledger_err: LedgerError = val_err.into();
        assert!(matches!(ledger_err, LedgerError::Database(_)));
    }

    #[test]
    fn test_database_error_edge_cases() {
        // Test with empty strings
        let empty_conn = DatabaseError::Connection("".to_string());
        assert_eq!(format!("{}", empty_conn), "Error connecting to the database: ");

        // Test with special characters
        let special = DatabaseError::Validation("test\nwith\ttabs".to_string());
        assert_eq!(format!("{}", special), "Validation: test\nwith\ttabs");

        // Test with unicode
        let unicode = DatabaseError::Other("测试错误".to_string());
        assert_eq!(format!("{}", unicode), "Other database error: 测试错误");
    }
}

