//! # Category Database Model
//!
//! This module defines the `Category` struct and related types for representing
//! financial categories in the Personal Ledger backend database.
//!
//! Categories are used to classify financial transactions and accounts into
//! the fundamental accounting types: assets, liabilities, income, expenses, and equity.
//! They support hierarchical organisation and include metadata for user interface
//! customisation such as colours, icons, and URL-friendly slugs.
//!
//! ## Core Features
//!
//! - **Builder Pattern**: Fluent API for constructing Category instances
//! - **Mock Generation**: Test utilities for generating random Category data
//! - **Type Safety**: Strong typing with domain-specific CategoryTypes enum
//! - **Timestamps**: Automatic tracking of creation and update times
//! - **Flexible Fields**: Optional metadata fields for UI customisation
//!
//! ## Example Usage
//!
//! ```rust
//! use personal_ledger_backend::database::categories::Category;
//! use personal_ledger_backend::domain::CategoryTypes;
//!
//! // Using the builder pattern
//! let category = Category::builder()
//!     .code("FOOD-001")
//!     .name("Groceries")
//!     .category_type(CategoryTypes::Expense)
//!     .description("Food and grocery expenses")
//!     .color("#4CAF50")
//!     .icon("shopping-cart")
//!     .is_active(true)
//!     .build();
//! ```

use chrono;
use serde;
use sqlx;

use crate::domain;

/// Category domain model matching the `categories` table.
///
/// Represents a financial category used to classify transactions and accounts
/// according to standard accounting principles. Categories follow the fundamental
/// accounting equation and support the five main category types: assets, liabilities,
/// income, expenses, and equity.
///
/// # Database Mapping
///
/// This struct maps directly to the `categories` table and is used with SQLx's
/// `FromRow` derive macro for seamless database integration.
///
/// # Features
///
/// - **Type Safety**: Uses domain types (`RowID`, `UrlSlug`, `CategoryTypes`) for validation
/// - **Serialization**: Fully supports JSON serialization/deserialization via Serde
/// - **Metadata**: Includes UI customisation fields (color, icon, slug)
/// - **Audit Trail**: Automatic timestamp tracking for creation and updates
///
/// # Fields
///
/// - `id`: Time-ordered UUID v7 identifier ensuring chronological sorting
/// - `code`: Short, unique identifier (case-insensitive, e.g., "FOOD-001")
/// - `name`: Human-readable display name shown in user interfaces
/// - `description`: Optional detailed explanation of the category's purpose
/// - `slug`: Optional URL-safe identifier for web interfaces (e.g., "food-and-dining")
/// - `category_type`: Accounting classification (Asset, Liability, Income, Expense, Equity)
/// - `color`: Optional hex color code for visual identification (e.g., "#4CAF50")
/// - `icon`: Optional icon identifier for UI rendering (e.g., "shopping-cart")
/// - `is_active`: Visibility flag for new transactions (preserves historical data)
/// - `created_on`: UTC timestamp of initial creation
/// - `updated_on`: UTC timestamp of last modification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow, PartialEq)]
pub struct Category {
    /// Unique time-ordered identifier for the category.
    ///
    /// Uses UUID v7 for chronological sorting while maintaining global uniqueness.
    pub id: domain::RowID,

    /// Short, unique code for the category (case-insensitive).
    ///
    /// Should be descriptive but concise (e.g., "FOOD-001", "SALARY", "UTIL").
    /// Used for quick reference and system integration.
    pub code: String,

    /// Human-readable display name for the category.
    ///
    /// This is what users see in the interface. Should be clear and descriptive
    /// (e.g., "Groceries", "Monthly Salary", "Electricity Bills").
    pub name: String,

    /// Optional detailed description of the category's purpose and usage.
    ///
    /// Provides additional context about what types of transactions should be
    /// classified under this category. Helps users understand when to use it.
    pub description: Option<String>,

    /// Optional URL-safe slug for web interfaces and APIs.
    ///
    /// Automatically parsed from strings to ensure URL safety (lowercase,
    /// alphanumeric, and hyphens only). Must be unique when present.
    /// Example: "food-and-dining" from "Food & Dining".
    pub slug: Option<domain::UrlSlug>,

    /// Accounting classification type for this category.
    ///
    /// Determines the category's role in the accounting equation and financial
    /// statements. One of: Asset, Liability, Income, Expense, or Equity.
    pub category_type: domain::CategoryTypes,

    /// Optional hexadecimal color code for visual identification.
    ///
    /// Used in charts, graphs, and UI elements for quick visual recognition.
    /// Should be in #RRGGBB format (e.g., "#4CAF50" for green, "#FF5733" for red).
    pub color: Option<String>,

