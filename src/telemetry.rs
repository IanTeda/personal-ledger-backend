//! # Telemetry Module
//!
//! This module provides centralized logging and telemetry functionality for the Personal Ledger backend.
//! It configures the `tracing` ecosystem with structured logging, environment-based filtering,
//! and proper error handling.
//!
//! ## Features
//!
//! - **Structured Logging**: Consistent log format with contextual information
//! - **Environment-Aware**: Different log levels based on `RUST_LOG` environment variable
//! - **Error Handling**: Comprehensive error types specific to telemetry operations
//! - **Integration**: Seamless integration with the application's main error handling system
//! - **Convenient Aliases**: Short aliases like `TelemetryLevel` for common types
//!
//! ## Actions
//! 
//! - [ ] Tidy up log level and add display trait
//! 
//! ## Usage
//!
//! ### Basic Initialization
//!
//! ```rust
//! use personal_ledger_backend::telemetry::{TelemetryLevel, init};
//!
//! // Initialize with INFO level filter using the convenient alias
//! init(TelemetryLevel::INFO)?;
//!
//! // Now you can use telemetry macros
//! telemetry::info!("Application started successfully");
//! telemetry::debug!("Debug information: {:?}", some_data);
//! ```
//!
//! ### Using the Full Path
//!
//! ```rust
//! use personal_ledger_backend::telemetry;
//! use telemetry::level_filters::LevelFilter;
//!
//! // Initialize with INFO level using the full path
//! telemetry::init(LevelFilter::INFO)?;
//!
//! // Now you can use tracing macros
//! tracing::info!("Application started successfully");
//! tracing::debug!("Debug information: {:?}", some_data);
//! ```
//!
//! ### Environment-Based Configuration
//!
//! ```bash
//! # Set log level via environment variable
//! export RUST_LOG=debug
//! # Application will use DEBUG level instead of default
//! ```
//!
//! ## Error Handling
//!
//! The module provides specific error types for telemetry operations:
//!
//! ```rust
//! use personal_ledger_backend::telemetry::{TelemetryError, TelemetryLevel, init};
//!
//! match init(TelemetryLevel::INFO) {
//!     Ok(()) => tracing::info!("Telemetry initialized"),
//!     Err(TelemetryError::SubscriberInit(msg)) => {
//!         eprintln!("Failed to initialize subscriber: {}", msg);
//!     }
//!     Err(other) => eprintln!("Other telemetry error: {}", other),
//! }
//! ```

use tracing::subscriber::set_global_default;
use tracing_subscriber::{
    layer::SubscriberExt,
    EnvFilter,
};
use tracing_log;

use crate::LedgerResult;

// Re-export serde derives for convenience in this module
use serde::{Serialize, de};

/// Re-export of tracing::level_filters::LevelFilter for convenience.
///
/// This type alias provides a shorter, more convenient name for the tracing LevelFilter
/// while maintaining compatibility with the underlying tracing crate.
///
/// # Examples
///
/// ```rust
/// use personal_ledger_backend::tracing::TrelemetryLevel;
///
/// // Use the convenient alias
/// let level = TrelemetryLevel::INFO;
///
/// // Initialize tracing
/// tracing::init(level)?;
/// ```
pub type TelemetryLevel = tracing::level_filters::LevelFilter;

/// A serde-friendly representation of log levels used in configuration.
//
/// The tracing crate's `LevelFilter` type does not implement `serde::{Deserialize, Serialize}`
/// so we expose a small enum that can be used in configuration files and converted to
/// the runtime `LevelFilter` when initializing telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
pub enum LogLevel {
    OFF,
    ERROR,
    WARN,
    #[default]
    INFO,
    DEBUG,
    TRACE,
}


