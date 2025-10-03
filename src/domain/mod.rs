//! # Domain Types Module
//!
//! This module contains the core domain types and business logic abstractions
//! for the Personal Ledger backend. Domain types represent the fundamental
//! business concepts and enforce invariants at the type level.
//!
//! The domain layer is independent of external frameworks and focuses on
//! expressing business rules clearly and safely using Rust's type system.
//!
//! ## Available Types
//!
//! - [`CategoryTypes`] - Classification types for financial transactions
//! - [`DbEngine`] - Database backend selection (SQLite or PostgreSQL)
//! - [`RowID`] - Time-ordered UUID v7 identifiers for database rows
//! - [`UrlSlug`] - URL-safe identifiers for web-friendly resource names
//!
//! ## Design Principles
//!
//! - **Type Safety**: Use newtypes to prevent mixing of different ID types
//! - **Validation**: Enforce business rules at construction time
//! - **Immutability**: Domain objects are immutable after creation
//! - **Serialisation**: All types support JSON serialisation for API responses

mod db_engine;
/// Database backend selector for runtime database engine configuration.
///
/// [`DbEngine`] determines which database system (SQLite or PostgreSQL) the
/// application connects to, enabling flexible deployment options from embedded
/// databases to enterprise PostgreSQL servers.
pub use db_engine::DbEngine;

/// Database row identifier type using time-ordered UUID v7.
///
/// [`RowID`] provides unique, sortable identifiers that maintain chronological
/// ordering while ensuring global uniqueness across distributed systems.
/// RowIDs are naturally ordered by creation time and prevent accidental
/// mixing with other identifier types.
mod row_id;
pub use row_id::{RowID, RowIDError};

/// Category types used for classifying financial transactions.
///
/// [`CategoryTypes`] represents the fundamental accounting categories
/// (assets, liabilities, income, expenses, equity) used to classify
/// and organise financial transactions in the personal ledger.
/// These categories follow the standard accounting equation and provide
/// type-safe classification for all financial operations.
mod category_types;
pub use category_types::{CategoryTypes, CategoryTypesError};

mod url_slug;
/// URL-safe slug type for human-readable, SEO-friendly identifiers.
///
/// [`UrlSlug`] creates web-friendly identifiers from titles and names by
/// converting them to lowercase, alphanumeric strings with hyphens. Used
/// throughout the application for creating readable URLs and resource names
/// that are both user-friendly and search engine optimized.
pub use url_slug::{UrlSlug, UrlSlugError};