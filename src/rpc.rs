//! gRPC API module for Personal Ledger Backend.
//!
//! This module provides the generated gRPC client/server types and message types for the
//! `personal_ledger` protobuf package. It re-exports the most commonly used types for
//! ergonomic use in the rest of the application.
//!
//! # Usage
//!
//! Import this module to access the gRPC service definitions and message types:
//!
//! ```rust
//! use personal_ledger_backend::rpc::{UtilitiesServiceServer, UtilitiesServiceClient, PingRequest, PingResponse};
//! ```
//!
//! The `FILE_DESCRIPTOR_SET` constant is used for gRPC reflection support.

mod proto {
    // The string specified here must match the proto package name
    tonic::include_proto!("personal_ledger");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("personal_ledger_descriptor");
}

/// gRPC client for the `UtilitiesService`.
pub use proto::utilities_service_client::UtilitiesServiceClient;

/// gRPC server and trait for the `UtilitiesService`.
pub use proto::utilities_service_server::{UtilitiesService, UtilitiesServiceServer};

/// Request and response message types for the Ping endpoint.
pub use proto::{PingRequest, PingResponse};

/// File descriptor set for gRPC reflection.
pub use proto::FILE_DESCRIPTOR_SET;