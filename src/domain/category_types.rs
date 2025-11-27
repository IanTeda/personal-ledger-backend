//! # Category Types Domain Module
//!
//! This module defines the `CategoryTypes` enum representing the fundamental
//! accounting categories used in personal financial management.
//!
//! ## Category Types
//!
//! The five fundamental accounting categories are:
//! - **Asset**: Resources owned (cash, investments, property)
//! - **Liability**: Debts owed (loans, credit cards, mortgages)
//! - **Income**: Money earned (salary, dividends, interest)
//! - **Expense**: Money spent (groceries, utilities, entertainment)
//! - **Equity**: Net worth (assets minus liabilities)

/// Represents the fundamental accounting categories for financial transactions.
///
/// These categories follow the standard accounting equation:
/// Assets = Liabilities + Equity
///
/// Income increases assets or equity, while expenses decrease assets or increase liabilities.
///
/// # Examples
///
/// ```rust
/// use personal_ledger_backend::domain::CategoryTypes;
///
/// let category = CategoryTypes::Asset;
/// assert_eq!(category.as_str(), "asset");
/// assert!(category.is_asset());
/// ```
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum CategoryTypes {
    /// Resources owned that have economic value (cash, investments, property).
    Asset,

    /// Owner's Capital, Owner's Drawings (Withdrawals), Retained Earnings, Common Stock
    Equity,

    /// Money spent or costs incurred (groceries, utilities, entertainment).
    #[default]
    Expense,

    /// Money earned or received (salary, dividends, interest, sales).
    Income,

    /// Debts or obligations owed to others (loans, credit cards, mortgages).
    Liability,
}

/// Error type for CategoryTypes parsing operations.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CategoryTypesError {
    /// The provided string is not a valid category type.
    #[error("Invalid category type: {0}")]
    InvalidCategoryType(String),
}

impl std::fmt::Display for CategoryTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for CategoryTypes {
    type Err = CategoryTypesError;

    /// Parse a string to a CategoryTypes variant (case-insensitive).
    ///
    /// # Errors
    ///
    /// Returns `CategoryTypesError::InvalidCategoryType` if the string doesn't match any valid category type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::str::FromStr;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// let category = CategoryTypes::from_str("asset").unwrap();
    /// assert_eq!(category, CategoryTypes::Asset);
    ///
    /// // Invalid strings return an error
    /// assert!(CategoryTypes::from_str("invalid").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "asset" => Ok(CategoryTypes::Asset),
            "liability" => Ok(CategoryTypes::Liability),
            "income" => Ok(CategoryTypes::Income),
            "expense" => Ok(CategoryTypes::Expense),
            "equity" => Ok(CategoryTypes::Equity),
            _ => Err(CategoryTypesError::InvalidCategoryType(s.to_string())),
        }
    }
}

