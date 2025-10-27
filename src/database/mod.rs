//! # Database Module
//!
//! This module provides the core database functionality for the Personal Ledger backend.
//! It includes connection pooling, error handling, and domain models for financial data.
//!
//! ## Overview
//!
//! The database layer is designed following domain-driven design principles, with
//! separate modules for different concerns:
//!
//! - Connection management and pooling ([`DatabasePool`])
//! - Standardized error types ([`DatabaseError`], [`DatabaseResult`])
//! - Domain models like financial categories ([`Category`])
//!
//! ## Architecture
//!
//! - **Pool Management**: [`DatabasePool`] provides a lightweight wrapper around SQLx
//!   connection pools with centralized error handling and lifecycle management.
//! - **Error Handling**: All database operations return [`DatabaseResult<T>`] for
//!   consistent error propagation using [`DatabaseError`] variants.
//! - **Domain Models**: Structs like [`Category`] represent business entities with
//!   validation and conversion logic.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use personal_ledger_backend::database::{DatabasePool, DatabaseResult};
//!
//! async fn initialize_db() -> DatabaseResult<()> {
//!     let pool = DatabasePool::connect("sqlite::memory:").await?;
//!     // Use pool for database operations...
//!     Ok(())
//! }
//! ```

mod error;
/// Core error type for all database operations.
///
/// This enum standardizes error handling across the database layer, wrapping
/// connection failures, SQLx errors, migration issues, and validation problems.
/// All database functions should return `Result<T, DatabaseError>` for consistent
/// error propagation.
///
/// See [`error`] module for detailed documentation and examples.
pub use error::DatabaseError;

/// Result type alias for database operations.
///
/// Convenience type for functions that return database results.
/// Equivalent to `Result<T, DatabaseError>`.
///
/// # Examples
///
/// ```rust
/// use personal_ledger_backend::database::DatabaseResult;
///
/// fn get_user(id: i32) -> DatabaseResult<User> {
///     // Database operation that may fail...
/// }
/// ```
pub use error::DatabaseResult;

mod pool;
/// Database connection pool wrapper.
///
/// Provides a lightweight abstraction over SQLx connection pools with centralized
/// error handling, lifecycle management, and graceful shutdown. Supports both
/// SQLite and PostgreSQL through the `sqlx::any` driver.
///
/// See [`pool`] module for detailed API documentation and examples.
pub use pool::DatabasePool;

mod categories;
/// Financial category domain model.
///
/// Represents accounting categories (assets, liabilities, income, expenses, equity)
/// used for classifying transactions and accounts. Includes validation and
/// builder pattern support.
///
/// See [`categories`] module for implementation details.
pub use categories::Categories;
pub use categories::CategoriesBuilder;