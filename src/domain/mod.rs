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
//! - [`RowID`] - Time-ordered UUID v7 identifiers for database rows
//! - [`CategoryTypes`] - Classification types for financial transactions
//!
//! ## Design Principles
//!
//! - **Type Safety**: Use newtypes to prevent mixing of different ID types
//! - **Validation**: Enforce business rules at construction time
//! - **Immutability**: Domain objects are immutable after creation
//! - **Serialisation**: All types support JSON serialisation for API responses

mod db_engine;
pub use db_engine::DbEngine;

/// Database row identifier type using time-ordered UUID v7.
///
/// [`RowID`] provides unique, sortable identifiers that maintain chronological
/// ordering while ensuring global uniqueness across distributed systems.
mod row_id;
pub use row_id::RowID;

/// Category types used for classifying financial transactions.
///
/// [`CategoryTypes`] represents the fundamental accounting categories
/// (assets, liabilities, income, expenses, equity) used to classify
/// and organise financial transactions in the personal ledger.
// mod category_types;
// pub use category_types::CategoryTypes;

mod url_slug;
pub use url_slug::UrlSlug;