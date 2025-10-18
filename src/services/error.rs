//! # Service Error Module
//!
//! This module defines comprehensive error types for service layer operations
//! in the Personal Ledger backend. It provides structured error handling for
//! database operations, validation, authentication, and business logic errors.
//!
//! ## Error Categories
//!
//! - **Database Errors**: SQLx database operation failures
//! - **Validation Errors**: Domain type parsing and validation failures
//! - **Not Found Errors**: Resource not found scenarios
//! - **Authentication Errors**: Auth and authorization failures
//! - **Business Logic Errors**: Application-specific business rule violations
//! - **External Service Errors**: Failures in external API calls
//!
//! ## Usage
//!
//! ```rust
//! use personal_ledger_backend::services::ServiceError;
//!
//! // Convert various error types to ServiceError
//! let db_error = sqlx::Error::RowNotFound;
//! let service_error = ServiceError::from(db_error);
//!
//! // ServiceError implements std::error::Error and thiserror::Error
//! assert!(service_error.source().is_some());
//! ```

use chrono;
use sqlx;
use uuid;

/// Comprehensive error type for service layer operations.
///
/// This enum wraps various error types that can occur during service operations,
/// providing consistent error handling and user-friendly error messages.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// Database operation errors (SQLx errors).
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Domain validation errors (parsing domain types).
    #[error("Validation error: {0}")]
    Validation(String),

    /// Resource not found errors.
    #[error("{resource_type} with {field} '{value}' not found")]
    NotFound {
        resource_type: String,
        field: String,
        value: String,
    },

    /// Authentication and authorization errors.
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Business logic violation errors.
    #[error("Business logic error: {0}")]
    BusinessLogic(String),

    /// External service call errors.
    #[error("External service error: {0}")]
    ExternalService(String),

    /// Configuration errors.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Unexpected internal errors.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl ServiceError {
    /// Create a validation error from a message.
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation(message.into())
    }

    /// Create a not found error for a specific resource.
    pub fn not_found(resource_type: &str, field: &str, value: &str) -> Self {
        Self::NotFound {
            resource_type: resource_type.to_string(),
            field: field.to_string(),
            value: value.to_string(),
        }
    }

    /// Create an authentication error from a message.
    pub fn authentication<S: Into<String>>(message: S) -> Self {
        Self::Authentication(message.into())
    }

    /// Create a business logic error from a message.
    pub fn business_logic<S: Into<String>>(message: S) -> Self {
        Self::BusinessLogic(message.into())
    }

    /// Create an external service error from a message.
    pub fn external_service<S: Into<String>>(message: S) -> Self {
        Self::ExternalService(message.into())
    }

    /// Create a configuration error from a message.
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration(message.into())
    }

    /// Create an internal error from a message.
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }

    /// Check if this is a not found error.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    /// Check if this is a validation error.
    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Validation(_))
    }

    /// Check if this is an authentication error.
    pub fn is_authentication(&self) -> bool {
        matches!(self, Self::Authentication(_))
    }

    /// Get the HTTP status code that should be returned for this error.
    pub fn http_status_code(&self) -> u16 {
        match self {
            Self::Database(sqlx::Error::RowNotFound) => 404,
            Self::Database(_) => 500,
            Self::Validation(_) => 400,
            Self::NotFound { .. } => 404,
            Self::Authentication(_) => 401,
            Self::BusinessLogic(_) => 422,
            Self::ExternalService(_) => 502,
            Self::Configuration(_) => 500,
            Self::Internal(_) => 500,
        }
    }
}

/// Convert UUID parsing errors to ServiceError.
impl From<uuid::Error> for ServiceError {
    fn from(err: uuid::Error) -> Self {
        Self::Validation(format!("Invalid UUID: {}", err))
    }
}

/// Convert chrono parsing errors to ServiceError.
impl From<chrono::ParseError> for ServiceError {
    fn from(err: chrono::ParseError) -> Self {
        Self::Validation(format!("Invalid timestamp: {}", err))
    }
}

/// Convert URL slug parsing errors to ServiceError.
impl From<crate::domain::UrlSlugError> for ServiceError {
    fn from(err: crate::domain::UrlSlugError) -> Self {
        Self::Validation(format!("Invalid URL slug: {}", err))
    }
}

/// Convert hex color parsing errors to ServiceError.
impl From<crate::domain::HexColorError> for ServiceError {
    fn from(err: crate::domain::HexColorError) -> Self {
        Self::Validation(format!("Invalid hex color: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = ServiceError::not_found("Category", "id", "123");
        assert_eq!(error.to_string(), "Category with id '123' not found");

        let error = ServiceError::validation("Invalid name");
        assert_eq!(error.to_string(), "Validation error: Invalid name");

        let error = ServiceError::authentication("Invalid token");
        assert_eq!(error.to_string(), "Authentication error: Invalid token");
    }

    #[test]
    fn test_error_classification() {
        let not_found = ServiceError::not_found("User", "email", "test@example.com");
        assert!(not_found.is_not_found());
        assert!(!not_found.is_validation());
        assert!(!not_found.is_authentication());

        let validation = ServiceError::validation("Invalid input");
        assert!(!validation.is_not_found());
        assert!(validation.is_validation());
        assert!(!validation.is_authentication());

        let auth = ServiceError::authentication("Unauthorized");
        assert!(!auth.is_not_found());
        assert!(!auth.is_validation());
        assert!(auth.is_authentication());
    }

    #[test]
    fn test_http_status_codes() {
        assert_eq!(ServiceError::not_found("Test", "id", "1").http_status_code(), 404);
        assert_eq!(ServiceError::validation("test").http_status_code(), 400);
        assert_eq!(ServiceError::authentication("test").http_status_code(), 401);
        assert_eq!(ServiceError::business_logic("test").http_status_code(), 422);
        assert_eq!(ServiceError::external_service("test").http_status_code(), 502);
        assert_eq!(ServiceError::internal("test").http_status_code(), 500);
    }

    #[test]
    fn test_from_sqlx_error() {
        let sqlx_error = sqlx::Error::RowNotFound;
        let service_error: ServiceError = sqlx_error.into();
        assert!(matches!(service_error, ServiceError::Database(_)));
        assert_eq!(service_error.http_status_code(), 404);
    }

    #[test]
    fn test_convenience_methods() {
        let error = ServiceError::validation("test message");
        assert!(matches!(error, ServiceError::Validation(_)));

        let error = ServiceError::not_found("Category", "id", "123");
        assert!(matches!(error, ServiceError::NotFound { .. }));

        let error = ServiceError::authentication("test");
        assert!(matches!(error, ServiceError::Authentication(_)));

        let error = ServiceError::business_logic("test");
        assert!(matches!(error, ServiceError::BusinessLogic(_)));

        let error = ServiceError::external_service("test");
        assert!(matches!(error, ServiceError::ExternalService(_)));

        let error = ServiceError::configuration("test");
        assert!(matches!(error, ServiceError::Configuration(_)));

        let error = ServiceError::internal("test");
        assert!(matches!(error, ServiceError::Internal(_)));
    }
}