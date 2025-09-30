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
/// Represents a financial category used to classify transactions and accounts.
/// Categories have a hierarchical type system (asset, liability, income, expense, equity)
/// and support additional metadata like colors, icons, and slugs for UI purposes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow, PartialEq)]
pub struct Category {
    /// Unique Row ID identifier for the category
    pub id: domain::RowID,

    /// Short, unique code for the category (case-insensitive)
    pub code: String,

    /// Display name for the category
    pub name: String,

    /// Optional longer description
    pub description: Option<String>,

    /// URL-friendly slug (optional, unique if present)
    pub slug: Option<String>,

    /// Category type: asset, liability, income, expense, or equity
    pub category_type: domain::CategoryTypes,

    /// Optional color code in hex format (#RRGGBB)
    pub color: Option<String>,

    /// Optional icon identifier
    pub icon: Option<String>,

    /// Whether the category is active and available for use
    pub is_active: bool,

    /// When the category was created
    pub created_on: chrono::DateTime<chrono::Utc>,

    /// When the category was last updated
    pub updated_on: chrono::DateTime<chrono::Utc>,
}

impl Category {
    /// Create a new CategoryBuilder for constructing a Category.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// let category = Category::builder()
    ///     .code("FOOD")
    ///     .name("Food & Dining")
    ///     .category_type(CategoryTypes::Expense)
    ///     .description("Expenses for food and dining")
    ///     .color("#FF5733")
    ///     .build();
    /// ```
    pub fn builder() -> CategoryBuilder {
        CategoryBuilder::default()
    }

    /// Create a mock Category with random data for testing.
    ///
    /// Generates a Category instance with realistic random values for all fields,
    /// useful for unit tests, integration tests, and development fixtures.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    ///
    /// let mock_category = Category::mock();
    /// assert!(!mock_category.code.is_empty());
    /// assert!(!mock_category.name.is_empty());
    /// ```
    #[cfg(test)]
    pub fn mock() -> Self {
        use fake::Fake;
        use fake::faker::lorem::en::*;
        use fake::faker::boolean::en::Boolean;
        use fake::faker::color::en::HexColor;
        use fake::faker::chrono::en::DateTimeAfter;

        let random_id = domain::RowID::mock();

        let random_code = generate_random_category_code();

        let random_name: Vec<String> = Words(1..3).fake();
        let random_name = random_name.join(" ");

        let random_description: Vec<String> = Words(5..9).fake();
        let random_description = random_description.join(" ");
        let random_description = if Boolean(50).fake() { Some(random_description) } else { None };

        let random_slug = random_name.to_lowercase().replace(" ", "-");
        let random_slug = if Boolean(50).fake() { Some(random_slug) } else { None };

        let random_category = domain::CategoryTypes::mock();

        let random_color = HexColor().fake();
        let random_color = if Boolean(50).fake() { Some(random_color) } else { None };
        
        let random_icon: Vec<String> = Words(1..2).fake();
        let random_icon = random_icon.join("-");
        let random_icon = if Boolean(50).fake() { Some(random_icon) } else { None };

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
/// Creates a unique identifier for categories using uppercase letters and digits
/// separated by periods. Each segment contains 3 characters, providing a total
/// of 9 alphanumeric characters arranged as XXX.XXX.XXX.
///
/// # Examples
///
/// ```rust
/// let code = generate_random_category_code();
/// // Returns something like "A1B.C3D.4EF"
/// assert_eq!(code.len(), 11); // 9 chars + 2 periods
/// ```
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

/// Builder for creating Category instances with a fluent API.
///
/// Provides a more flexible way to construct Category objects compared to
/// the basic constructor, allowing optional fields to be set incrementally.
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
///     .description("Monthly utility bills")
///     .color("#4A90E2")
///     .icon("zap")
///     .is_active(true)
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct CategoryBuilder {
    code: Option<String>,
    name: Option<String>,
    description: Option<String>,
    slug: Option<String>,
    category_type: Option<domain::CategoryTypes>,
    color: Option<String>,
    icon: Option<String>,
    is_active: Option<bool>,
}

impl CategoryBuilder {
    /// Set the category code.
    ///
    /// The code should be a short, unique identifier for the category.
    /// It's recommended to use uppercase letters and be descriptive but concise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    ///
    /// let builder = Category::builder().code("UTIL-001");
    /// ```
    pub fn code<S: Into<String>>(mut self, code: S) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Set the category name.
    ///
    /// The display name should be human-readable and descriptive.
    /// This is what users will see in the interface.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    ///
    /// let builder = Category::builder().name("Monthly Utilities");
    /// ```
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the category description.
    ///
    /// Provides additional context and details about the category's purpose
    /// and intended use. This field is optional but recommended for clarity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    ///
    /// let builder = Category::builder()
    ///     .description("Regular monthly utility payments including electricity and gas");
    /// ```
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the category slug.
    ///
    /// URL-friendly identifier used in web interfaces and APIs.
    /// Should be lowercase with hyphens instead of spaces.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    ///
    /// let builder = Category::builder().slug("monthly-utilities");
    /// ```
    pub fn slug<S: Into<String>>(mut self, slug: S) -> Self {
        self.slug = Some(slug.into());
        self
    }

    /// Set the category type.
    ///
    /// Specifies the accounting classification for this category.
    /// Must be one of: Asset, Liability, Income, Expense, or Equity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// let builder = Category::builder().category_type(CategoryTypes::Expense);
    /// ```
    pub fn category_type(mut self, category_type: domain::CategoryTypes) -> Self {
        self.category_type = Some(category_type);
        self
    }

    /// Set the category color (hex format).
    ///
    /// Color code in hexadecimal format for UI theming and visual identification.
    /// Should be in the format #RRGGBB (e.g., #FF5733 for red-orange).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    ///
    /// let builder = Category::builder().color("#4CAF50");
    /// ```
    pub fn color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set the category icon.
    ///
    /// Icon identifier for visual representation in user interfaces.
    /// Typically corresponds to icon libraries like Font Awesome or similar.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    ///
    /// let builder = Category::builder().icon("shopping-cart");
    /// ```
    pub fn icon<S: Into<String>>(mut self, icon: S) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set whether the category is active.
    ///
    /// Controls whether the category is available for new transactions.
    /// Inactive categories are hidden in selection interfaces but preserve
    /// historical data integrity. Defaults to `true` if not specified.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::Category;
    ///
    /// let builder = Category::builder().is_active(false); // Deactivate category
    /// ```
    pub fn is_active(mut self, is_active: bool) -> Self {
        self.is_active = Some(is_active);
        self
    }

    /// Build the Category instance.
    ///
    /// Creates a new `Category` with the configured values. Automatically generates
    /// a unique identifier and sets creation/update timestamps to the current time.
    /// Optional fields not set will use sensible defaults (`is_active` defaults to `true`).
    ///
    /// # Panics
    ///
    /// Panics if required fields (`code`, `name`, `category_type`) are not set.
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