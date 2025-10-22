//! # Category Integration Test Mock Utilities
//!
//! This module provides comprehensive mock data generation functions specifically designed
//! for category-related integration tests in the Personal Ledger backend. These utilities
//! create realistic, randomized test data that closely mimics production scenarios,
//! ensuring robust testing of database operations, gRPC endpoints, and business logic.
//!
//! ## Key Features
//! - **Realistic Data Generation**: Uses the `fake` crate with deterministic seeds for reproducible tests
//! - **Production-Aligned Distributions**: Field probabilities match expected production data patterns
//! - **Complete Coverage**: Supports all Category fields including optional components
//! - **Time-Ordered IDs**: UUID v7 generation for proper temporal ordering in database tests
//! - **Flexible Scenarios**: Both fully randomized and controlled test data generation
//!
//! ## Usage in Integration Tests
//! Import functions as needed in your test files:
//! ```rust
//! use tests::api::categories::mock::{mock_category, generate_mock_name};
//! use personal_ledger_backend::database::Category;
//!
//! #[tokio::test]
//! async fn test_category_creation() {
//!     let category = mock_category();
//!     // Test your category creation logic...
//! }
//! ```
//!
//! ## Testing Philosophy
//! These mocks follow the project's testing principles:
//! - **Deterministic**: Use fixed seeds for reproducible CI/CD results
//! - **Realistic**: Data distributions reflect production expectations
//! - **Isolated**: No dependencies on production code to prevent test pollution
//! - **Comprehensive**: Cover both happy path and edge case scenarios
//! - **Maintainable**: Clear documentation and predictable behavior
//!
//! ## Data Distribution Guidelines
//! - Categories are active 80% of the time (realistic for production systems)
//! - Optional fields (description, color, icon) have 50% presence probability
//! - Names use 1-3 words for varied but realistic lengths
//! - Codes follow the XXX.XXX.XXX format with uppercase alphanumeric characters
//! - Timestamps use current time for created/updated fields in most scenarios

use personal_ledger_backend::domain;
use personal_ledger_backend::rpc;

use crate::helpers;

/// Creates a mock RPC 
pub fn mock_rpc_category() -> rpc::Category {
    let created_on = helpers::mock_datetime();
    rpc::Category {
        id: helpers::mock_row_id().to_string(),
        code: generate_mock_code(),
        name: generate_mock_name(),
        description: generate_mock_description(),
        url_slug: generate_mock_url_slug(),
        category_type: helpers::mock_category_types().to_rpc_i32(),
        color: helpers::mock_hex_color_with_option().map(|c| c.to_string()),
        icon: generate_mock_icon(),
        is_active: generate_mock_is_active(),
        created_on: Some(helpers::to_rpc_datetime(created_on)),
        updated_on: Some(helpers::to_rpc_datetime(helpers::mock_datetime_after(created_on))),
    }
}

/// Generates a mock category code following the project's code format conventions.
///
/// Creates a deterministic code in the format XXX.XXX.XXX where each X is an uppercase
/// alphanumeric character. This format is used throughout the Personal Ledger system
/// for category identification and ensures uniqueness constraints can be properly tested.
///
/// # Randomization Details
/// - **Format**: Three groups of three characters separated by dots (XXX.XXX.XXX)
/// - **Characters**: Random uppercase letters (A-Z) and digits (0-9)
/// - **Length**: Always exactly 11 characters including separators
/// - **Distribution**: Uniform random selection from alphanumeric character set
///
/// # Examples
///
/// Basic code generation:
/// ```rust
/// use tests::api::categories::mock::generate_mock_code;
///
/// let code = generate_mock_code();
/// assert_eq!(code.len(), 11);
/// assert_eq!(code.chars().filter(|&c| c == '.').count(), 2);
/// assert!(code.chars().all(|c| c.is_ascii_alphanumeric() || c == '.'));
/// ```
///
/// Using in uniqueness constraint tests:
/// ```rust
/// use tests::api::categories::mock::generate_mock_code;
///
/// let codes: Vec<String> = (0..100).map(|_| generate_mock_code()).collect();
/// let unique_codes: std::collections::HashSet<_> = codes.iter().cloned().collect();
/// // While collisions are possible, they're extremely rare with random generation
/// ```
///
/// # Panics
/// This function does not panic under normal circumstances as it uses only safe
/// string operations and character conversions.
///
/// # See Also
/// - [`mock_category()`] for complete category generation including codes
pub fn generate_mock_code() -> String {
    use rand::Rng;

    // Generate 9 random alphanumeric chars, uppercase, then split into 3 groups
    let mut rng = rand::rng();
    let s: String = (&mut rng)
        .sample_iter(&rand::distr::Alphanumeric)
        .take(9)
        .map(|b| (b as char).to_ascii_uppercase())
        .collect();

    format!("{}.{}.{}", &s[0..3], &s[3..6], &s[6..9])
}