impl<'de> de::Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "off" => Ok(LogLevel::OFF),
            "error" => Ok(LogLevel::ERROR),
            "warn" | "warning" => Ok(LogLevel::WARN),
            "info" => Ok(LogLevel::INFO),
            "debug" => Ok(LogLevel::DEBUG),
            "trace" => Ok(LogLevel::TRACE),
            other => Err(de::Error::unknown_variant(other, &["off", "error", "warn", "info", "debug", "trace"])),
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(LogLevel::OFF),
            "error" => Ok(LogLevel::ERROR),
            "warn" | "warning" => Ok(LogLevel::WARN),
            "info" => Ok(LogLevel::INFO),
            "debug" => Ok(LogLevel::DEBUG),
            "trace" => Ok(LogLevel::TRACE),
            other => Err(format!("Unknown log level: {}. Valid values are: off, error, warn, info, debug, trace", other)),
        }
    }
}

impl From<LogLevel> for tracing::level_filters::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::OFF => tracing::level_filters::LevelFilter::OFF,
            LogLevel::ERROR => tracing::level_filters::LevelFilter::ERROR,
            LogLevel::WARN => tracing::level_filters::LevelFilter::WARN,
            LogLevel::INFO => tracing::level_filters::LevelFilter::INFO,
            LogLevel::DEBUG => tracing::level_filters::LevelFilter::DEBUG,
            LogLevel::TRACE => tracing::level_filters::LevelFilter::TRACE,
        }
    }
}

/// Telemetry-specific error types for logging and tracing operations.
///
/// This enum provides detailed error classification for telemetry-related failures,
/// allowing for precise error handling and debugging of logging infrastructure issues.
#[derive(thiserror::Error, Debug)]
pub enum TelemetryError {
    /// Errors that occur during tracing subscriber initialization.
    ///
    /// This typically happens when attempting to set the global default subscriber
    /// fails, often due to another subscriber already being registered.
    #[error("Tracing subscriber initialization failed: {0}")]
    SubscriberInit(String),

    /// Errors related to environment filter configuration.
    ///
    /// This occurs when parsing log level directives from environment variables
    /// or configuration strings fails.
    #[error("Environment filter configuration error: {0}")]
    EnvFilter(String),

    /// Errors during log tracer initialization.
    ///
    /// This happens when the `tracing_log` crate fails to initialize,
    /// typically due to logging infrastructure conflicts.
    #[error("Log tracer initialization failed: {0}")]
    LogTracerInit(#[from] tracing_log::log::SetLoggerError),

    /// I/O errors that occur during telemetry operations.
    ///
    /// This includes file system errors when writing logs to files
    /// or other I/O-related telemetry failures.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic telemetry configuration errors.
    ///
    /// This covers configuration-related issues that don't fit
    /// into the more specific error categories above.
    #[error("Telemetry configuration error: {0}")]
    Config(String),
}

/// Helper methods for creating `TelemetryError` variants.
///
/// These methods provide convenient constructors for common error scenarios,
/// allowing for more ergonomic error creation throughout the telemetry module.
impl TelemetryError {
    /// Creates a new subscriber initialization error.
    ///
    /// # Arguments
    ///
    /// * `message` - A description of the initialization failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// let error = TelemetryError::subscriber_init("Global subscriber already set");
    /// ```
    pub fn subscriber_init<S: Into<String>>(message: S) -> Self {
        Self::SubscriberInit(message.into())
    }

    /// Creates a new environment filter configuration error.
    ///
    /// # Arguments
    ///
    /// * `message` - A description of the filter configuration failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// let error = TelemetryError::env_filter("Invalid log level directive");
    /// ```
    pub fn env_filter<S: Into<String>>(message: S) -> Self {
        Self::EnvFilter(message.into())
    }

    /// Creates a new generic configuration error.
    ///
    /// # Arguments
    ///
    /// * `message` - A description of the configuration issue
    ///
    /// # Examples
    ///
    /// ```rust
    /// let error = TelemetryError::config("Missing required configuration");
    /// ```
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config(message.into())
    }
}

