//! # Integration Test Helpers
//!
//! This module provides shared utilities and helper functions for integration tests.
//! These utilities help set up test environments, manage test servers and clients,
//! and generate mock data for comprehensive testing of the Personal Ledger backend.
//!
//! ## Modules
//! - `spawn_server`: Utilities for spawning test gRPC servers
//! - `spawn_client`: Utilities for creating test gRPC clients
//! - `mocks`: Mock data generation functions for domain types
//!
//! ## Usage
//! Import helpers in your integration test files as needed:
//! ```rust
//! use tests::api::helpers::{SpawnTonicServer, mock_row_id};
//! ```
//!
//! ## Testing Guidelines
//! These helpers follow the project's testing principles:
//! - Provide consistent test setup and teardown
//! - Enable isolated, reproducible test scenarios
//! - Support both unit and integration testing patterns

mod spawn_server;
mod spawn_client;
mod mocks;

pub use spawn_server::SpawnTonicServer;
pub use spawn_client::SpawnTonicClient;
pub use mocks::*;