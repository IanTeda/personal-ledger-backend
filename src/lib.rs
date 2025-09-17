//-- .src/lib.rs

//! # Personal Ledger Backend Library
//!
//! This crate provides the core library for the Personal Ledger backend server.
//! It exposes the gRPC API, domain logic, and shared utilities for use by the
//! main server binary and integration tests.
//!
//! ## Modules
//!
//! - [`rpc`]: Generated and re-exported gRPC types and services for the API.
//! - [`error`]: Standardized error types and handling utilities.
//!
mod error;
pub mod rpc;

pub use error::{LedgerError, LedgerResult};