/// Initializes the telemetry system with the specified log level.
///
/// This function sets up the complete tracing infrastructure including:
/// - Environment-based log filtering
/// - Console output formatting
/// - Log tracer integration for compatibility with the `log` crate
/// - Global subscriber registration
///
/// # Arguments
///
/// * `log_level` - The default log level to use when no environment variable is set.
///   You can also use the convenient `TelemetryLevel` alias instead of
///   the full `tracing::level_filters::LevelFilter` path.
///
/// # Returns
///
/// Returns `Ok(())` if initialization succeeds, or a `LedgerError` if it fails.
///
/// # Examples
///
/// Using the convenient TelemetryLevel alias:
/// ```rust
/// use personal_ledger_backend::tracing::{TelemetryLevel, init};
///
/// // Initialize with INFO level using the alias
/// init(TelemetryLevel::INFO)?;
/// ```
///
/// Using the full path:
/// ```rust
/// use personal_ledger_backend::tracing::init;
/// use tracing::level_filters::LevelFilter;
///
/// // Initialize with INFO level using the full path
/// init(TelemetryLevel::INFO)?;
/// ```
///
/// # Environment Variables
///
/// The `RUST_LOG` environment variable can override the default log level:
///
/// ```bash
/// export RUST_LOG=debug
/// ```
///
/// # Errors
///
/// This function can fail in several ways:
/// - `TelemetryError::SubscriberInit` - If the global subscriber is already set
/// - `TelemetryError::LogTracerInit` - If log tracer initialization fails
/// - `TelemetryError::EnvFilter` - If environment filter parsing fails
pub fn init(
    tracing_level: LogLevel 
) -> LedgerResult<()> {
    //-- 1. Filter events
    // Set default log level based on configuration
    let default_env_filter = {
        // Convert our serde-friendly LogLevel -> tracing LevelFilter -> Directive
        let default_directive = tracing::level_filters::LevelFilter::from(tracing_level).into();
        EnvFilter::builder()
            .with_default_directive(default_directive)
            .from_env_lossy()
    };

    // Try to use env runtime level, if not present use default
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(default_env_filter);

    // Build event collector for console output
    let console_collector = tracing_subscriber::fmt::layer();

    //-- 2. Build a registry of collectors
    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(console_collector);

    // Convert all log records into tracing events.
    tracing_log::LogTracer::init()
        .map_err(TelemetryError::LogTracerInit)?;

    //-- 3. Initiate tracing
    set_global_default(registry)
        .map_err(|e| TelemetryError::subscriber_init(format!("Failed to set global default subscriber: {}", e)))?;

    Ok(())
}

#[cfg(test)]
/// Test module for telemetry functionality.
///
/// This module contains comprehensive unit tests for all telemetry components,
/// ensuring proper error handling, type conversions, and infrastructure setup.
mod tests {
    use super::*;

    /// Tests basic error creation for all TelemetryError variants.
    #[test]
    fn test_error_creation() {
        let err = TelemetryError::config("Test config error");
        assert!(matches!(err, TelemetryError::Config(_)));

        let err = TelemetryError::subscriber_init("Test subscriber error");
        assert!(matches!(err, TelemetryError::SubscriberInit(_)));

        let err = TelemetryError::env_filter("Test env filter error");
        assert!(matches!(err, TelemetryError::EnvFilter(_)));
    }

    /// Tests the helper methods for creating TelemetryError variants.
    #[test]
    fn test_error_helper_methods() {
        let err = TelemetryError::config("test message");
        match err {
            TelemetryError::Config(msg) => assert_eq!(msg, "test message"),
            _ => panic!("Expected Config variant"),
        }

        let err = TelemetryError::subscriber_init("test message");
        match err {
            TelemetryError::SubscriberInit(msg) => assert_eq!(msg, "test message"),
            _ => panic!("Expected SubscriberInit variant"),
        }

        let err = TelemetryError::env_filter("test message");
        match err {
            TelemetryError::EnvFilter(msg) => assert_eq!(msg, "test message"),
            _ => panic!("Expected EnvFilter variant"),
        }
    }

