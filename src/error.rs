//! # Ledger Error Module
//!
//! This module provides standardized error handling for the Personal Ledger backend.
//! It uses the `thiserror` crate to define custom error types and implements
//! conversions from common error types used throughout the application.
//!
//! ## Ledger Error Categories
//!
//! - `Grpc`: Errors related to gRPC communication and Tonic framework
//! - `Database`: Errors from database operations (SQLx)
//! - `Config`: Configuration loading and parsing errors
//! - `Io`: File system and I/O operations
//! - `Validation`: Data validation errors
//! - `Authentication`: JWT and authentication-related errors
//! - `Internal`: Internal server errors and unexpected conditions

use crate::telemetry;

/// Result type used across the application.
///
/// This is a convenience alias for functions that return a `Result` with
/// `LedgerError` as the error type. Use `LedgerResult<T>` for clarity in APIs.
///
/// Example:
///
/// ```rust
/// fn do_work() -> LedgerResult<()> {
///     Err(LedgerError::internal("boom"))
/// }
/// ```
pub type LedgerResult<T> = std::result::Result<T, LedgerError>;

/// Central application error type
///
/// `LedgerError` unifies all errors the application can produce so they can
/// be handled and converted in a single place. Use `From`/`into` conversions
/// to convert other error types into `LedgerError` and then convert to a
/// `tonic::Status` when returning errors over gRPC.
///
/// The enum intentionally keeps client-facing gRPC messages generic; full
/// details are logged via `tracing` when converting to `tonic::Status` so
/// operators can diagnose issues without leaking internals to clients.
///
/// Example (converting to gRPC status):
///
/// ```rust
/// let err = LedgerError::validation("bad input");
/// let status: tonic::Status = err.into();
/// assert_eq!(status.code(), tonic::Code::InvalidArgument);
/// ```
#[derive(thiserror::Error, Debug)]
pub enum LedgerError {
    /// For starter, to remove as code matures.
    #[error("Generic error: {0}")]
    Generic(String),

    /// Errors related to gRPC communication and Tonic framework
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    /// Errors from Tonic transport layer
    #[error("Tonic transport error: {0}")]
    TonicTransport(#[from] tonic::transport::Error),

    /// Errors from Tonic reflection service
    #[error("Tonic reflection error: {0}")]
    TonicReflection(#[from] tonic_reflection::server::Error),

    /// Configuration errors from config module
    #[error("Configuration error: {0}")]
    Config(crate::config::ConfigError),

    /// Database errors from database module
    #[error("Database error: {0}")]
    Database(crate::database::DatabaseError),

    /// Configuration loading and parsing errors
    // Note: use the structured `Config(crate::config::ConfigError)` variant
    // for configuration errors. Previous code had a separate
    // `Configuration(String)` variant which caused duplication; we now
    // represent simple configuration messages as
    // `Config(crate::config::ConfigError::Validation(...))`.

    /// File system and I/O operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Network address parsing errors
    #[error("Address parsing error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),

    /// Environment variable errors
    #[error("Environment variable error: {0}")]
    Env(#[from] std::env::VarError),

    /// Data validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// JWT and authentication-related errors
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Internal server errors and unexpected conditions
    #[error("Internal error: {0}")]
    Internal(String),
}

impl LedgerError {
    /// Creates a new configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Config(crate::config::ConfigError::Validation(message.into()))
    }

    /// Creates a new database error
    pub fn database<S: Into<String>>(message: S) -> Self {
        Self::Database(crate::database::DatabaseError::Validation(message.into()))
    }

    /// Creates a new validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation(message.into())
    }

    /// Creates a new authentication error
    pub fn authentication<S: Into<String>>(message: S) -> Self {
        Self::Authentication(message.into())
    }

    /// Creates a new internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }

    /// Creates a new generic error
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Self::Generic(message.into())
    }
}

