//! # Integration Test Mock Utilities
//!
//! This module provides mock data generation functions specifically for integration tests.
//! These utilities create realistic, randomized test data for domain types like `RowID`,
//! helping ensure integration tests are robust and cover various scenarios.
//!
//! ## Key Features
//! - Deterministic randomization using the `fake` crate for reproducible tests
//! - UUID v7 generation for time-ordered identifiers
//! - Support for both random and deterministic timestamp-based IDs
//!
//! ## Usage
//! Import the desired functions in your integration test files:
//! ```rust
//! use tests::api::helpers::mocks::{mock_row_id, mock_row_id_from_datetime};
//! ```
//!
//! ## Testing Guidelines
//! These mocks follow the project's testing principles:
//! - Use deterministic seeds for reproducible results
//! - Generate realistic data distributions
//! - Avoid production code dependencies

use chrono::Timelike;
use personal_ledger_backend::rpc;

/// Create a mock RowID with a random timestamp for integration testing.
///
/// This generates a RowID with a random creation time between the Unix epoch
/// and the current time, useful for testing scenarios requiring varied but valid identifiers.
/// The generated RowID is always a valid UUID v7 with a time-ordered component.
///
/// This function is available in integration tests to provide diverse test data
/// for database operations, API endpoints, and business logic validation.
///
/// # Randomization Details
/// - Timestamp: Randomly selected between Unix epoch and now
/// - UUID Version: Always v7 for time-based ordering
/// - Distribution: Uniform random selection using the `fake` crate
///
/// # Examples
///
/// ```rust
/// use tests::api::helpers::mocks::mock_row_id;
/// use personal_ledger_backend::domain::RowID;
///
/// let mock_id = mock_row_id();
/// assert_eq!(mock_id.as_uuid().get_version_num(), 7);
/// // Use in integration test scenarios
/// ```
///
/// # Panics
/// This function may panic if the system's random number generator fails,
/// though this is extremely rare in practice.
pub fn mock_row_id() -> personal_ledger_backend::domain::RowID {
    use chrono::{DateTime, Utc};
    use fake::Fake;
    use fake::faker::chrono::en::DateTimeAfter;

    // Generate random DateTime after UNIX time epoch (00:00:00 UTC on 1 January 1970)
    let random_datetime: DateTime<Utc> = DateTimeAfter(chrono::DateTime::UNIX_EPOCH).fake();

    // Convert datetime to a UUID timestamp
    let random_uuid_timestamp: uuid::Timestamp = uuid::Timestamp::from_unix(
        uuid::NoContext,
        random_datetime.timestamp() as u64,
        random_datetime.timestamp_nanos_opt().unwrap() as u32,
    );

    // Generate Uuid V7
    let row_id = uuid::Uuid::new_v7(random_uuid_timestamp);

    personal_ledger_backend::domain::RowID::from_uuid(row_id)
}

/// Create a mock RowID with a specific timestamp for integration testing.
///
/// This allows creating RowIDs with deterministic timestamps, useful for
/// testing chronological ordering, time-based sorting, and temporal logic.
/// Unlike [`mock_row_id()`], this provides full control over the embedded timestamp,
/// enabling precise test scenarios.
///
/// This function is available in integration tests to provide controlled,
/// reproducible test data for scenarios requiring specific temporal ordering.
///
/// # Arguments
///
/// * `date_time` - The UTC timestamp to embed in the UUID. Must be a valid
///   `chrono::DateTime<Utc>`. The timestamp affects the UUID's ordering properties.
///
/// # Examples
///
/// ```rust
/// use chrono::{DateTime, Utc};
/// use tests::api::helpers::mocks::mock_row_id_from_datetime;
/// use personal_ledger_backend::domain::RowID;
///
/// let timestamp = DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
///     .unwrap()
///     .with_timezone(&Utc);
/// let id = mock_row_id_from_datetime(timestamp);
/// assert_eq!(id.as_uuid().get_version_num(), 7);
/// // Use for testing time-ordered operations
/// ```
///
/// # Panics
/// - Panics if `date_time.timestamp_nanos_opt()` returns `None`, which occurs
///   for dates far outside the valid Unix timestamp range.
/// - Panics if the timestamp conversion to UUID fails, though this is unlikely
///   for valid `DateTime<Utc>` inputs.
#[allow(dead_code)]
pub fn mock_row_id_from_datetime(
    date_time: chrono::DateTime<chrono::Utc>,
) -> personal_ledger_backend::domain::RowID {
    // Convert datetime to a UUID timestamp
    let uuid_timestamp: uuid::Timestamp = uuid::Timestamp::from_unix(
        uuid::NoContext,
        date_time.timestamp() as u64,
        date_time.timestamp_nanos_opt().unwrap() as u32,
    );

    // Generate Uuid V7
    let row_id = uuid::Uuid::new_v7(uuid_timestamp);

    personal_ledger_backend::domain::RowID::from_uuid(row_id)
}