impl CategoryTypes {
    /// Returns the string representation of the category type (lowercase).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// assert_eq!(CategoryTypes::Asset.as_str(), "asset");
    /// assert_eq!(CategoryTypes::Expense.as_str(), "expense");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            CategoryTypes::Asset => "asset",
            CategoryTypes::Liability => "liability", 
            CategoryTypes::Income => "income",
            CategoryTypes::Expense => "expense",
            CategoryTypes::Equity => "equity",
        }
    }

    /// Returns all valid category types as a slice.
    ///
    /// Useful for validation, UI dropdowns, or iteration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// let all_types = CategoryTypes::all();
    /// assert_eq!(all_types.len(), 5);
    /// assert!(all_types.contains(&CategoryTypes::Asset));
    /// ```
    pub fn all() -> &'static [CategoryTypes] {
        &[
            CategoryTypes::Asset,
            CategoryTypes::Liability,
            CategoryTypes::Income,
            CategoryTypes::Expense,
            CategoryTypes::Equity,
        ]
    }

    /// Create a random CategoryTypes variant for testing.
    ///
    /// This method randomly selects one of the five category types using
    /// the `fake` crate, useful for generating test data and mock objects.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// let random_type = CategoryTypes::mock();
    /// // random_type will be one of: Asset, Liability, Income, Expense, or Equity
    /// ```
    #[cfg(test)]
    pub fn mock() -> Self {
        use fake::Fake;

        // Get all category types and randomly select one
        let all_types = Self::all();
        let random_index: usize = (0..all_types.len()).fake();
        all_types[random_index].clone()
    }

    /// Convert this CategoryTypes to the corresponding RPC CategoryType enum value as i32.
    ///
    /// This method provides a clean way to convert domain types to RPC types
    /// for use in gRPC service implementations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// let domain_type = CategoryTypes::Asset;
    /// let rpc_type_value = domain_type.to_rpc_i32();
    /// // rpc_type_value will be the i32 value of the corresponding rpc::CategoryType
    /// ```
    pub fn to_rpc_i32(&self) -> i32 {
        match self {
            CategoryTypes::Asset => crate::rpc::CategoryTypes::Asset as i32,
            CategoryTypes::Equity => crate::rpc::CategoryTypes::Equity as i32,
            CategoryTypes::Expense => crate::rpc::CategoryTypes::Expense as i32,
            CategoryTypes::Income => crate::rpc::CategoryTypes::Income as i32,
            CategoryTypes::Liability => crate::rpc::CategoryTypes::Liability as i32,
        }
    }

    /// Convert from the protobuf i32 enum value to CategoryTypes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// let asset = CategoryTypes::from_rpc_i32(crate::rpc::CategoryTypes::Asset as i32).unwrap();
    /// assert_eq!(asset, CategoryTypes::Asset);
    /// ```
    pub fn from_rpc_i32(value: i32) -> Result<Self, String> {
        match value {
            x if x == crate::rpc::CategoryTypes::Asset as i32 => Ok(CategoryTypes::Asset),
            x if x == crate::rpc::CategoryTypes::Equity as i32 => Ok(CategoryTypes::Equity),
            x if x == crate::rpc::CategoryTypes::Expense as i32 => Ok(CategoryTypes::Expense),
            x if x == crate::rpc::CategoryTypes::Income as i32 => Ok(CategoryTypes::Income),
            x if x == crate::rpc::CategoryTypes::Liability as i32 => Ok(CategoryTypes::Liability),
            _ => Err(format!("Invalid category type value: {}", value)),
        }
    }
}

// SQLx trait implementations for database integration

/// Implements SQLx `Type` trait for `CategoryTypes` to enable database storage.
///
/// This implementation allows `CategoryTypes` to be stored in any SQLx-supported
/// database using the "any" driver. Values are stored as strings in the database.
impl sqlx::Type<sqlx::Any> for CategoryTypes {
    fn type_info() -> sqlx::any::AnyTypeInfo {
        <String as sqlx::Type<sqlx::Any>>::type_info()
    }
}

