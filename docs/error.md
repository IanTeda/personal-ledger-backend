# Error Handling

This document describes the error handling system implemented in the Personal Ledger backend, which provides standardized error management across the entire application.

## Overview

The error handling system is built around a custom `LedgerError` enum that centralises all error types used throughout the application. This approach ensures consistent error handling, proper error propagation, and appropriate HTTP/gRPC status code mapping.

## Core Components

### LedgerError Enum

The `LedgerError` enum is the central error type for the application, defined in `src/error.rs`. It uses the `thiserror` crate for ergonomic error handling and automatic error message formatting.

### LedgerResult Type Alias

For convenience, a `LedgerResult<T>` type alias is provided:

```rust
pub type LedgerResult<T> = std::result::Result<T, LedgerError>;
```

## Error Categories

### Grpc Errors
- **Purpose**: Handle gRPC communication and Tonic framework errors
- **Source**: `tonic::Status`
- **Usage**: Automatically converted from Tonic status codes
- **gRPC Mapping**: Passed through unchanged

### TonicTransport Errors
- **Purpose**: Handle transport layer errors (connection, binding, etc.)
- **Source**: `tonic::transport::Error`
- **Usage**: Automatically converted from transport operations
- **gRPC Mapping**: Maps to `INTERNAL` status

### Configuration Errors
- **Purpose**: Handle configuration file loading and parsing issues
- **Source**: Custom string messages
- **Usage**: `LedgerError::config("message")` or `LedgerError::Config("message".to_string())`
- **gRPC Mapping**: Maps to `INTERNAL` status

### I/O Errors
- **Purpose**: Handle file system and I/O operation failures
- **Source**: `std::io::Error`
- **Usage**: Automatically converted from I/O operations
- **gRPC Mapping**: Maps to `INTERNAL` status

### Validation Errors
- **Purpose**: Handle data validation failures
- **Source**: Custom string messages
- **Usage**: `LedgerError::validation("message")` or `LedgerError::Validation("message".to_string())`
- **gRPC Mapping**: Maps to `INVALID_ARGUMENT` status

### Authentication Errors
- **Purpose**: Handle JWT and authentication-related failures
- **Source**: Custom string messages
- **Usage**: `LedgerError::authentication("message")` or `LedgerError::Authentication("message".to_string())`
- **gRPC Mapping**: Maps to `UNAUTHENTICATED` status

### Internal Errors
- **Purpose**: Handle unexpected server errors and edge cases
- **Source**: Custom string messages
- **Usage**: `LedgerError::internal("message")` or `LedgerError::Internal("message".to_string())`
- **gRPC Mapping**: Maps to `INTERNAL` status

## Usage Examples

### Basic Error Creation

```rust
use personal_ledger_backend::error::{LedgerError, LedgerResult};

// Create errors using constructor methods
let config_err = LedgerError::config("Configuration file not found");
let validation_err = LedgerError::validation("Invalid email format");
let auth_err = LedgerError::authentication("Token expired");

// Create errors directly
let internal_err = LedgerError::Internal("Unexpected database state".to_string());
```

### Error Propagation

```rust
use personal_ledger_backend::error::LedgerResult;

async fn process_user_data(data: UserData) -> LedgerResult<ProcessedData> {
    // Validation error
    if data.email.is_empty() {
        return Err(LedgerError::validation("Email is required"));
    }

    // I/O operation with automatic error conversion
    let file_content = std::fs::read_to_string("config.json")?;

    // Configuration parsing
    let config: Config = serde_json::from_str(&file_content)
        .map_err(|_| LedgerError::config("Invalid configuration format"))?;

    Ok(processed_data)
}
```

## gRPC Status Code Mapping

The `LedgerError` enum implements `From<LedgerError> for tonic::Status`, ensuring appropriate HTTP status codes are returned to clients:

| Error Variant | gRPC Status Code | HTTP Status |
|---------------|------------------|-------------|
| `Grpc` | Original status | Varies |
| `TonicTransport` | `INTERNAL` | 500 |
| `Config` | `INTERNAL` | 500 |
| `Io` | `INTERNAL` | 500 |
| `Validation` | `INVALID_ARGUMENT` | 400 |
| `Authentication` | `UNAUTHENTICATED` | 401 |
| `Internal` | `INTERNAL` | 500 |

## Best Practices

### 1. Use Appropriate Error Types
Choose the most specific error variant for your use case:

```rust
// ✅ Good: Specific validation error
if !is_valid_email(email) {
    return Err(LedgerError::validation("Invalid email format"));
}

// ❌ Avoid: Generic internal error for validation
if !is_valid_email(email) {
    return Err(LedgerError::internal("Email validation failed"));
}
```

### 2. Provide Descriptive Messages
Include context in error messages:

```rust
// ✅ Good: Descriptive message
return Err(LedgerError::config(format!("Failed to load config from {}", path)));

// ❌ Avoid: Vague message
return Err(LedgerError::config("Config error"));
```

### 3. Use Constructor Methods
Prefer constructor methods for consistency:

```rust
// ✅ Good: Constructor method
let err = LedgerError::validation("Field is required");

// ✅ Also good: Direct construction
let err = LedgerError::Validation("Field is required".to_string());
```

### 4. Handle Errors at Appropriate Boundaries
Convert errors at service boundaries:

```rust
// In repository layer - use specific errors
async fn find_user(&self, id: Uuid) -> LedgerResult<User> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => LedgerError::validation(format!("User {} not found", id)),
            _ => LedgerError::internal(format!("Database error: {}", e)),
        })
}
```

## Testing

The error module includes comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        // Test constructor methods
    }

    #[test]
    fn test_error_to_status_conversion() {
        // Test gRPC status mapping
    }
}
```

Run tests with:
```bash
cargo test error
```

### Custom Error Variants
Add new error variants as needed:

```rust
/// Business logic errors
#[error("Business logic error: {0}")]
BusinessLogic(String),
```

## Module Organization

The error module is organized as follows:

- `src/error.rs`: Main error definitions and implementations
- `src/lib.rs`: Public exports (`LedgerError`, `LedgerResult`)
- `docs/error.md`: This documentation

## Dependencies

- `thiserror`: For ergonomic error definitions and formatting
- `tonic`: For gRPC status code integration

## Related Documentation

- [gRPC Status Codes](https://grpc.io/docs/guides/status-codes/)
- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [thiserror Documentation](https://docs.rs/thiserror/latest/thiserror/)</content>
<parameter name="filePath">/workspaces/personal-ledger-backend/docs/error.md
