//! Top-level configuration module.
//!
//! This module organises all configuration-related submodules and re-exports
//! the public domain types used throughout the application: `LedgerConfig`,
//! `DatabaseConfig`, `ServerConfig`, `ConnectionPool`, and the `ConfigError`
//! type used for configuration loading and validation errors.
//!
//! ## Structure
//!
//! - [`database`] - Database connection configuration and URL building
//! - [`error`] - Configuration error types
//! - [`ledger`] - Top-level application configuration
//! - [`server`] - Server networking and TLS configuration
//!
//! ## Connection Pool
//!
//! The [`ConnectionPool`] type wraps a SQLx connection pool with database engine
//! type information, enabling runtime type checking and appropriate handling for
//! both SQLite and PostgreSQL backends.

use crate::domain::DbEngine;

mod database;
/// Database configuration and helpers.
pub use database::DatabaseConfig;
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
