//-- ./src/rpc.rs

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

/// File descriptor set for gRPC reflection.
pub use proto::FILE_DESCRIPTOR_SET;

/// gRPC client for the `UtilitiesService`.
pub use proto::utilities_service_client::UtilitiesServiceClient;

/// gRPC server and trait for the `UtilitiesService`.
pub use proto::utilities_service_server::{UtilitiesService, UtilitiesServiceServer};

/// Request and response message types for the Ping endpoint.
pub use proto::{PingRequest, PingResponse};

/// gRPC client for the `CategoriesService`.
pub use proto::categories_service_client::CategoriesServiceClient;

/// gRPC server and trait for the `CategoriesService`.
pub use proto::categories_service_server::{CategoriesService, CategoriesServiceServer};

/// Message types for categories.
pub use proto::{
    Category, CategoryTypes,
    CategoryCreateRequest, CategoryCreateResponse,
    CategoryGetRequest, CategoryGetResponse,
    CategoryGetByCodeRequest, CategoryGetByCodeResponse,
    CategoryGetBySlugRequest, CategoryGetBySlugResponse,
    CategoriesListRequest, CategoriesListResponse,
    CategoryUpdateRequest, CategoryUpdateResponse,
    CategoriesBatchCreateRequest, CategoriesBatchCreateResponse,
    CategoryDeleteRequest, CategoryDeleteResponse,
    CategoriesBatchDeleteRequest, CategoriesBatchDeleteResponse,
    CategoryActivateRequest, CategoryActivateResponse,
    CategoryDeactivateRequest, CategoryDeactivateResponse,
};

/// Google protobuf types used in the API.
pub use prost_types::{Timestamp, FieldMask};