/// Convert ConfigError to LedgerError
///
/// This allows propagation of structured `ConfigError` values into the
/// central `LedgerError` enum so they can be logged and translated to
/// gRPC statuses in a single place.
impl From<crate::config::ConfigError> for LedgerError {
    fn from(error: crate::config::ConfigError) -> Self {
        LedgerError::Config(error)
    }
}

/// Convert database errors into the central `LedgerError` enum.
///
/// This allows propagation of structured `DatabaseError` values into the
/// central `LedgerError` enum so they can be logged and translated to
/// gRPC statuses in a single place.
impl From<crate::database::DatabaseError> for LedgerError {
    fn from(error: crate::database::DatabaseError) -> Self {
        LedgerError::Database(error)
    }
}

/// Conversion from `TelemetryError` to the application's main error type.
///
/// This implementation allows telemetry errors to be seamlessly integrated
/// with the application's primary error handling system, ensuring consistent
/// error propagation and handling across the codebase.
impl From<telemetry::TelemetryError> for crate::LedgerError {
    fn from(error: telemetry::TelemetryError) -> Self {
        match error {
            telemetry::TelemetryError::SubscriberInit(_) => LedgerError::Internal(error.to_string()),
            telemetry::TelemetryError::EnvFilter(_) => LedgerError::Config(crate::config::ConfigError::Validation(error.to_string())),
            telemetry::TelemetryError::LogTracerInit(_) => LedgerError::Internal(error.to_string()),
            telemetry::TelemetryError::Io(e) => LedgerError::Io(e),
            telemetry::TelemetryError::Config(_) => LedgerError::Config(crate::config::ConfigError::Validation(error.to_string())),
        }
    }
}