/// Generates a random CategoryTypes variant for integration testing.
///
/// This function randomly selects one of the available category types (Asset, Liability,
/// Income, Expense, or Equity) using uniform random distribution. Useful for testing
/// category-related operations that need to work across all category types.
///
/// This function is available in integration tests to provide diverse test coverage
/// across all category types, ensuring business logic works correctly regardless
/// of the category type being processed.
///
/// # Randomization Details
/// - **Selection**: Uniform random selection from all available category types
/// - **Types Available**: Asset, Liability, Income, Expense, Equity (5 total)
/// - **Distribution**: Equal probability for each type
///
/// # Examples
///
/// Basic category type generation:
/// ```rust
/// use tests::api::helpers::mocks::mock_category_types;
/// use personal_ledger_backend::domain::CategoryTypes;
///
/// let category_type = mock_category_types();
/// // category_type will be one of the 5 available types
/// ```
///
/// Using in category creation tests:
/// ```rust
/// use tests::api::helpers::mocks::{mock_category_types, mock_row_id};
/// use personal_ledger_backend::database::CategoryBuilder;
///
/// let category = CategoryBuilder::new()
///     .with_id(mock_row_id())
///     .with_name("Test Category".to_string())
///     .with_category_type(mock_category_types())
///     .build()
///     .expect("Category should build with mock data");
/// ```
///
/// # Panics
/// This function does not panic as it uses only safe operations and the
/// category types are guaranteed to exist.
///
/// # See Also
/// - [`mock_category()`] in the categories module for complete category generation
pub fn mock_category_types() -> personal_ledger_backend::domain::CategoryTypes {
    use fake::Fake;

    // Get all category types and randomly select one
    let all_types = personal_ledger_backend::domain::CategoryTypes::all();
    let random_index: usize = (0..all_types.len()).fake();
    all_types[random_index].clone()
}

/// Generates a random hex color for integration testing.
///
/// This function creates a valid hex color code using the `fake` crate's color generator,
/// ensuring the generated color is always a valid hex format. Useful for testing
/// color-related functionality in categories and other components that support colors.
///
/// This function is available in integration tests to provide realistic color data
/// for UI components, validation logic, and display functionality.
///
/// # Randomization Details
/// - **Format**: Valid hex color codes (e.g., #FF0000, #3366CC)
/// - **Validation**: Automatically validated through `HexColor::parse()`
/// - **Distribution**: Uniform random selection from available color space
///
/// # Examples
///
/// Basic color generation:
/// ```rust
/// use tests::api::helpers::mocks::mock_hex_color;
/// use personal_ledger_backend::domain::HexColor;
///
/// let color = mock_hex_color();
/// // color is guaranteed to be a valid hex color
/// ```
///
/// Using in category color tests:
/// ```rust
/// use tests::api::helpers::mocks::mock_hex_color;
/// use personal_ledger_backend::database::CategoryBuilder;
///
/// let category = CategoryBuilder::new()
///     .with_id(mock_row_id())
///     .with_name("Colored Category".to_string())
///     .with_color_opt(Some(mock_hex_color()))
///     .build()
///     .expect("Category should build with mock color");
/// ```
///
/// # Panics
/// This function may panic if `HexColor::parse()` fails on the generated color string,
/// though this is extremely unlikely as the `fake` crate generates valid hex colors.
/// If this occurs, it indicates an issue with the color parsing logic.
///
/// # See Also
/// - [`mock_hex_color_with_option()`] for optional color generation
/// - [`mock_category()`] in the categories module for complete category generation
pub fn mock_hex_color() -> personal_ledger_backend::domain::HexColor {
    use fake::Fake;
    use fake::faker::color::en::HexColor as FakeHex;

    let value: String = FakeHex().fake();
    personal_ledger_backend::domain::HexColor::parse(value)
        .expect("fake hex colour should be valid")
}

