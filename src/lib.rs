//! # Personal Ledger Backend Library
//!
//! This crate provides the core library for the Personal Ledger backend server.
//! It exposes the gRPC API, domain logic, and shared utilities for use by the
//! main server binary and integration tests.
//!
//! ## Modules
//!
//! - [`config`]: Configuration loading and management.
//! - [`database`]: Database connection and query utilities.
//! - [`error`]: Standardized error types and handling utilities.
//! - [`rpc`]: Generated and re-exported gRPC types and services for the API.
//! - [`tracing`]: Comprehensive logging and tracing functionality.
//!
//! The library is kept minimal: runtime wiring and integration tests live in the
//! binary crate (`src/main.rs`), while `src/lib.rs` provides the composable
//! pieces used across tests and examples.

/// Configuration module and types.
pub mod config;

/// The top-level configuration type used by the binary to bootstrap the app.
pub use config::LedgerConfig;

/// Database configuration and helpers.
pub mod database;

/// Error types and handling utilities.
mod error;

// Re-Exports for cleaner code
/// Common error and result types used across the crate and binary.
pub use error::{LedgerError, LedgerResult};

/// gRPC API module and types.
pub mod rpc;

/// Tracing and logging utilities.
pub mod telemetry;

