//! # Categories Database Module
//!
//! Provides data access helpers, builders, and models for working with
//! category records in the persistence layer. The module exposes the
//! database representation of a category alongside utilities for creating
//! and inserting records during tests or data seeding.

// #![allow(unused)] // For development only

mod builder;
mod model;
mod insert;
mod update;
mod delete;
mod find;

/// Database row model representing a persisted category.
pub use model::Category;

/// Fluent builder for constructing `Category` instances in tests and fixtures.
#[cfg(test)]
pub use builder::CategoriesBuilder;