/// Generates a random optional hex color for integration testing.
///
/// This function randomly decides whether to return a valid hex color or None,
/// with equal probability of each outcome. When Some, it generates a valid hex
/// color code using the `fake` crate. Useful for testing optional color fields
/// in categories and other components.
///
/// This function is available in integration tests to provide realistic optional
/// color data, ensuring components handle both presence and absence of colors correctly.
///
/// # Randomization Details
/// - **Presence**: 50% probability of Some, 50% probability of None
/// - **Color Format**: Valid hex color codes when present (e.g., #FF0000, #3366CC)
/// - **Validation**: Automatically validated through `HexColor::parse()` when present
/// - **Distribution**: Balanced to test both optional field scenarios
///
/// # Examples
///
/// Basic optional color generation:
/// ```rust
/// use tests::api::helpers::mocks::mock_hex_color_with_option;
/// use personal_ledger_backend::domain::HexColor;
///
/// let color_opt = mock_hex_color_with_option();
/// match color_opt {
///     Some(color) => {
///         // color is guaranteed to be a valid hex color
///     }
///     None => {
///         // Test None handling
///     }
/// }
/// ```
///
/// Using in category tests:
/// ```rust
/// use tests::api::helpers::mocks::mock_hex_color_with_option;
/// use personal_ledger_backend::database::CategoryBuilder;
///
/// let category = CategoryBuilder::new()
///     .with_id(mock_row_id())
///     .with_name("Optional Color Category".to_string())
///     .with_color_opt(mock_hex_color_with_option())
///     .build()
///     .expect("Category should build with optional color");
/// ```
///
/// # Panics
/// This function may panic if `HexColor::parse()` fails on the generated color string
/// when Some is returned, though this is extremely unlikely as the `fake` crate
/// generates valid hex colors. If this occurs, it indicates an issue with the color parsing logic.
///
/// # See Also
/// - [`mock_hex_color()`] for guaranteed color generation
/// - [`mock_category()`] in the categories module for complete category generation
pub fn mock_hex_color_with_option() -> Option<personal_ledger_backend::domain::HexColor> {
    use fake::Fake;
    use fake::faker::boolean::en::Boolean;

    let is_some: bool = Boolean(50).fake(); // 50% chance of Some
    if is_some {
        Some(mock_hex_color())
    } else {
        None
    }
}

