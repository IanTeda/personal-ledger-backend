//! Top-level configuration module.
//!
//! This module organises all configuration-related submodules and re-exports
//! the public domain types used throughout the application: `LedgerConfig`,
//! `ServerConfig`, and the `ConfigError` type used for configuration loading
//! and validation errors.
//!
//! ## Structure
//!
//! - [`error`] - Configuration error types
//! - [`ledger`] - Top-level application configuration
//! - [`server`] - Server networking, TLS, and database path configuration
//!
//! ## Database Configuration
//!
//! Database configuration is handled through the `ServerConfig.database_path` field,
//! which specifies the SQLite database file path. The `ServerConfig::database_url()`
//! method constructs the appropriate SQLx connection URL for SQLite databases.

mod error;
/// Configuration loading and validation errors.
pub use error::{ConfigResult, ConfigError};

mod ledger;
/// The top-level application configuration type.
pub use ledger::LedgerConfig;

mod server;
/// Server-specific configuration values and defaults.
pub use server::ServerConfig;