/// Convert our custom Error to tonic::Status for gRPC responses
impl From<LedgerError> for tonic::Status {
    fn from(error: LedgerError) -> Self {
        match error {
            LedgerError::Grpc(status) => status,
            LedgerError::Generic(msg) => {
                // Generic errors are internal by nature; log details for operators and return a safe message to clients.
                tracing::error!(%msg, "Generic error");
                tonic::Status::internal("Internal server error")
            }
            LedgerError::TonicTransport(e) => {
                tracing::error!(?e, "Tonic transport error");
                tonic::Status::internal("Transport error occurred")
            }
            LedgerError::TonicReflection(e) => {
                tracing::error!(?e, "Tonic reflection error");
                tonic::Status::internal("Reflection service error")
            }
            LedgerError::Database(e) => {
                tracing::error!(?e, "Database error");
                tonic::Status::internal("Database error")
            }
            LedgerError::Config(e) => {
                // Log structured config errors and return a generic client-facing message
                tracing::error!(?e, "Configuration error");
                tonic::Status::internal("Configuration error")
            }
            LedgerError::Io(e) => {
                tracing::error!(?e, "I/O error");
                tonic::Status::internal("I/O error occurred")
            }
            LedgerError::AddrParse(_) => {
                tonic::Status::invalid_argument("Invalid network address format")
            }
            LedgerError::Env(e) => {
                tracing::error!(?e, "Environment variable error");
                tonic::Status::internal("Environment configuration error")
            }
            LedgerError::Validation(msg) => {
                tonic::Status::invalid_argument(msg)
            }
            LedgerError::Authentication(msg) => {
                tonic::Status::unauthenticated(msg)
            }
            LedgerError::Internal(msg) => {
                tracing::error!(%msg, "Internal server error");
                tonic::Status::internal("Internal server error")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use tonic::{Code, Status};

    #[test]
    fn test_error_creation() {
        let config_err = LedgerError::configuration("Invalid config");
        assert!(matches!(config_err, LedgerError::Config(_)));

        let validation_err = LedgerError::validation("Invalid data");
        assert!(matches!(validation_err, LedgerError::Validation(_)));

        let auth_err = LedgerError::authentication("Invalid token");
        assert!(matches!(auth_err, LedgerError::Authentication(_)));

        let internal_err = LedgerError::internal("Something went wrong");
        assert!(matches!(internal_err, LedgerError::Internal(_)));

        let generic_err = LedgerError::generic("Generic error");
        assert!(matches!(generic_err, LedgerError::Generic(_)));
    }

    #[test]
    fn test_error_creation_with_different_string_types() {
        // Test with &str
        let err1 = LedgerError::configuration("test message");
        assert!(matches!(err1, LedgerError::Config(_)));

        // Test with String
        let err2 = LedgerError::validation("test message".to_string());
        assert!(matches!(err2, LedgerError::Validation(_)));

        // Test with empty string
        let err3 = LedgerError::internal("");
        assert!(matches!(err3, LedgerError::Internal(_)));

        // Test with String for generic
        let err4 = LedgerError::generic("test message".to_string());
        assert!(matches!(err4, LedgerError::Generic(_)));
    }

    #[test]
    fn test_error_variant_coverage() {
        // Test all error variants are covered
        let grpc_err = LedgerError::Grpc(Status::internal("test"));
        assert!(matches!(grpc_err, LedgerError::Grpc(_)));

        // We cannot construct tonic::transport::Error directly because its constructor is private.
        // Instead, we'll skip direct construction in this test.

        // We cannot construct tonic_reflection::server::Error directly because its constructor is private.
        // Instead, we'll skip direct construction in this test.

        let config_err = LedgerError::Config(crate::config::ConfigError::Validation("test".to_string()));
        assert!(matches!(config_err, LedgerError::Config(_)));

    let configuration_err = LedgerError::Config(crate::config::ConfigError::Validation("test".to_string()));
    assert!(matches!(configuration_err, LedgerError::Config(_)));

        let io_err = LedgerError::Io(io::Error::new(io::ErrorKind::NotFound, "test"));
        assert!(matches!(io_err, LedgerError::Io(_)));

        // Test AddrParse variant by parsing an invalid address
        let addr_parse_result: Result<std::net::SocketAddr, std::net::AddrParseError> = "invalid:address:99999".parse();
        if let Err(addr_parse_err) = addr_parse_result {
            let ledger_err: LedgerError = addr_parse_err.into();
            assert!(matches!(ledger_err, LedgerError::AddrParse(_)));
        }

        // Test Env variant by trying to get a non-existent env var
        let env_result = std::env::var("NON_EXISTENT_VAR");
        if let Err(env_err) = env_result {
            let ledger_err: LedgerError = env_err.into();
            assert!(matches!(ledger_err, LedgerError::Env(_)));
        }

        let validation_err = LedgerError::Validation("test".to_string());
        assert!(matches!(validation_err, LedgerError::Validation(_)));

        let auth_err = LedgerError::Authentication("test".to_string());
        assert!(matches!(auth_err, LedgerError::Authentication(_)));

        let internal_err = LedgerError::Internal("test".to_string());
        assert!(matches!(internal_err, LedgerError::Internal(_)));

        let generic_err = LedgerError::Generic("test".to_string());
        assert!(matches!(generic_err, LedgerError::Generic(_)));
    }

    #[test]
    fn test_error_message_formatting() {
        let config_err = LedgerError::configuration("Test config error");
        assert_eq!(format!("{}", config_err), "Configuration error: Invalid configuration: Test config error");

        let validation_err = LedgerError::validation("Test validation error");
        assert_eq!(format!("{}", validation_err), "Validation error: Test validation error");

        let auth_err = LedgerError::authentication("Test auth error");
        assert_eq!(format!("{}", auth_err), "Authentication error: Test auth error");

        let internal_err = LedgerError::internal("Test internal error");
        assert_eq!(format!("{}", internal_err), "Internal error: Test internal error");

        let generic_err = LedgerError::generic("Test generic error");
        assert_eq!(format!("{}", generic_err), "Generic error: Test generic error");
    }

    #[test]
    fn test_error_debug_formatting() {
        let err = LedgerError::configuration("Test error");
        let debug_str = format!("{:?}", err);
        // Config errors are now represented by `LedgerError::Config(ConfigError::Validation(_))`.
        // The debug output will contain the `Config`/`ConfigError` variant name and the message.
        assert!(debug_str.contains("Config"), "debug output should mention Config variant: {}", debug_str);
        assert!(debug_str.contains("Test error"), "debug output should include the original message: {}", debug_str);
    }

    #[test]
    fn test_error_to_status_conversion() {
        let validation_err = LedgerError::validation("Test validation error");
        let status: tonic::Status = validation_err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert_eq!(status.message(), "Test validation error");

        let auth_err = LedgerError::authentication("Test auth error");
        let status: tonic::Status = auth_err.into();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
        assert_eq!(status.message(), "Test auth error");
    }

    #[test]
    fn test_error_to_status_conversion_all_variants() {
        // Test Grpc variant (should return the status as-is)
        let grpc_status = Status::new(Code::NotFound, "Not found");
        let grpc_err = LedgerError::Grpc(grpc_status.clone());
        let converted_status: Status = grpc_err.into();
        assert_eq!(converted_status.code(), Code::NotFound);
        assert_eq!(converted_status.message(), "Not found");

        // Test TonicTransport variant
        // Cannot construct tonic::transport::Error directly, so we skip this test.

        // Test TonicReflection variant
        // Cannot construct tonic_reflection::server::Error directly, so we skip this test.

        // Test Config variant
        let config_err = LedgerError::Config(crate::config::ConfigError::Validation("Config error".to_string()));
        let status: Status = config_err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("Configuration error"));

    // Test Configuration variant (now represented by Config(ConfigError::Validation))
    let configuration_err = LedgerError::Config(crate::config::ConfigError::Validation("Config error".to_string()));
    let status: Status = configuration_err.into();
    assert_eq!(status.code(), Code::Internal);
    assert!(status.message().contains("Configuration error"));

        // Test Io variant
        let io_err = LedgerError::Io(io::Error::new(io::ErrorKind::NotFound, "File not found"));
        let status: Status = io_err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("I/O error"));

        // Test AddrParse variant
        let addr_parse_result: Result<std::net::SocketAddr, std::net::AddrParseError> = "invalid:address:99999".parse();
        if let Err(addr_parse_err) = addr_parse_result {
            let ledger_err: LedgerError = addr_parse_err.into();
            let status: Status = ledger_err.into();
            assert_eq!(status.code(), Code::InvalidArgument);
            assert!(status.message().contains("Invalid network address format"));
        }

        // Test Env variant
        let env_result = std::env::var("NON_EXISTENT_VAR");
        if let Err(env_err) = env_result {
            let ledger_err: LedgerError = env_err.into();
            let status: Status = ledger_err.into();
            assert_eq!(status.code(), Code::Internal);
            assert!(status.message().contains("Environment configuration error"));
        }

        // Test Internal variant
        let internal_err = LedgerError::Internal("Internal error".to_string());
        let status: Status = internal_err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("Internal server error"));

        // Test Generic variant
        let generic_err = LedgerError::Generic("Generic error".to_string());
        let status: Status = generic_err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("Internal server error"));
    }

    #[test]
    fn test_config_error_conversion() {
        let config_validation_err = crate::config::ConfigError::Validation("Test validation".to_string());
        let ledger_err: LedgerError = config_validation_err.into();
        assert!(matches!(ledger_err, LedgerError::Config(_)));

        // Test that it converts to the right status
        let status: Status = ledger_err.into();
        assert_eq!(status.code(), Code::Internal);
    }

    #[test]
    fn test_grpc_status_conversion() {
        let grpc_status = Status::invalid_argument("Invalid argument");
        let ledger_err: LedgerError = grpc_status.into();
        assert!(matches!(ledger_err, LedgerError::Grpc(_)));

        // Test round-trip conversion
        let back_to_status: Status = ledger_err.into();
        assert_eq!(back_to_status.code(), Code::InvalidArgument);
        assert_eq!(back_to_status.message(), "Invalid argument");
    }
    #[test]
    fn test_tonic_transport_error_conversion() {
        // Cannot construct tonic::transport::Error directly, so we skip this test.
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");
        let ledger_err: LedgerError = io_err.into();
        assert!(matches!(ledger_err, LedgerError::Io(_)));

        let status: Status = ledger_err.into();
        assert_eq!(status.code(), Code::Internal);
    }

    #[test]
    fn test_error_display_messages() {
        // Test that error messages are properly formatted
        let validation_err = LedgerError::validation("Field is required");
        assert_eq!(validation_err.to_string(), "Validation error: Field is required");

        let auth_err = LedgerError::authentication("Token expired");
        assert_eq!(auth_err.to_string(), "Authentication error: Token expired");

        let internal_err = LedgerError::internal("Database connection failed");
        assert_eq!(internal_err.to_string(), "Internal error: Database connection failed");

    let config_err = LedgerError::configuration("Invalid port number");
    // The Config variant wraps a ConfigError::Validation which formats as
    // "Invalid configuration: <msg>". The LedgerError::Config display
    // prefixes that with "Configuration error: ", resulting in a nested
    // message. Assert the full produced string here.
    assert_eq!(config_err.to_string(), "Configuration error: Invalid configuration: Invalid port number");
    }

    #[test]
    fn test_error_helper_methods() {
        // Test that helper methods create the correct variants
        let config_err = LedgerError::configuration("test");
        match config_err {
            LedgerError::Config(crate::config::ConfigError::Validation(msg)) => assert_eq!(msg, "test"),
            _ => panic!("Expected Config::Validation variant"),
        }

        let validation_err = LedgerError::validation("test");
        match validation_err {
            LedgerError::Validation(msg) => assert_eq!(msg, "test"),
            _ => panic!("Expected Validation variant"),
        }

        let auth_err = LedgerError::authentication("test");
        match auth_err {
            LedgerError::Authentication(msg) => assert_eq!(msg, "test"),
            _ => panic!("Expected Authentication variant"),
        }

        let internal_err = LedgerError::internal("test");
        match internal_err {
            LedgerError::Internal(msg) => assert_eq!(msg, "test"),
            _ => panic!("Expected Internal variant"),
        }
    }

    #[test]
    fn test_error_with_special_characters() {
        // Test error messages with special characters
        let err = LedgerError::validation("Field 'email' contains invalid characters: <>&\"'");
        assert!(err.to_string().contains("<>&\"'"));

        let err = LedgerError::internal("Error with newline\nand tab\tcharacters");
        assert!(err.to_string().contains("\n"));
        assert!(err.to_string().contains("\t"));
    }

    #[test]
    fn test_error_with_unicode() {
        // Test error messages with Unicode characters
        let err = LedgerError::validation("Erreur de validation: champ requis");
        assert!(err.to_string().contains("Erreur"));

        let err = LedgerError::internal("内部服务器错误");
        assert!(err.to_string().contains("内部服务器错误"));
    }

    #[test]
    fn test_sqlx_database_error_conversion() {
        // sqlx::Error::RowNotFound is a common sqlx error we can construct
        let sqlx_err = sqlx::Error::RowNotFound;

        // Convert to the module's DatabaseError
        let db_err: crate::database::DatabaseError = sqlx_err.into();
        // Convert to LedgerError
        let ledger_err: LedgerError = db_err.into();

        // Ensure it matches the Database variant
        assert!(matches!(ledger_err, LedgerError::Database(_)));
    }

    #[test]
    fn test_database_error_to_status() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let db_err: crate::database::DatabaseError = sqlx_err.into();
        let ledger_err: LedgerError = db_err.into();
        let status: tonic::Status = ledger_err.into();

        // We currently map Database errors to internal server errors for clients
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("Database"));
    }

    #[test]
    fn test_sqlx_error_conversion() {
        let sqlx_err = sqlx::Error::RowNotFound;
        // Convert sqlx::Error into our DatabaseError and then into the central LedgerError
        let db_err: crate::database::DatabaseError = sqlx_err.into();
        let app_err: LedgerError = db_err.into();
        let status: tonic::Status = app_err.into();
        // Database errors are mapped to internal server errors for clients
        assert_eq!(status.code(), tonic::Code::Internal);
    }
}