// Also provide Sqlite-specific trait impls so the query_as! macro can resolve
// types for the sqlite driver at compile time.
impl sqlx::Type<sqlx::Sqlite> for CategoryTypes {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for CategoryTypes {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        use std::str::FromStr;
        Ok(CategoryTypes::from_str(&s).map_err(|e| format!("Invalid category type in DB: {}", e))?)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for CategoryTypes {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <String as sqlx::Encode<'q, sqlx::Sqlite>>::encode(self.as_str().to_string(), buf)
    }
}

/// Implements SQLx `Decode` trait for `CategoryTypes` to enable reading from database.
///
/// This implementation reads string values from the database and converts them
/// to `CategoryTypes` variants using the `FromStr` implementation. Invalid values
/// in the database will result in a decoding error.
impl<'r> sqlx::Decode<'r, sqlx::Any> for CategoryTypes {
    fn decode(value: sqlx::any::AnyValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        use std::str::FromStr;
        let s = <String as sqlx::Decode<sqlx::Any>>::decode(value)?;
        Ok(CategoryTypes::from_str(&s).map_err(|e| format!("Invalid category type in database: {}", e))?)
    }
}

/// Newtype wrapper for `chrono::DateTime<chrono::Utc>` to enable SQLx trait implementations.
///
/// This wrapper provides SQLx database integration for UTC timestamps, storing them
/// as RFC 3339 formatted strings in the database for maximum compatibility across
/// different database backends (SQLite and PostgreSQL).
///
/// # Examples
///
/// ```rust,no_run
/// use chrono::Utc;
/// // Note: UtcDateTime is internal and not exposed in the public API.
/// // It's used internally for database serialization.
/// let now = Utc::now();
/// // let wrapped = UtcDateTime::from(now);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UtcDateTime(pub chrono::DateTime<chrono::Utc>);

/// Implements SQLx `Type` trait for `UtcDateTime` to enable database storage.
///
/// Timestamps are stored as strings in RFC 3339 format for compatibility
/// across different database backends.
impl sqlx::Type<sqlx::Any> for UtcDateTime {
    fn type_info() -> sqlx::any::AnyTypeInfo {
        <String as sqlx::Type<sqlx::Any>>::type_info()
    }
}

/// Implements SQLx `Decode` trait for `UtcDateTime` to enable reading from database.
///
/// This implementation reads RFC 3339 formatted strings from the database and
/// converts them to UTC timestamps. Invalid datetime strings will result in
/// a decoding error.
impl<'r> sqlx::Decode<'r, sqlx::Any> for UtcDateTime {
    fn decode(value: sqlx::any::AnyValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Any>>::decode(value)?;
        Ok(UtcDateTime(
            chrono::DateTime::parse_from_rfc3339(&s)
                .map_err(|e| format!("Invalid datetime in database: {}", e))?
                .with_timezone(&chrono::Utc),
        ))
    }
}

/// Converts a `chrono::DateTime<Utc>` into a `UtcDateTime` wrapper.
impl From<chrono::DateTime<chrono::Utc>> for UtcDateTime {
    fn from(dt: chrono::DateTime<chrono::Utc>) -> Self {
        UtcDateTime(dt)
    }
}

/// Extracts the inner `chrono::DateTime<Utc>` from a `UtcDateTime` wrapper.
impl From<UtcDateTime> for chrono::DateTime<chrono::Utc> {
    fn from(udt: UtcDateTime) -> Self {
        udt.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_as_str() {
        assert_eq!(CategoryTypes::Asset.as_str(), "asset");
        assert_eq!(CategoryTypes::Liability.as_str(), "liability");
        assert_eq!(CategoryTypes::Income.as_str(), "income");
        assert_eq!(CategoryTypes::Expense.as_str(), "expense");
        assert_eq!(CategoryTypes::Equity.as_str(), "equity");
    }

    #[test]
    fn test_from_str_valid() {
        use std::str::FromStr;
        
        assert_eq!(CategoryTypes::from_str("asset"), Ok(CategoryTypes::Asset));
        assert_eq!(CategoryTypes::from_str("liability"), Ok(CategoryTypes::Liability));
        assert_eq!(CategoryTypes::from_str("income"), Ok(CategoryTypes::Income));
        assert_eq!(CategoryTypes::from_str("expense"), Ok(CategoryTypes::Expense));
        assert_eq!(CategoryTypes::from_str("equity"), Ok(CategoryTypes::Equity));
    }

    #[test]
    fn test_from_str_case_insensitive() {
        use std::str::FromStr;
        
        assert_eq!(CategoryTypes::from_str("ASSET"), Ok(CategoryTypes::Asset));
        assert_eq!(CategoryTypes::from_str("LiAbIlItY"), Ok(CategoryTypes::Liability));
        assert_eq!(CategoryTypes::from_str("InCoMe"), Ok(CategoryTypes::Income));
        assert_eq!(CategoryTypes::from_str("EXPENSE"), Ok(CategoryTypes::Expense));
        assert_eq!(CategoryTypes::from_str("equity"), Ok(CategoryTypes::Equity));
    }

    #[test]
    fn test_from_str_invalid() {
        use std::str::FromStr;
        
        assert!(CategoryTypes::from_str("invalid").is_err());
        assert!(CategoryTypes::from_str("").is_err());
        assert!(CategoryTypes::from_str("assets").is_err()); // plural
        assert!(CategoryTypes::from_str("unknown").is_err());
    }

    #[test]
    fn test_all() {
        let all_types = CategoryTypes::all();
        assert_eq!(all_types.len(), 5);
        assert!(all_types.contains(&CategoryTypes::Asset));
        assert!(all_types.contains(&CategoryTypes::Liability));
        assert!(all_types.contains(&CategoryTypes::Income));
        assert!(all_types.contains(&CategoryTypes::Expense));
        assert!(all_types.contains(&CategoryTypes::Equity));
    }

    #[test]
    fn test_default() {
        assert_eq!(CategoryTypes::default(), CategoryTypes::Expense);
    }

    #[test]
    fn test_serialization() {
        // Test all variants for consistent serialization
        let test_cases = [
            (CategoryTypes::Asset, "\"Asset\""),
            (CategoryTypes::Liability, "\"Liability\""),
            (CategoryTypes::Income, "\"Income\""),
            (CategoryTypes::Expense, "\"Expense\""),
            (CategoryTypes::Equity, "\"Equity\""),
        ];
        
        for (category_type, expected_json) in test_cases {
            // Test serialization
            let serialized = serde_json::to_string(&category_type).unwrap();
            assert_eq!(serialized, expected_json);
            
            // Test deserialization round-trip
            let deserialized: CategoryTypes = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized, category_type);
        }
    }