/// Generates a random datetime for integration testing.
///
/// This function creates a random UTC datetime between the Unix epoch and the current time,
/// useful for testing scenarios that require varied timestamps. The generated datetime
/// follows a uniform distribution across the valid time range.
///
/// This function is available in integration tests to provide diverse temporal test data
/// for scenarios involving time-based operations, sorting, and validation.
///
/// # Randomization Details
/// - **Range**: From Unix epoch (1970-01-01T00:00:00Z) to current time
/// - **Timezone**: Always UTC for consistency
/// - **Distribution**: Uniform random selection using the `fake` crate
///
/// # Examples
///
/// Basic datetime generation:
/// ```rust
/// use tests::api::helpers::mocks::mock_datetime;
/// use chrono::{DateTime, Utc};
///
/// let datetime = mock_datetime();
/// assert!(datetime >= DateTime::UNIX_EPOCH);
/// assert!(datetime <= Utc::now());
/// ```
///
/// Using in temporal testing:
/// ```rust
/// use tests::api::helpers::mocks::mock_datetime;
/// use personal_ledger_backend::database::CategoryBuilder;
///
/// let created_at = mock_datetime();
/// let category = CategoryBuilder::new()
///     .with_id(mock_row_id())
///     .with_name("Time-Tested Category".to_string())
///     .with_created_on_opt(Some(created_at))
///     .build()
///     .expect("Category should build with mock datetime");
/// ```
///
/// # Panics
/// This function does not panic as it uses only safe operations and the
/// `fake` crate's well-tested datetime generation.
///
/// # See Also
/// - [`mock_datetime_after()`] for datetime generation after a specific time
/// - [`mock_row_id()`] for time-ordered UUID generation
pub fn mock_datetime() -> chrono::DateTime<chrono::Utc> {
    use fake::Fake;
    use fake::faker::chrono::en::DateTimeAfter;

    // Generate random DateTime after UNIX time epoch (00:00:00 UTC on 1 January 1970)
    let random_datetime: chrono::DateTime<chrono::Utc> =
        DateTimeAfter(chrono::DateTime::UNIX_EPOCH).fake();

    random_datetime
}

pub fn to_rpc_datetime(datetime: chrono::DateTime<chrono::Utc>) -> rpc::Timestamp {
    rpc::Timestamp {
        seconds: datetime.timestamp(),
        nanos: datetime.nanosecond() as i32,
    }
}

/// Generates a random datetime after a specified time for integration testing.
///
/// This function creates a random UTC datetime that occurs after the provided `after` datetime,
/// useful for testing scenarios requiring chronological ordering or time-based relationships.
/// The generated datetime is guaranteed to be after the specified time.
///
/// This function is available in integration tests to provide controlled temporal sequences,
/// ensuring proper ordering in time-sensitive operations and validations.
///
/// # Arguments
///
/// * `after` - The UTC datetime after which the generated datetime must occur.
///   Must be a valid `chrono::DateTime<Utc>`.
///
/// # Randomization Details
/// - **Range**: From the specified `after` time to current time
/// - **Timezone**: Always UTC for consistency
/// - **Distribution**: Uniform random selection using the `fake` crate
/// - **Guarantee**: Generated datetime is always > `after` parameter
///
/// # Examples
///
/// Basic datetime generation after a specific time:
/// ```rust
/// use tests::api::helpers::mocks::mock_datetime_after;
/// use chrono::{DateTime, Utc};
///
/// let base_time = DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
///     .unwrap()
///     .with_timezone(&Utc);
/// let future_datetime = mock_datetime_after(base_time);
/// assert!(future_datetime > base_time);
/// ```
///
/// Using in update timestamp testing:
/// ```rust
/// use tests::api::helpers::mocks::{mock_datetime, mock_datetime_after};
/// use personal_ledger_backend::database::CategoryBuilder;
///
/// let created_at = mock_datetime();
/// let updated_at = mock_datetime_after(created_at);
///
/// let category = CategoryBuilder::new()
///     .with_id(mock_row_id())
///     .with_name("Updated Category".to_string())
///     .with_created_on_opt(Some(created_at))
///     .with_updated_on_opt(Some(updated_at))
///     .build()
///     .expect("Category should build with chronological timestamps");
/// assert!(updated_at > created_at);
/// ```
///
/// # Panics
/// This function does not panic as it uses only safe operations and the
/// `fake` crate's well-tested datetime generation.
///
/// # See Also
/// - [`mock_datetime()`] for datetime generation from Unix epoch
/// - [`mock_row_id_from_datetime()`] for UUID generation with specific timestamps
pub fn mock_datetime_after(after: chrono::DateTime<chrono::Utc>) -> chrono::DateTime<chrono::Utc> {
    use fake::Fake;
    use fake::faker::chrono::en::DateTimeAfter;

    // Generate random DateTime after passed in date time
    let random_datetime: chrono::DateTime<chrono::Utc> = DateTimeAfter(after).fake();

    random_datetime
}