    /// Tests conversion from TelemetryError to the application's main LedgerError type.
    #[test]
    fn test_error_conversion_to_ledger_error() {
        // Test Config error conversion
        let telemetry_err = TelemetryError::config("Test error");
        let ledger_err: crate::LedgerError = telemetry_err.into();
        assert!(matches!(ledger_err, crate::LedgerError::Config(_)));

        // Test SubscriberInit error conversion
        let telemetry_err = TelemetryError::subscriber_init("Test error");
        let ledger_err: crate::LedgerError = telemetry_err.into();
        assert!(matches!(ledger_err, crate::LedgerError::Internal(_)));

        // Test EnvFilter error conversion
        let telemetry_err = TelemetryError::env_filter("Test error");
        let ledger_err: crate::LedgerError = telemetry_err.into();
        assert!(matches!(ledger_err, crate::LedgerError::Config(_)));
    }

    /// Tests conversion from std::io::Error to TelemetryError and through to LedgerError.
    #[test]
    fn test_io_error_conversion() {
        use std::io::Error;

        let io_err = Error::other("File not found");
        let telemetry_err: TelemetryError = io_err.into();
        assert!(matches!(telemetry_err, TelemetryError::Io(_)));

        // Test conversion through to LedgerError
        let ledger_err: crate::LedgerError = telemetry_err.into();
        assert!(matches!(ledger_err, crate::LedgerError::Io(_)));
    }

    /// Tests that the SetLoggerError type exists and can be referenced.
    ///
    /// Note: Creating actual SetLoggerError instances is complex in tests,
    /// so we verify the type exists and can be used in type annotations.
    #[test]
    fn test_log_tracer_error_conversion() {
        // Verify the error type exists and can be used
        let _error_type_exists = std::any::TypeId::of::<tracing_log::log::SetLoggerError>();
    }

    /// Tests error display message formatting for all variants.
    #[test]
    fn test_error_display_messages() {
        let err = TelemetryError::config("test message");
        let display_msg = format!("{}", err);
        assert!(display_msg.contains("Telemetry configuration error"));
        assert!(display_msg.contains("test message"));

        let err = TelemetryError::subscriber_init("test message");
        let display_msg = format!("{}", err);
        assert!(display_msg.contains("Tracing subscriber initialization failed"));
        assert!(display_msg.contains("test message"));

        let err = TelemetryError::env_filter("test message");
        let display_msg = format!("{}", err);
        assert!(display_msg.contains("Environment filter configuration error"));
        assert!(display_msg.contains("test message"));
    }