    #[test]
    fn test_error_display() {
        let error = CategoryTypesError::InvalidCategoryType("invalid_type".to_string());
        assert_eq!(format!("{}", error), "Invalid category type: invalid_type");
    }

    #[test]
    fn test_error_debug() {
        let error = CategoryTypesError::InvalidCategoryType("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidCategoryType"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_mock() {
        // Test that mock() returns valid CategoryTypes
        let mock_type = CategoryTypes::mock();
        
        // Should be one of the valid variants
        let all_types = CategoryTypes::all();
        assert!(all_types.contains(&mock_type));
        
        // Should have a valid string representation
        assert!(!mock_type.as_str().is_empty());
        
        // Should be parseable back from its string representation
        assert_eq!(CategoryTypes::from_str(mock_type.as_str()), Ok(mock_type));
    }

    #[test]
    fn test_to_rpc_i32() {
        // Test conversion to RPC i32 values
        assert_eq!(CategoryTypes::Asset.to_rpc_i32(), crate::rpc::CategoryTypes::Asset as i32);
        assert_eq!(CategoryTypes::Equity.to_rpc_i32(), crate::rpc::CategoryTypes::Equity as i32);
        assert_eq!(CategoryTypes::Expense.to_rpc_i32(), crate::rpc::CategoryTypes::Expense as i32);
        assert_eq!(CategoryTypes::Income.to_rpc_i32(), crate::rpc::CategoryTypes::Income as i32);
        assert_eq!(CategoryTypes::Liability.to_rpc_i32(), crate::rpc::CategoryTypes::Liability as i32);
    }

    #[test]
    fn test_from_rpc_i32() {
        // Test conversion from RPC i32 values
        assert_eq!(CategoryTypes::from_rpc_i32(crate::rpc::CategoryTypes::Asset as i32), Ok(CategoryTypes::Asset));
        assert_eq!(CategoryTypes::from_rpc_i32(crate::rpc::CategoryTypes::Equity as i32), Ok(CategoryTypes::Equity));
        assert_eq!(CategoryTypes::from_rpc_i32(crate::rpc::CategoryTypes::Expense as i32), Ok(CategoryTypes::Expense));
        assert_eq!(CategoryTypes::from_rpc_i32(crate::rpc::CategoryTypes::Income as i32), Ok(CategoryTypes::Income));
        assert_eq!(CategoryTypes::from_rpc_i32(crate::rpc::CategoryTypes::Liability as i32), Ok(CategoryTypes::Liability));
        
        // Invalid values should return an error
        assert!(CategoryTypes::from_rpc_i32(-1).is_err());
        assert!(CategoryTypes::from_rpc_i32(999).is_err());
    }
}