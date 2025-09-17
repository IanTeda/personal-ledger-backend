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

    /// Errors from database operations
    // #[error("Database error: {0}")]
    // Database(#[from] sqlx::Error),

    /// Configuration loading and parsing errors
    #[error("Configuration error: {0}")]
    Config(String),

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
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config(message.into())
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

    #[test]
    fn test_error_creation() {
        let config_err = LedgerError::config("Invalid config");
        assert!(matches!(config_err, LedgerError::Config(_)));

        let validation_err = LedgerError::validation("Invalid data");
        assert!(matches!(validation_err, LedgerError::Validation(_)));

        let auth_err = LedgerError::authentication("Invalid token");
        assert!(matches!(auth_err, LedgerError::Authentication(_)));

        let internal_err = LedgerError::internal("Something went wrong");
        assert!(matches!(internal_err, LedgerError::Internal(_)));
    }

    #[test]
    fn test_error_to_status_conversion() {
        let validation_err = LedgerError::validation("Test validation error");
        let status: tonic::Status = validation_err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);

        let auth_err = LedgerError::authentication("Test auth error");
        let status: tonic::Status = auth_err.into();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    // #[test]
    // fn test_sqlx_error_conversion() {
    //     let sqlx_err = sqlx::Error::RowNotFound;
    //     let app_err: Error = sqlx_err.into();
    //     let status: tonic::Status = app_err.into();
    //     assert_eq!(status.code(), tonic::Code::NotFound);
    // }
}