    /// Tests error debug formatting.
    #[test]
    fn test_error_debug_formatting() {
        let err = TelemetryError::config("test message");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("test message"));
    }

    /// Tests TelemetryLevel enum values and equality.
    #[test]
    fn test_level_filter_values() {
        // Test that TelemetryLevel values work as expected
        assert_eq!(TelemetryLevel::OFF, tracing::level_filters::LevelFilter::OFF);
        assert_eq!(TelemetryLevel::ERROR, tracing::level_filters::LevelFilter::ERROR);
        assert_eq!(TelemetryLevel::WARN, tracing::level_filters::LevelFilter::WARN);
        assert_eq!(TelemetryLevel::INFO, tracing::level_filters::LevelFilter::INFO);
        assert_eq!(TelemetryLevel::DEBUG, tracing::level_filters::LevelFilter::DEBUG);
        assert_eq!(TelemetryLevel::TRACE, tracing::level_filters::LevelFilter::TRACE);
    }

    /// Tests EnvFilter creation with default directives.
    #[test]
    fn test_env_filter_creation() {
        // Test creating EnvFilter with different directives
        let filter = EnvFilter::builder()
            .with_default_directive(TelemetryLevel::INFO.into())
            .from_env_lossy();

        // The filter should be created successfully and have a non-empty string representation
        assert!(!filter.to_string().is_empty());
    }

    /// Tests environment variable handling for log configuration.
    ///
    /// Note: We can't safely manipulate environment variables in unit tests
    /// as they affect global state, so we test the API with the current environment.
    #[test]
    fn test_env_var_handling() {
        // Test that EnvFilter can be created from current environment
        let result = EnvFilter::try_from_default_env();
        // This may succeed or fail depending on whether RUST_LOG is set
        // We just verify the function doesn't panic
        let _ = result;
    }

    /// Tests conversion from various error types to TelemetryError.
    #[test]
    fn test_telemetry_error_from_conversion() {
        // From std::io::Error
        let io_err = std::io::Error::other("test");
        let telemetry_err: TelemetryError = io_err.into();
        assert!(matches!(telemetry_err, TelemetryError::Io(_)));
    }

    /// Tests that all TelemetryError variants can be created and converted properly.
    ///
    /// This test ensures complete coverage of error variant handling.
    #[test]
    fn test_error_variant_coverage() {
        // Ensure all error variants are covered in tests
        let config_err = TelemetryError::Config("test".to_string());
        let subscriber_err = TelemetryError::SubscriberInit("test".to_string());
        let env_filter_err = TelemetryError::EnvFilter("test".to_string());
        let io_err = TelemetryError::Io(std::io::Error::other("test"));

        // Test that all variants can be created and converted to LedgerError
        let _: crate::LedgerError = config_err.into();
        let _: crate::LedgerError = subscriber_err.into();
        let _: crate::LedgerError = env_filter_err.into();
        let _: crate::LedgerError = io_err.into();
    }

    /// Tests that the TelemetryLevel alias works correctly.
    ///
    /// This test verifies that the TelemetryLevel type alias is equivalent
    /// to tracing::level_filters::LevelFilter and can be used interchangeably.
    #[test]
    fn test_tracing_level_alias() {
        // Test that TelemetryLevel is the same as the original type
        let level1: TelemetryLevel = TelemetryLevel::INFO;
        let level2: tracing::level_filters::LevelFilter = tracing::level_filters::LevelFilter::INFO;

        // They should be equal
        assert_eq!(level1, level2);

        // Test all level variants
        assert_eq!(TelemetryLevel::OFF, tracing::level_filters::LevelFilter::OFF);
        assert_eq!(TelemetryLevel::ERROR, tracing::level_filters::LevelFilter::ERROR);
        assert_eq!(TelemetryLevel::WARN, tracing::level_filters::LevelFilter::WARN);
        assert_eq!(TelemetryLevel::INFO, tracing::level_filters::LevelFilter::INFO);
        assert_eq!(TelemetryLevel::DEBUG, tracing::level_filters::LevelFilter::DEBUG);
        assert_eq!(TelemetryLevel::TRACE, tracing::level_filters::LevelFilter::TRACE);

        // Test that we can use TelemetryLevel in function calls
        // (This would normally call init, but we can't test that easily due to global state)
        let _level_param: TelemetryLevel = TelemetryLevel::DEBUG;
    }

    /// Ensure the serde-friendly `LogLevel` accepts lowercase values when deserializing.
    #[test]
    fn test_log_level_lowercase_deserialize() {
        // Direct enum deserialization from JSON string
        let j = "\"info\"";
        let level: LogLevel = serde_json::from_str(j).expect("should deserialize lowercase 'info'");
        assert_eq!(level, LogLevel::INFO);

        let j = "\"debug\"";
        let level: LogLevel = serde_json::from_str(j).expect("should deserialize lowercase 'debug'");
        assert_eq!(level, LogLevel::DEBUG);

        let j = "\"trace\"";
        let level: LogLevel = serde_json::from_str(j).expect("should deserialize lowercase 'trace'");
        assert_eq!(level, LogLevel::TRACE);

        // Deserialize via a small struct to simulate config section
        #[derive(serde::Deserialize)]
        struct S {
            level: LogLevel,
        }

        let s = serde_json::from_str::<S>("{\"level\":\"warn\"}").expect("struct should deserialize");
        assert_eq!(s.level, LogLevel::WARN);
    }
}