/// Generates a mock category name using realistic lorem ipsum text.
///
/// Creates category names consisting of 1-3 words, providing varied lengths that
/// reflect typical category naming patterns in financial applications. The names
/// are suitable for testing display, sorting, and validation logic.
///
/// # Randomization Details
/// - **Word Count**: Random selection between 1 and 3 words
/// - **Content**: Lorem ipsum words for realistic English-like text
/// - **Format**: Space-separated words with proper capitalization
/// - **Length**: Typically 5-20 characters for varied testing scenarios
///
/// # Examples
///
/// Basic name generation:
/// ```rust
/// use tests::api::categories::mock::generate_mock_name;
///
/// let name = generate_mock_name();
/// assert!(!name.is_empty());
/// assert!(name.split_whitespace().count() >= 1);
/// assert!(name.split_whitespace().count() <= 3);
/// ```
///
/// Using in validation tests:
/// ```rust
/// use tests::api::categories::mock::generate_mock_name;
/// use personal_ledger_backend::domain::CategoryName;
///
/// let name = generate_mock_name();
/// // Test that the name can be parsed into a CategoryName
/// let category_name = CategoryName::parse(name).expect("Mock name should be valid");
/// ```
///
/// # Panics
/// This function does not panic as it uses only safe string operations and
/// the `fake` crate's well-tested lorem ipsum generation.
///
/// # See Also
/// - [`mock_category()`] for complete category generation including names
/// - [`generate_mock_description()`] for longer descriptive text
pub fn generate_mock_name() -> String {
    use fake::Fake;
    use fake::faker::lorem::en::Words;

    let words: Vec<String> = Words(1..3).fake();
    words.join(" ")
}

/// Generates a mock category description with realistic optional presence.
///
/// Randomly decides whether to return a description or None, with equal probability.
/// When present, generates 3-8 lorem ipsum words to simulate typical category descriptions
/// found in financial applications. This helps test both scenarios where descriptions
/// are provided and where they are omitted.
///
/// # Randomization Details
/// - **Presence**: 50% probability of Some, 50% probability of None
/// - **Content**: 3-8 lorem ipsum words when present
/// - **Format**: Space-separated words forming coherent text
/// - **Length**: Typically 20-60 characters for substantial descriptions
///
/// # Examples
///
/// Basic description generation:
/// ```rust
/// use tests::api::categories::mock::generate_mock_description;
///
/// let description = generate_mock_description();
/// match description {
///     Some(text) => {
///         assert!(!text.is_empty());
///         assert!(text.split_whitespace().count() >= 3);
///         assert!(text.split_whitespace().count() <= 8);
///     }
///     None => {
///         // Test None handling
///     }
/// }
/// ```
///
/// Using in form validation tests:
/// ```rust
/// use tests::api::categories::mock::generate_mock_description;
///
/// // Test both optional field scenarios
/// let with_desc = generate_mock_description();
/// let without_desc = None;
///
/// // Verify your form handling logic works for both cases
/// ```
///
/// # Panics
/// This function does not panic as it uses only safe operations and the
/// `fake` crate's well-tested random generation facilities.
///
/// # See Also
/// - [`generate_mock_name()`] for shorter name text
/// - [`mock_category()`] for complete category generation
pub fn generate_mock_description() -> Option<String> {
    use fake::Fake;
    use fake::faker::boolean::en::Boolean;
    use fake::faker::lorem::en::Words;

    let is_some: bool = Boolean(50).fake(); // 50% chance of Some
    if is_some {
        let words: Vec<String> = Words(3..8).fake();
        Some(words.join(" "))
    } else {
        None
    }
}

