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

/// Result type alias for convenience
pub type LedgerResult<T> = std::result::Result<T, LedgerError>;

/// Custom error type for the Personal Ledger backend application.
///
/// This enum centralizes all error types used throughout the application,
/// providing consistent error handling and conversion to appropriate HTTP/gRPC status codes.
#[derive(thiserror::Error, Debug)]
pub enum LedgerError {
    /// Errors related to gRPC communication and Tonic framework
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    /// Errors from Tonic transport layer
    #[error("Tonic transport error: {0}")]
    TonicTransport(#[from] tonic::transport::Error),

    /// Configuration errors from config module
    #[error("Configuration error: {0}")]
    Config(crate::config::ConfigError),

    /// Errors from database operations
    // #[error("Database error: {0}")]
    // Database(#[from] sqlx::Error),

    /// Configuration loading and parsing errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// File system and I/O operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

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
        Self::Configuration(message.into())
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
}

/// Convert ConfigError to LedgerError
impl From<crate::config::ConfigError> for LedgerError {
    fn from(error: crate::config::ConfigError) -> Self {
        LedgerError::Config(error)
    }
}

/// Convert our custom Error to tonic::Status for gRPC responses
impl From<LedgerError> for tonic::Status {
    fn from(error: LedgerError) -> Self {
        match error {
            LedgerError::Grpc(status) => status,
            LedgerError::TonicTransport(_) => {
                tonic::Status::internal("Transport error occurred")
            }
            // Error::Database(sqlx::Error::RowNotFound) => {
            //     tonic::Status::not_found("Resource not found")
            // }
            // Error::Database(_) => {
            //     tonic::Status::internal("Database error occurred")
            // }
            LedgerError::Config(_) => {
                tonic::Status::internal("Configuration error")
            }
            LedgerError::Configuration(_) => {
                tonic::Status::internal("Configuration error")
            }
            LedgerError::Io(_) => {
                tonic::Status::internal("I/O error occurred")
            }
            LedgerError::Validation(msg) => {
                tonic::Status::invalid_argument(msg)
            }
            LedgerError::Authentication(msg) => {
                tonic::Status::unauthenticated(msg)
            }
            LedgerError::Internal(_) => {
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
        assert!(matches!(config_err, LedgerError::Configuration(_)));

        let validation_err = LedgerError::validation("Invalid data");
        assert!(matches!(validation_err, LedgerError::Validation(_)));

        let auth_err = LedgerError::authentication("Invalid token");
        assert!(matches!(auth_err, LedgerError::Authentication(_)));

        let internal_err = LedgerError::internal("Something went wrong");
        assert!(matches!(internal_err, LedgerError::Internal(_)));
    }

    #[test]
    fn test_error_creation_with_different_string_types() {
        // Test with &str
        let err1 = LedgerError::configuration("test message");
        assert!(matches!(err1, LedgerError::Configuration(_)));

        // Test with String
        let err2 = LedgerError::validation("test message".to_string());
        assert!(matches!(err2, LedgerError::Validation(_)));

        // Test with empty string
        let err3 = LedgerError::internal("");
        assert!(matches!(err3, LedgerError::Internal(_)));
    }

    #[test]
    fn test_error_variant_coverage() {
        // Test all error variants are covered
        let grpc_err = LedgerError::Grpc(Status::internal("test"));
        assert!(matches!(grpc_err, LedgerError::Grpc(_)));

        // We cannot construct tonic::transport::Error directly because its constructor is private.
        // Instead, we'll skip direct construction in this test.

        let config_err = LedgerError::Config(crate::config::ConfigError::Validation("test".to_string()));
        assert!(matches!(config_err, LedgerError::Config(_)));

        let configuration_err = LedgerError::Configuration("test".to_string());
        assert!(matches!(configuration_err, LedgerError::Configuration(_)));

        let io_err = LedgerError::Io(io::Error::new(io::ErrorKind::NotFound, "test"));
        assert!(matches!(io_err, LedgerError::Io(_)));

        let validation_err = LedgerError::Validation("test".to_string());
        assert!(matches!(validation_err, LedgerError::Validation(_)));

        let auth_err = LedgerError::Authentication("test".to_string());
        assert!(matches!(auth_err, LedgerError::Authentication(_)));

        let internal_err = LedgerError::Internal("test".to_string());
        assert!(matches!(internal_err, LedgerError::Internal(_)));
    }

    #[test]
    fn test_error_message_formatting() {
        let config_err = LedgerError::configuration("Test config error");
        assert_eq!(format!("{}", config_err), "Configuration error: Test config error");

        let validation_err = LedgerError::validation("Test validation error");
        assert_eq!(format!("{}", validation_err), "Validation error: Test validation error");

        let auth_err = LedgerError::authentication("Test auth error");
        assert_eq!(format!("{}", auth_err), "Authentication error: Test auth error");

        let internal_err = LedgerError::internal("Test internal error");
        assert_eq!(format!("{}", internal_err), "Internal error: Test internal error");
    }

    #[test]
    fn test_error_debug_formatting() {
        let err = LedgerError::configuration("Test error");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Configuration"));
        assert!(debug_str.contains("Test error"));
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

        // Test Config variant
        let config_err = LedgerError::Config(crate::config::ConfigError::Validation("Config error".to_string()));
        let status: Status = config_err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("Configuration error"));

        // Test Configuration variant
        let configuration_err = LedgerError::Configuration("Config error".to_string());
        let status: Status = configuration_err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("Configuration error"));

        // Test Io variant
        let io_err = LedgerError::Io(io::Error::new(io::ErrorKind::NotFound, "File not found"));
        let status: Status = io_err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("I/O error"));

        // Test Internal variant
        let internal_err = LedgerError::Internal("Internal error".to_string());
        let status: Status = internal_err.into();
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
        assert_eq!(config_err.to_string(), "Configuration error: Invalid port number");
    }

    #[test]
    fn test_error_helper_methods() {
        // Test that helper methods create the correct variants
        let config_err = LedgerError::configuration("test");
        match config_err {
            LedgerError::Configuration(msg) => assert_eq!(msg, "test"),
            _ => panic!("Expected Configuration variant"),
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

    // #[test]
    // fn test_sqlx_error_conversion() {
    //     let sqlx_err = sqlx::Error::RowNotFound;
    //     let app_err: Error = sqlx_err.into();
    //     let status: tonic::Status = app_err.into();
    //     assert_eq!(status.code(), tonic::Code::NotFound);
    // }
}