    /// Optional icon identifier for UI rendering.
    ///
    /// Typically corresponds to icon libraries like Font Awesome or Material Icons.
    /// Examples: "shopping-cart", "dollar-sign", "home", "car".
    pub icon: Option<String>,

    /// Visibility flag indicating if the category is available for new transactions.
    ///
    /// When `false`, the category is hidden from selection interfaces but remains
    /// in the database to preserve historical transaction data integrity.
    pub is_active: bool,

    /// UTC timestamp of when the category was initially created.
    ///
    /// Set automatically on creation and never modified thereafter.
    pub created_on: chrono::DateTime<chrono::Utc>,

    /// UTC timestamp of the last modification to this category.
    ///
    /// Updated automatically whenever any field is changed.
    pub updated_on: chrono::DateTime<chrono::Utc>,
}

impl Category {
    /// Create a new `CategoryBuilder` for constructing a `Category` instance.
    ///
    /// Returns a builder with sensible defaults that allows fluent configuration
    /// of category properties before construction. Required fields must be set
    /// before calling `build()`.
    ///
    /// # Required Fields
    ///
    /// - `code`: Category identifier
    /// - `name`: Display name
    /// - `category_type`: Accounting classification
    ///
    /// # Optional Fields
    ///
    /// All other fields have sensible defaults or can be left as `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// // Minimal category with required fields
    /// let category = Category::builder()
    ///     .code("FOOD")
    ///     .name("Food & Dining")
    ///     .category_type(CategoryTypes::Expense)
    ///     .build();
    ///
    /// // Category with optional metadata
    /// let category = Category::builder()
    ///     .code("SALARY")
    ///     .name("Monthly Salary")
    ///     .category_type(CategoryTypes::Income)
    ///     .description("Regular monthly income from employment")
    ///     .color("#00FF00")
    ///     .icon("dollar-sign")
    ///     .slug("monthly-salary")
    ///     .build();
    /// ```
    pub fn builder() -> CategoryBuilder {
        CategoryBuilder::default()
    }

    /// Create a mock `Category` with randomized data for testing purposes.
    ///
    /// Generates a fully populated Category instance with realistic random values
    /// for all fields. Useful for unit tests, integration tests, development fixtures,
    /// and seeding test databases with sample data.
    ///
    /// # Generated Data
    ///
    /// - **ID**: Time-ordered UUID v7 for realistic sorting
    /// - **Code**: Random XXX.XXX.XXX format (e.g., "A1B.C3D.4EF")
    /// - **Name**: 1-2 random words
    /// - **Description**: 50% chance of 5-8 word description
    /// - **Slug**: 50% chance of URL-safe slug derived from name
    /// - **Category Type**: Random accounting type (Asset/Liability/Income/Expense/Equity)
    /// - **Color**: 50% chance of random hex color code
    /// - **Icon**: 50% chance of random icon identifier
    /// - **Active Status**: 80% chance of being active
    /// - **Timestamps**: Realistic creation and update times
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(test)]
    /// # {
    /// use personal_ledger_backend::database::categories::Category;
    ///
    /// // Generate a single mock category
    /// let mock_category = Category::mock();
    /// assert!(!mock_category.code.is_empty());
    /// assert!(!mock_category.name.is_empty());
    ///
    /// // Generate multiple categories for testing
    /// let categories: Vec<Category> = (0..10)
    ///     .map(|_| Category::mock())
    ///     .collect();
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// This function is only available in test builds (`#[cfg(test)]`).
    /// Each invocation produces different random data.
    #[cfg(test)]
    pub fn mock() -> Self {
        use fake::Fake;
        use fake::faker::boolean::en::Boolean;
        use fake::faker::chrono::en::DateTimeAfter;
        use fake::faker::color::en::HexColor;
        use fake::faker::lorem::en::*;

        let random_id = domain::RowID::mock();

        let random_code = generate_random_category_code();

        let random_name: Vec<String> = Words(1..3).fake();
        let random_name = random_name.join(" ");

        let random_description: Vec<String> = Words(5..9).fake();
        let random_description = random_description.join(" ");
        let random_description = if Boolean(50).fake() {
            Some(random_description)
        } else {
            None
        };

        let random_slug = domain::UrlSlug::parse(&random_name).ok();
        let random_slug = if Boolean(50).fake() {
            random_slug
        } else {
            None
        };

        let random_category = domain::CategoryTypes::mock();

        let random_color = HexColor().fake();
        let random_color = if Boolean(50).fake() {
            Some(random_color)
        } else {
            None
        };

        let random_icon: Vec<String> = Words(1..2).fake();
        let random_icon = random_icon.join("-");
        let random_icon = if Boolean(50).fake() {
            Some(random_icon)
        } else {
            None
        };

        let random_active: bool = Boolean(80).fake();

        let random_created_on = DateTimeAfter(chrono::DateTime::UNIX_EPOCH).fake();

        let random_updated_on = DateTimeAfter(random_created_on).fake();

        Self {
            id: random_id,
            code: random_code,
            name: random_name,
            description: random_description,
            slug: random_slug,
            category_type: random_category,
            color: random_color,
            icon: random_icon,
            is_active: random_active,
            created_on: random_created_on,
            updated_on: random_updated_on,
        }
    }
}

