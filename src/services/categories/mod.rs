//! # Categories Service Module
//!
//! This module provides the gRPC service implementation for managing financial categories
//! in the Personal Ledger backend. It includes CRUD operations, batch operations,
//! filtering, and activation/deactivation functionality.
//!
//! ## Modules
//!
//! - `activate`: Category activation logic
//! - `create`: Category creation logic (single and batch)
//! - `deactivate`: Category deactivation logic
//! - `delete`: Category deletion logic (single and batch)
//! - `get`: Category retrieval logic (by ID, code, slug)
//! - `list`: Category listing with filtering and pagination
//! - `service`: gRPC service trait implementation
//! - `update`: Category update logic with field mask support
//!
//! ## Re-exports
//!
//! All public functions and types from the submodules are re-exported here for cleaner code 
//! and convenience.

mod activate;
mod create;
mod deactivate;
mod delete;
mod get;
mod list;
mod service;
mod update;


// Reexport all submodules for cleaner code and convenience
pub use activate::*;
pub use create::*;
pub use deactivate::*;
pub use delete::*;
pub use get::*;
pub use list::*;
pub use service::*;
pub use update::*;