
//! Top-level configuration module.
//!
//! This module organises all configuration-related submodules and re-exports
//! the public domain types used throughout the application: `LedgerConfig`,
//! `DatabaseConfig`, `ServerConfig` and the `ConfigError` type used for
//! configuration loading and validation errors.

mod database;
/// Database configuration and helpers.
pub use database::DatabaseConfig;
pub use database::ConnectionPool;
pub use database::DbEngine;
pub use database::PostgresConfig;

mod error;
/// Configuration loading and validation errors.
pub use error::ConfigError;

mod ledger;
/// The top-level application configuration type.
pub use ledger::LedgerConfig;

mod server;
/// Server-specific configuration values and defaults.
pub use server::ServerConfig;