/// Generate a random category code in the format "XXX.XXX.XXX".
///
/// Creates a unique-looking identifier for categories using uppercase letters and
/// digits separated by periods. Each segment contains 3 characters, providing a
/// total of 9 alphanumeric characters arranged as XXX.XXX.XXX.
///
/// # Format
///
/// The generated code follows the pattern: `[A-Z0-9]{3}.[A-Z0-9]{3}.[A-Z0-9]{3}`
///
/// # Returns
///
/// A string of exactly 11 characters (9 alphanumeric + 2 periods).
///
/// # Examples
///
/// ```rust
/// # #[cfg(test)]
/// # {
/// # use personal_ledger_backend::database::categories::model::generate_random_category_code;
/// let code = generate_random_category_code();
/// // Returns something like "A1B.C3D.4EF" or "XY2.9AB.CD7"
/// assert_eq!(code.len(), 11); // 9 chars + 2 periods
/// assert!(code.chars().nth(3) == Some('.'));
/// assert!(code.chars().nth(7) == Some('.'));
/// # }
/// ```
///
/// # Note
///
/// This function is only available in test builds (`#[cfg(test)]`).
/// Each invocation produces a different random code.
#[cfg(test)]
fn generate_random_category_code() -> String {
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

/// Builder for creating `Category` instances with a fluent API.
///
/// Provides a flexible, type-safe way to construct `Category` objects with
/// incremental configuration. The builder pattern allows setting fields in any
/// order and provides clear separation between required and optional fields.
///
/// # Required Fields
///
/// The following fields must be set before calling `build()`, or it will panic:
/// - `code`: Category identifier
/// - `name`: Display name
/// - `category_type`: Accounting classification
///
/// # Optional Fields with Defaults
///
/// - `description`: Defaults to `None`
/// - `slug`: Defaults to `None`
/// - `color`: Defaults to `None`
/// - `icon`: Defaults to `None`
/// - `is_active`: Defaults to `true`
///
/// # Automatic Fields
///
/// These fields are automatically generated by `build()`:
/// - `id`: New UUID v7 identifier
/// - `created_on`: Current UTC timestamp
/// - `updated_on`: Current UTC timestamp (same as `created_on`)
///
/// # Examples
///
/// ```rust
/// use personal_ledger_backend::database::categories::Category;
/// use personal_ledger_backend::domain::CategoryTypes;
///
/// // Minimal category with only required fields
/// let category = Category::builder()
///     .code("UTIL")
///     .name("Utilities")
///     .category_type(CategoryTypes::Expense)
///     .build();
///
/// // Fully configured category with all optional fields
/// let category = Category::builder()
///     .code("FOOD-001")
///     .name("Groceries")
///     .category_type(CategoryTypes::Expense)
///     .description("Weekly grocery shopping and food expenses")
///     .slug("groceries")  // Accepts String, &str, or UrlSlug
///     .color("#4A90E2")
///     .icon("shopping-cart")
///     .is_active(true)
///     .build();
///
/// // Methods can be chained in any order
/// let category = Category::builder()
///     .is_active(false)
///     .category_type(CategoryTypes::Income)
///     .slug("old-income-source")
///     .name("Deprecated Income")
///     .code("OLD-INC")
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct CategoryBuilder {
    code: Option<String>,
    name: Option<String>,
    description: Option<String>,
    slug: Option<domain::UrlSlug>,
    category_type: Option<domain::CategoryTypes>,
    color: Option<String>,
    icon: Option<String>,
    is_active: Option<bool>,
}

impl CategoryBuilder {
    /// Set the category code (required).
    ///
    /// The code is a short, unique identifier for the category. It should be
    /// descriptive but concise, typically using uppercase letters, numbers,
    /// and hyphens. Codes are case-insensitive in the database.
    ///
    /// # Arguments
    ///
    /// * `code` - Any type that can be converted into a `String` (&str or String)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// let category = Category::builder()
    ///     .code("UTIL-001")      // Using &str
    ///     .name("Utilities")
    ///     .category_type(CategoryTypes::Expense)
    ///     .build();
    ///
    /// let category = Category::builder()
    ///     .code("FOOD".to_string())  // Using String
    ///     .name("Food")
    ///     .category_type(CategoryTypes::Expense)
    ///     .build();
    /// ```
    pub fn code<S: Into<String>>(mut self, code: S) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Set the category name (required).
    ///
    /// The name is the human-readable display name shown in user interfaces.
    /// It should be clear, descriptive, and easily understood by users.
    ///
    /// # Arguments
    ///
    /// * `name` - Any type that can be converted into a `String` (&str or String)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// let category = Category::builder()
    ///     .code("UTIL")
    ///     .name("Monthly Utilities")  // Clear, descriptive name
    ///     .category_type(CategoryTypes::Expense)
    ///     .build();
    ///
    /// let category = Category::builder()
    ///     .code("SALARY")
    ///     .name("Employment Income")
    ///     .category_type(CategoryTypes::Income)
    ///     .build();
    /// ```
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set an optional description for the category.
    ///
    /// Provides additional context and details about the category's purpose,
    /// intended use, and what types of transactions should be classified here.
    /// While optional, a good description helps users understand when to use
    /// this category.
    ///
    /// # Arguments
    ///
    /// * `description` - Any type that can be converted into a `String`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// let category = Category::builder()
    ///     .code("UTIL")
    ///     .name("Utilities")
    ///     .category_type(CategoryTypes::Expense)
    ///     .description("Regular monthly utility payments including electricity, water, and gas")
    ///     .build();
    /// ```
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set an optional URL-friendly slug for the category.
    ///
    /// The slug is automatically parsed into a URL-safe format (lowercase,
    /// alphanumeric, and hyphens only). Accepts `String`, `&str`, or pre-validated
    /// `UrlSlug` instances. The slug must be unique across all categories when present.
    ///
    /// # Arguments
    ///
    /// * `slug` - String, &str, or UrlSlug that will be converted to URL-safe format
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::domain::{CategoryTypes, UrlSlug};
    ///
    /// // Using a string (automatically parsed)
    /// let category = Category::builder()
    ///     .code("UTIL")
    ///     .name("Monthly Utilities")
    ///     .category_type(CategoryTypes::Expense)
    ///     .slug("Monthly Utilities")  // Becomes "monthly-utilities"
    ///     .build();
    ///
    /// // Using a pre-validated UrlSlug
    /// let slug = UrlSlug::parse("monthly utilities").unwrap();
    /// let category = Category::builder()
    ///     .code("UTIL")
    ///     .name("Utilities")
    ///     .category_type(CategoryTypes::Expense)
    ///     .slug(slug)
    ///     .build();
    /// ```
    pub fn slug<T: Into<domain::UrlSlug>>(mut self, slug: T) -> Self {
        self.slug = Some(slug.into());
        self
    }

    /// Set the accounting category type (required).
    ///
    /// Determines the category's classification in the accounting system and
    /// how it appears in financial statements. Must be one of the five fundamental
    /// accounting types.
    ///
    /// # Arguments
    ///
    /// * `category_type` - One of: Asset, Liability, Income, Expense, or Equity
    ///
    /// # Category Types
    ///
    /// - **Asset**: Resources owned (cash, equipment, inventory)
    /// - **Liability**: Obligations owed (loans, accounts payable)
    /// - **Income**: Revenue earned (salary, sales, interest)
    /// - **Expense**: Costs incurred (rent, utilities, supplies)
    /// - **Equity**: Owner's stake (capital, retained earnings)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// // Expense category
    /// let category = Category::builder()
    ///     .code("FOOD")
    ///     .name("Groceries")
    ///     .category_type(CategoryTypes::Expense)
    ///     .build();
    ///
    /// // Income category
    /// let category = Category::builder()
    ///     .code("SALARY")
    ///     .name("Monthly Salary")
    ///     .category_type(CategoryTypes::Income)
    ///     .build();
    ///
    /// // Asset category
    /// let category = Category::builder()
    ///     .code("SAVINGS")
    ///     .name("Savings Account")
    ///     .category_type(CategoryTypes::Asset)
    ///     .build();
    /// ```
    pub fn category_type(mut self, category_type: domain::CategoryTypes) -> Self {
        self.category_type = Some(category_type);
        self
    }

    /// Set an optional color for visual identification.
    ///
    /// Hex color code used in charts, graphs, and UI elements for quick visual
    /// recognition of the category. Should be in #RRGGBB format.
    ///
    /// # Arguments
    ///
    /// * `color` - Hex color string in #RRGGBB format (e.g., "#4CAF50")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// let category = Category::builder()
    ///     .code("FOOD")
    ///     .name("Groceries")
    ///     .category_type(CategoryTypes::Expense)
    ///     .color("#4CAF50")  // Green for food expenses
    ///     .build();
    ///
    /// let category = Category::builder()
    ///     .code("URGENT")
    ///     .name("Urgent Expenses")
    ///     .category_type(CategoryTypes::Expense)
    ///     .color("#FF5733")  // Red for urgent items
    ///     .build();
    /// ```
    pub fn color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set an optional icon identifier for UI rendering.
    ///
    /// Icon identifier typically corresponds to icon libraries like Font Awesome,
    /// Material Icons, or similar. Used for visual representation in user interfaces.
    ///
    /// # Arguments
    ///
    /// * `icon` - Icon identifier string (e.g., "shopping-cart", "dollar-sign")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// let category = Category::builder()
    ///     .code("FOOD")
    ///     .name("Groceries")
    ///     .category_type(CategoryTypes::Expense)
    ///     .icon("shopping-cart")  // Font Awesome icon
    ///     .build();
    ///
    /// let category = Category::builder()
    ///     .code("TRANSPORT")
    ///     .name("Transportation")
    ///     .category_type(CategoryTypes::Expense)
    ///     .icon("car")
    ///     .build();
    /// ```
    pub fn icon<S: Into<String>>(mut self, icon: S) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set whether the category is active (defaults to `true`).
    ///
    /// Controls the visibility of the category in selection interfaces for new
    /// transactions. When set to `false`, the category is hidden from dropdown
    /// menus and selection lists but remains in the database to preserve historical
    /// transaction data integrity.
    ///
    /// # Arguments
    ///
    /// * `is_active` - `true` to make category available, `false` to hide it
    ///
    /// # Use Cases
    ///
    /// - Deprecating old categories without losing historical data
    /// - Temporarily disabling categories during reorganization
    /// - Archiving seasonal or one-time categories
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// // Active category (default behavior)
    /// let category = Category::builder()
    ///     .code("CURRENT")
    ///     .name("Current Category")
    ///     .category_type(CategoryTypes::Expense)
    ///     .is_active(true)
    ///     .build();
    ///
    /// // Deactivated category (hidden from selection)
    /// let category = Category::builder()
    ///     .code("OLD")
    ///     .name("Deprecated Category")
    ///     .category_type(CategoryTypes::Expense)
    ///     .is_active(false)
    ///     .build();
    /// ```
    pub fn is_active(mut self, is_active: bool) -> Self {
        self.is_active = Some(is_active);
        self
    }

    /// Build and finalize the `Category` instance.
    ///
    /// Constructs a new `Category` with all configured values. Automatically generates
    /// a new UUID v7 identifier and sets creation/update timestamps to the current UTC time.
    ///
    /// # Required Fields
    ///
    /// Must set before calling `build()`, or this method will panic:
    /// - `code` - Category identifier
    /// - `name` - Display name
    /// - `category_type` - Accounting classification
    ///
    /// # Defaults
    ///
    /// - `is_active`: `true` if not explicitly set
    /// - `description`, `slug`, `color`, `icon`: `None` if not set
    ///
    /// # Automatic Fields
    ///
    /// - `id`: New UUID v7 identifier (time-ordered)
    /// - `created_on`: Current UTC timestamp
    /// - `updated_on`: Current UTC timestamp (equal to `created_on`)
    ///
    /// # Panics
    ///
    /// Panics with a descriptive message if any required field is missing:
    /// - "Category code is required"
    /// - "Category name is required"
    /// - "Category type is required"
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// // Minimal category with required fields only
    /// let category = Category::builder()
    ///     .code("UTIL")
    ///     .name("Utilities")
    ///     .category_type(CategoryTypes::Expense)
    ///     .build();
    ///
    /// // Fully configured category
    /// let category = Category::builder()
    ///     .code("FOOD-001")
    ///     .name("Groceries")
    ///     .category_type(CategoryTypes::Expense)
    ///     .description("Weekly grocery shopping")
    ///     .slug("groceries")
    ///     .color("#4CAF50")
    ///     .icon("shopping-cart")
    ///     .is_active(true)
    ///     .build();
    /// ```
    ///
    /// # Panics Example
    ///
    /// ```should_panic
    /// use personal_ledger_backend::database::categories::Category;
    ///
    /// // This will panic: missing required fields
    /// let category = Category::builder().build();
    /// ```
    ///     .code("UTIL")
    ///     .name("Utilities")
    ///     .category_type(CategoryTypes::Expense)
    ///     .build();
    /// ```
    pub fn build(self) -> Category {
        let now = chrono::Utc::now();

        Category {
            id: domain::RowID::new(),
            code: self.code.expect("Category code is required"),
            name: self.name.expect("Category name is required"),
            description: self.description,
            slug: self.slug,
            category_type: self.category_type.expect("Category type is required"),
            color: self.color,
            icon: self.icon,
            is_active: self.is_active.unwrap_or(true),
            created_on: now,
            updated_on: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CategoryTypes, UrlSlug};

    #[test]
    fn test_category_builder_basic() {
        let category = Category::builder()
            .code("TEST-001")
            .name("Test Category")
            .category_type(CategoryTypes::Expense)
            .build();

        assert_eq!(category.code, "TEST-001");
        assert_eq!(category.name, "Test Category");
        assert_eq!(category.category_type, CategoryTypes::Expense);
        assert!(category.is_active);
        assert!(!category.id.to_string().is_empty()); // Should have generated an ID
        assert!(category.created_on <= chrono::Utc::now());
        assert!(category.updated_on <= chrono::Utc::now());
    }

    #[test]
    fn test_category_builder_with_all_fields() {
        let slug = UrlSlug::parse("test-category").unwrap();
        let category = Category::builder()
            .code("FOOD-001")
            .name("Groceries")
            .description("Food and grocery expenses")
            .slug(slug.clone())
            .category_type(CategoryTypes::Expense)
            .color("#4CAF50")
            .icon("shopping-cart")
            .is_active(true)
            .build();

        assert_eq!(category.code, "FOOD-001");
        assert_eq!(category.name, "Groceries");
        assert_eq!(
            category.description,
            Some("Food and grocery expenses".to_string())
        );
        assert_eq!(category.slug, Some(slug));
        assert_eq!(category.category_type, CategoryTypes::Expense);
        assert_eq!(category.color, Some("#4CAF50".to_string()));
        assert_eq!(category.icon, Some("shopping-cart".to_string()));
        assert!(category.is_active);
    }

    #[test]
    fn test_category_builder_slug_from_string() {
        let category = Category::builder()
            .code("TEST")
            .name("Test")
            .category_type(CategoryTypes::Asset)
            .slug("Test Category Slug")
            .build();

        assert_eq!(
            category.slug.as_ref().unwrap().as_str(),
            "test-category-slug"
        );
    }

    #[test]
    fn test_category_builder_slug_from_url_slug() {
        let slug = UrlSlug::new("pre-validated-slug").unwrap();
        let category = Category::builder()
            .code("TEST")
            .name("Test")
            .category_type(CategoryTypes::Asset)
            .slug(slug.clone())
            .build();

        assert_eq!(category.slug, Some(slug));
    }

    #[test]
    fn test_category_builder_defaults() {
        let category = Category::builder()
            .code("TEST")
            .name("Test")
            .category_type(CategoryTypes::Asset)
            .build();

        assert_eq!(category.description, None);
        assert_eq!(category.slug, None);
        assert_eq!(category.color, None);
        assert_eq!(category.icon, None);
        assert!(category.is_active); // Default value
    }

    #[test]
    fn test_category_builder_inactive() {
        let category = Category::builder()
            .code("TEST")
            .name("Test")
            .category_type(CategoryTypes::Asset)
            .is_active(false)
            .build();

        assert!(!category.is_active);
    }

    #[test]
    #[should_panic(expected = "Category code is required")]
    fn test_category_builder_missing_code() {
        Category::builder()
            .name("Test")
            .category_type(CategoryTypes::Asset)
            .build();
    }

    #[test]
    #[should_panic(expected = "Category name is required")]
    fn test_category_builder_missing_name() {
        Category::builder()
            .code("TEST")
            .category_type(CategoryTypes::Asset)
            .build();
    }

    #[test]
    #[should_panic(expected = "Category type is required")]
    fn test_category_builder_missing_category_type() {
        Category::builder().code("TEST").name("Test").build();
    }

    #[test]
    fn test_category_equality() {
        let category1 = Category::builder()
            .code("TEST")
            .name("Test")
            .category_type(CategoryTypes::Asset)
            .build();

        // Create category2 with same values but different timestamps
        let mut category2 = Category::builder()
            .code("TEST")
            .name("Test")
            .category_type(CategoryTypes::Asset)
            .build();

        // Set same timestamps for equality comparison
        category2.created_on = category1.created_on;
        category2.updated_on = category1.updated_on;
        category2.id = category1.id;

        assert_eq!(category1, category2);
    }

    #[test]
    fn test_category_debug() {
        let category = Category::builder()
            .code("TEST")
            .name("Test")
            .category_type(CategoryTypes::Asset)
            .build();

        let debug_str = format!("{:?}", category);
        assert!(debug_str.contains("TEST"));
        assert!(debug_str.contains("Test"));
        assert!(debug_str.contains("Asset"));
    }

    #[test]
    fn test_category_clone() {
        let category1 = Category::builder()
            .code("TEST")
            .name("Test")
            .category_type(CategoryTypes::Asset)
            .build();

        let category2 = category1.clone();
        assert_eq!(category1, category2);
    }

    #[test]
    fn test_category_serialization() {
        let slug = UrlSlug::parse("test-category").unwrap();
        let category = Category::builder()
            .code("TEST")
            .name("Test Category")
            .description("A test category")
            .slug(slug.clone())
            .category_type(CategoryTypes::Expense)
            .color("#FF5733")
            .icon("test-icon")
            .is_active(false)
            .build();

        // Test serialization
        let serialized = serde_json::to_string(&category).unwrap();
        let deserialized: Category = serde_json::from_str(&serialized).unwrap();

        assert_eq!(category.code, deserialized.code);
        assert_eq!(category.name, deserialized.name);
        assert_eq!(category.description, deserialized.description);
        assert_eq!(category.slug, deserialized.slug);
        assert_eq!(category.category_type, deserialized.category_type);
        assert_eq!(category.color, deserialized.color);
        assert_eq!(category.icon, deserialized.icon);
        assert_eq!(category.is_active, deserialized.is_active);
    }

    #[test]
    fn test_mock_generation() {
        let mock = Category::mock();

        // Check that required fields are set
        assert!(!mock.code.is_empty());
        assert!(!mock.name.is_empty());
        assert!(!mock.id.to_string().is_empty());

        // Check code format (should be XXX.XXX.XXX)
        assert_eq!(mock.code.len(), 11);
        assert!(mock.code.chars().nth(3).unwrap() == '.');
        assert!(mock.code.chars().nth(7).unwrap() == '.');
        assert!(
            mock.code
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.')
        );
    }

    #[test]
    fn test_mock_variety() {
        // Generate multiple mocks to ensure variety
        let mocks: Vec<Category> = (0..10).map(|_| Category::mock()).collect();

        // Check that not all codes are the same (very unlikely if random)
        let codes: std::collections::HashSet<String> =
            mocks.iter().map(|c| c.code.clone()).collect();
        assert!(
            codes.len() > 1,
            "All mock codes were identical, randomness may be broken"
        );

        // Check that not all names are the same
        let names: std::collections::HashSet<String> =
            mocks.iter().map(|c| c.name.clone()).collect();
        assert!(
            names.len() > 1,
            "All mock names were identical, randomness may be broken"
        );
    }

    #[test]
    fn test_mock_function_comprehensive() {
        let mock = Category::mock();

        // Test that all required fields are properly set
        assert!(!mock.code.is_empty(), "Code should not be empty");
        assert!(!mock.name.is_empty(), "Name should not be empty");
        assert!(!mock.id.to_string().is_empty(), "ID should be generated");

        // Test code format (XXX.XXX.XXX)
        assert_eq!(mock.code.len(), 11, "Code should be 11 characters");
        assert_eq!(mock.code.chars().nth(3), Some('.'), "Code should have dot at position 3");
        assert_eq!(mock.code.chars().nth(7), Some('.'), "Code should have dot at position 7");

        // Test that code contains only valid characters
        for (i, c) in mock.code.chars().enumerate() {
            if i != 3 && i != 7 {
                assert!(c.is_ascii_uppercase() || c.is_ascii_digit(),
                       "Code character '{}' at position {} should be uppercase alphanumeric", c, i);
            }
        }

        // Test name properties
        assert!(!mock.name.is_empty(), "Name should have at least 1 character");
        assert!(mock.name.len() <= 50, "Name should not be excessively long");

        // Test slug generation (when present)
        if let Some(slug) = &mock.slug {
            assert!(!slug.as_str().is_empty(), "Slug should not be empty when present");
            // Slug should be derived from name, so it should contain some relation
            let slug_str = slug.as_str();
            assert!(slug_str.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                   "Slug '{}' should contain only lowercase letters, digits, and hyphens", slug_str);
        }

        // Test description (when present)
        if let Some(desc) = &mock.description {
            assert!(!desc.is_empty(), "Description should not be empty when present");
            assert!(desc.len() >= 5, "Description should be reasonably long when present");
        }

        // Test color (when present)
        if let Some(color) = &mock.color {
            assert!(color.starts_with('#'), "Color '{}' should start with #", color);
            assert_eq!(color.len(), 7, "Color '{}' should be 7 characters", color);
            assert!(color.chars().skip(1).all(|c| c.is_ascii_hexdigit()),
                   "Color '{}' should contain valid hex digits", color);
        }

        // Test icon (when present)
        if let Some(icon) = &mock.icon {
            assert!(!icon.is_empty(), "Icon should not be empty when present");
            assert!(icon.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                   "Icon '{}' should contain only lowercase letters and hyphens", icon);
        }

        // Test category type is valid
        let valid_types = [CategoryTypes::Asset, CategoryTypes::Liability,
                          CategoryTypes::Income, CategoryTypes::Expense, CategoryTypes::Equity];
        assert!(valid_types.contains(&mock.category_type),
               "Category type {:?} should be one of the valid types", mock.category_type);

        // Test timestamps
        assert!(mock.created_on >= chrono::DateTime::UNIX_EPOCH,
               "Created timestamp should be after Unix epoch");
        assert!(mock.updated_on >= mock.created_on,
               "Updated timestamp should be after or equal to created timestamp");

        // Test that the category is structurally valid (can be used in builder)
        // `is_active` is a boolean by type definition, so no runtime tautological check is required.
    }

    #[test]
    fn test_generate_random_category_code() {
        let code = generate_random_category_code();

        // Check format: XXX.XXX.XXX
        assert_eq!(code.len(), 11);
        assert_eq!(code.chars().nth(3), Some('.'));
        assert_eq!(code.chars().nth(7), Some('.'));

        // Check all characters are uppercase alphanumeric
        for (i, c) in code.chars().enumerate() {
            if i != 3 && i != 7 {
                assert!(
                    c.is_ascii_uppercase() || c.is_ascii_digit(),
                    "Character '{}' at position {} is not uppercase alphanumeric",
                    c,
                    i
                );
            }
        }
    }

    #[test]
    fn test_category_builder_fluent_api() {
        // Test that builder methods can be chained in any order
        let category = Category::builder()
            .category_type(CategoryTypes::Income)
            .name("Salary")
            .code("SALARY")
            .description("Monthly salary income")
            .icon("dollar-sign")
            .color("#00FF00")
            .slug("monthly-salary")
            .is_active(true)
            .build();

        assert_eq!(category.code, "SALARY");
        assert_eq!(category.name, "Salary");
        assert_eq!(category.category_type, CategoryTypes::Income);
        assert_eq!(
            category.description,
            Some("Monthly salary income".to_string())
        );
        assert_eq!(category.icon, Some("dollar-sign".to_string()));
        assert_eq!(category.color, Some("#00FF00".to_string()));
        assert_eq!(category.slug.as_ref().unwrap().as_str(), "monthly-salary");
        assert!(category.is_active);
    }

    #[test]
    fn test_category_timestamps() {
        let before = chrono::Utc::now();
        let category = Category::builder()
            .code("TEST")
            .name("Test")
            .category_type(CategoryTypes::Asset)
            .build();
        let after = chrono::Utc::now();

        assert!(category.created_on >= before);
        assert!(category.created_on <= after);
        assert!(category.updated_on >= before);
        assert!(category.updated_on <= after);
        assert_eq!(category.created_on, category.updated_on); // Should be equal for new categories
    }

    #[test]
    fn test_category_builder_method_types() {
        // Test that builder methods accept various string types
        let category = Category::builder()
            .code("STRING") // &str
            .name("String".to_string()) // String
            .description("desc") // &str
            .color("#FFF".to_string()) // String
            .icon("icon") // &str
            .category_type(CategoryTypes::Liability)
            .build();

        assert_eq!(category.code, "STRING");
        assert_eq!(category.name, "String");
        assert_eq!(category.description, Some("desc".to_string()));
        assert_eq!(category.color, Some("#FFF".to_string()));
        assert_eq!(category.icon, Some("icon".to_string()));
    }

    #[test]
    fn test_category_different_category_types() {
        let types = vec![
            CategoryTypes::Asset,
            CategoryTypes::Liability,
            CategoryTypes::Income,
            CategoryTypes::Expense,
            CategoryTypes::Equity,
        ];

        for category_type in types {
            let category = Category::builder()
                .code("TEST")
                .name("Test")
                .category_type(category_type.clone())
                .build();

            assert_eq!(category.category_type, category_type);
        }
    }
}