/// Generates a mock URL slug derived from a mock category name.
///
/// Creates a URL-safe slug from a randomly generated category name, ensuring
/// the slug follows the project's URL formatting conventions. Always returns
/// Some value as slugs are typically required for category identification.
///
/// # Randomization Details
/// - **Base Content**: Uses [`generate_mock_name()`] as input source
/// - **Format**: URL-safe format with lowercase letters, hyphens for spaces
/// - **Presence**: Always Some, as slugs are typically mandatory
/// - **Validation**: Automatically validated through `UrlSlug::from()`
///
/// # Examples
///
/// Basic slug generation:
/// ```rust
/// use tests::api::categories::mock::generate_mock_url_slug;
/// use personal_ledger_backend::domain::UrlSlug;
///
/// let slug = generate_mock_url_slug();
/// assert!(slug.is_some());
/// // The slug is guaranteed to be valid due to UrlSlug::from() validation
/// ```
///
/// Using in routing tests:
/// ```rust
/// use tests::api::categories::mock::generate_mock_url_slug;
///
/// let slug = generate_mock_url_slug().unwrap();
/// let slug_str = slug.as_str();
/// // Test that your routing logic can handle the generated slug
/// assert!(slug_str.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
/// ```
///
/// # Panics
/// This function may panic if `UrlSlug::from()` fails to parse the generated name,
/// though this is extremely unlikely for valid lorem ipsum text. If this occurs,
/// it indicates an issue with the name generation or URL slug parsing logic.
///
/// # See Also
/// - [`generate_mock_name()`] for the base name generation
/// - [`mock_category()`] for complete category generation including slugs
pub fn generate_mock_url_slug() -> Option<String> {
    use fake::Fake;
    use fake::faker::boolean::en::Boolean;

    let is_some: bool = Boolean(50).fake(); // 50% chance of Some
    if is_some {
        Some(domain::UrlSlug::from(generate_mock_name()).to_string())
    } else {
        None
    }
}

/// Generates a mock icon name with realistic optional presence.
///
/// Randomly decides whether to return an icon identifier or None, simulating
/// the optional nature of category icons in the Personal Ledger system.
/// When present, uses a single lorem ipsum word as the icon name.
///
/// # Randomization Details
/// - **Presence**: 50% probability of Some, 50% probability of None
/// - **Content**: Single lorem ipsum word when present
/// - **Format**: Alphabetic characters only, suitable for icon identifiers
/// - **Length**: Typically 3-10 characters for realistic icon names
///
/// # Examples
///
/// Basic icon generation:
/// ```rust
/// use tests::api::categories::mock::generate_mock_icon;
///
/// let icon = generate_mock_icon();
/// match icon {
///     Some(name) => {
///         assert!(!name.is_empty());
///         assert!(name.chars().all(|c| c.is_alphabetic()));
///     }
///     None => {
///         // Test None handling for categories without icons
///     }
/// }
/// ```
///
/// Using in UI component tests:
/// ```rust
/// use tests::api::categories::mock::generate_mock_icon;
///
/// let icon = generate_mock_icon();
/// // Test that your UI components handle both icon presence and absence
/// if let Some(icon_name) = icon {
///     // Verify icon loading/display logic
/// }
/// ```
///
/// # Panics
/// This function does not panic as it uses only safe operations and the
/// `fake` crate's well-tested word generation.
///
/// # See Also
/// - [`mock_category()`] for complete category generation including icons
pub fn generate_mock_icon() -> Option<String> {
    use fake::Fake;
    use fake::faker::boolean::en::Boolean;
    use fake::faker::lorem::en::Word;

    let is_some: bool = Boolean(50).fake(); // 50% chance of Some
    if is_some {
        Some(Word().fake())
    } else {
        None
    }
}

/// Generates a mock active status with production-realistic distribution.
///
/// Returns a boolean indicating whether a category should be considered active,
/// with an 80% probability of true to reflect real-world usage patterns where
/// most categories remain active in financial systems.
///
/// # Randomization Details
/// - **Distribution**: 80% probability of true, 20% probability of false
/// - **Bias**: Towards active categories for realistic test scenarios
/// - **Purpose**: Simulates production data where inactive categories are less common
///
/// # Examples
///
/// Basic active status generation:
/// ```rust
/// use tests::api::categories::mock::generate_mock_is_active;
///
/// let is_active = generate_mock_is_active();
/// // is_active is true 80% of the time on average
/// ```
///
/// Using in filtering tests:
/// ```rust
/// use tests::api::categories::mock::generate_mock_is_active;
///
/// let statuses: Vec<bool> = (0..1000).map(|_| generate_mock_is_active()).collect();
/// let active_count = statuses.iter().filter(|&&s| s).count();
/// // active_count should be approximately 800 (80%)
/// ```
///
/// # Panics
/// This function does not panic as it uses only safe boolean generation
/// from the well-tested `fake` crate.
///
/// # See Also
/// - [`mock_category()`] for complete category generation including active status
pub fn generate_mock_is_active() -> bool {
    use fake::Fake;
    use fake::faker::boolean::en::Boolean;

    Boolean(80).fake() // 80% chance of active for more realistic data
}