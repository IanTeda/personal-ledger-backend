//! # Server Module
//!
//! This module provides the core server infrastructure for the Personal Ledger backend,
//! offering high-level abstractions for running gRPC servers with advanced lifecycle management.
//!
//! ## Overview
//!
//! The server module is designed to simplify gRPC server setup and management while providing
//! powerful features for both production deployment and integration testing. It abstracts away
//! the complexities of service composition, network binding, and health management.
//!
//! ## Architecture
//!
//! The module follows a layered architecture:
//!
//! - **`Router`**: Composes and manages gRPC services (reflection, health, utilities)
//! - **`TonicServer`**: High-level server abstraction combining Router with network binding
//! - **TLS Support**: Secure communication capabilities (future extension)
//!
//! ## Key Features
//!
//! - **Service Composition**: Automatic registration of reflection, health, and utility services
//! - **Flexible Binding**: Support for specific addresses or automatic port assignment
//! - **Health Management**: Built-in gRPC health checking with service-level granularity
//! - **Integration Testing**: Designed for easy testing with controlled server lifecycles
//! - **Stream Support**: Advanced serving options with custom connection streams
//!
//! ## Usage
//!
//! ### Basic Server Setup
//!
//! ```rust,no_run
//! use personal_ledger_backend::server::{Router, TonicServer};
//!
//! // Create a router with all services
//! let router = Router::new().await?;
//!
//! // Create server with automatic port assignment
//! let server = TonicServer::new("127.0.0.1:0".parse().unwrap()).await?;
//!
//! // Get the assigned address
//! println!("Server listening on: {}", server.address_string()?);
//!
//! // Start serving
//! server.run().await?;
//! ```
//!
//! ### Integration Testing
//!
//! ```rust,no_run
//! use personal_ledger_backend::server::TonicServer;
//!
//! // Create server for testing
//! let server = TonicServer::new("127.0.0.1:0".parse().unwrap()).await?;
//! let addr = server.local_addr()?;
//!
//! // Run server in background or use custom streams for testing
//! // server.serve_with_incoming(custom_stream).await?;
//! ```
//!
//! ## Components
//!
//! - **[`Router`]**: Service composition and health management
//! - **[`TonicServer`]**: High-level server abstraction with network binding
//!
//! ## Error Handling
//!
//! All server operations return structured errors that can be converted to appropriate
//! gRPC status codes. The module uses the application's standard `LedgerResult` and
//! `LedgerError` types for consistent error propagation.
//!
//! ## Security Considerations
//!
//! - TLS support is available through the `tls` module (currently internal)
//! - Health endpoints should be properly secured in production environments
//! - Consider rate limiting and authentication for public-facing services
//!
//! ## Performance Notes
//!
//! - Servers are optimized for concurrent request handling using Tokio's async runtime
//! - Connection pooling and reuse is handled automatically by the underlying Tonic framework
//! - Health checks are lightweight and designed for frequent polling by load balancers

mod router;
/// Service router for composing and managing gRPC services.
///
/// The Router handles the composition of multiple gRPC services including
/// reflection, health checking, and application-specific utilities.
/// It provides a clean abstraction for service management and health reporting.
pub use router::Router;

mod tls;
// TLS configuration and utilities (internal module).
//
// This module provides TLS/SSL support for secure gRPC communications.
// Currently internal but may be exposed in future versions.

mod tonic;
/// High-level Tonic gRPC server abstraction.
///
/// TonicServer combines a Router with network binding capabilities,
/// providing an easy-to-use interface for running gRPC servers with
/// advanced lifecycle management and testing support.
pub use tonic::TonicServer;