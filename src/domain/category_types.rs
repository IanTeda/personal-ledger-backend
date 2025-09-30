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
    sqlx::Type,
    serde::Deserialize,
    serde::Serialize,
)]
#[sqlx(type_name = "category_type", rename_all = "lowercase")]
pub enum CategoryTypes {
    /// Resources owned that have economic value (cash, investments, property).
    Asset,

    /// Debts or obligations owed to others (loans, credit cards, mortgages).
    Liability,

    /// Money earned or received (salary, dividends, interest, sales).
    Income,
    
    /// Money spent or costs incurred (groceries, utilities, entertainment).
    #[default]
    Expense,
    
    /// Owner's residual interest in assets after deducting liabilities.
    Equity,
}

/// Error type for CategoryTypes parsing operations.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CategoryTypesError {
    /// The provided string is not a valid category type.
    #[error("Invalid category type: {0}")]
    InvalidCategoryType(String),
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
    /// let category = CategoryTypes::from_str("asset")?;
    /// assert_eq!(category, CategoryTypes::Asset);
    /// # Ok::<(), personal_ledger_backend::domain::CategoryTypesError>(())
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

    /// Returns true if the category type is Asset.
    pub fn is_asset(&self) -> bool {
        matches!(self, CategoryTypes::Asset)
    }

    /// Returns true if the category type is Liability.
    pub fn is_liability(&self) -> bool {
        matches!(self, CategoryTypes::Liability)
    }

    /// Returns true if the category type is Income.
    pub fn is_income(&self) -> bool {
        matches!(self, CategoryTypes::Income)
    }

    /// Returns true if the category type is Expense.
    pub fn is_expense(&self) -> bool {
        matches!(self, CategoryTypes::Expense)
    }

    /// Returns true if the category type is Equity.
    pub fn is_equity(&self) -> bool {
        matches!(self, CategoryTypes::Equity)
    }

    /// Returns true if the category type represents a debit balance in normal accounting.
    ///
    /// Assets and Expenses normally have debit balances.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// assert!(CategoryTypes::Asset.is_debit_normal());
    /// assert!(CategoryTypes::Expense.is_debit_normal());
    /// assert!(!CategoryTypes::Income.is_debit_normal());
    /// ```
    pub fn is_debit_normal(&self) -> bool {
        matches!(self, CategoryTypes::Asset | CategoryTypes::Expense)
    }

    /// Returns true if the category type represents a credit balance in normal accounting.
    ///
    /// Liabilities, Income, and Equity normally have credit balances.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// assert!(CategoryTypes::Liability.is_credit_normal());
    /// assert!(CategoryTypes::Income.is_credit_normal());
    /// assert!(!CategoryTypes::Asset.is_credit_normal());
    /// ```
    pub fn is_credit_normal(&self) -> bool {
        matches!(self, CategoryTypes::Liability | CategoryTypes::Income | CategoryTypes::Equity)
    }

    /// Returns true if the category affects the balance sheet.
    ///
    /// Assets, Liabilities, and Equity appear on the balance sheet.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// assert!(CategoryTypes::Asset.is_balance_sheet());
    /// assert!(!CategoryTypes::Income.is_balance_sheet());
    /// ```
    pub fn is_balance_sheet(&self) -> bool {
        matches!(self, CategoryTypes::Asset | CategoryTypes::Liability | CategoryTypes::Equity)
    }

    /// Returns true if the category affects the income statement.
    ///
    /// Income and Expense categories appear on the income statement.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// assert!(CategoryTypes::Income.is_income_statement());
    /// assert!(!CategoryTypes::Asset.is_income_statement());
    /// ```
    pub fn is_income_statement(&self) -> bool {
        matches!(self, CategoryTypes::Income | CategoryTypes::Expense)
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
    fn test_is_asset() {
        assert!(CategoryTypes::Asset.is_asset());
        assert!(!CategoryTypes::Liability.is_asset());
        assert!(!CategoryTypes::Income.is_asset());
        assert!(!CategoryTypes::Expense.is_asset());
        assert!(!CategoryTypes::Equity.is_asset());
    }

    #[test]
    fn test_is_liability() {
        assert!(!CategoryTypes::Asset.is_liability());
        assert!(CategoryTypes::Liability.is_liability());
        assert!(!CategoryTypes::Income.is_liability());
        assert!(!CategoryTypes::Expense.is_liability());
        assert!(!CategoryTypes::Equity.is_liability());
    }

    #[test]
    fn test_is_income() {
        assert!(!CategoryTypes::Asset.is_income());
        assert!(!CategoryTypes::Liability.is_income());
        assert!(CategoryTypes::Income.is_income());
        assert!(!CategoryTypes::Expense.is_income());
        assert!(!CategoryTypes::Equity.is_income());
    }

    #[test]
    fn test_is_expense() {
        assert!(!CategoryTypes::Asset.is_expense());
        assert!(!CategoryTypes::Liability.is_expense());
        assert!(!CategoryTypes::Income.is_expense());
        assert!(CategoryTypes::Expense.is_expense());
        assert!(!CategoryTypes::Equity.is_expense());
    }

    #[test]
    fn test_is_equity() {
        assert!(!CategoryTypes::Asset.is_equity());
        assert!(!CategoryTypes::Liability.is_equity());
        assert!(!CategoryTypes::Income.is_equity());
        assert!(!CategoryTypes::Expense.is_equity());
        assert!(CategoryTypes::Equity.is_equity());
    }

    #[test]
    fn test_is_debit_normal() {
        assert!(CategoryTypes::Asset.is_debit_normal());
        assert!(!CategoryTypes::Liability.is_debit_normal());
        assert!(!CategoryTypes::Income.is_debit_normal());
        assert!(CategoryTypes::Expense.is_debit_normal());
        assert!(!CategoryTypes::Equity.is_debit_normal());
    }

    #[test]
    fn test_is_credit_normal() {
        assert!(!CategoryTypes::Asset.is_credit_normal());
        assert!(CategoryTypes::Liability.is_credit_normal());
        assert!(CategoryTypes::Income.is_credit_normal());
        assert!(!CategoryTypes::Expense.is_credit_normal());
        assert!(CategoryTypes::Equity.is_credit_normal());
    }

    #[test]
    fn test_is_balance_sheet() {
        assert!(CategoryTypes::Asset.is_balance_sheet());
        assert!(CategoryTypes::Liability.is_balance_sheet());
        assert!(!CategoryTypes::Income.is_balance_sheet());
        assert!(!CategoryTypes::Expense.is_balance_sheet());
        assert!(CategoryTypes::Equity.is_balance_sheet());
    }

    #[test]
    fn test_is_income_statement() {
        assert!(!CategoryTypes::Asset.is_income_statement());
        assert!(!CategoryTypes::Liability.is_income_statement());
        assert!(CategoryTypes::Income.is_income_statement());
        assert!(CategoryTypes::Expense.is_income_statement());
        assert!(!CategoryTypes::Equity.is_income_statement());
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
    fn test_accounting_equation_helpers() {
        // Assets = Liabilities + Equity (balance sheet items)
        assert!(CategoryTypes::Asset.is_balance_sheet());
        assert!(CategoryTypes::Liability.is_balance_sheet());
        assert!(CategoryTypes::Equity.is_balance_sheet());
        
        // Income and Expenses affect the income statement
        assert!(CategoryTypes::Income.is_income_statement());
        assert!(CategoryTypes::Expense.is_income_statement());
        
        // Debit/Credit normal balances
        assert!(CategoryTypes::Asset.is_debit_normal());
        assert!(CategoryTypes::Expense.is_debit_normal());
        assert!(CategoryTypes::Liability.is_credit_normal());
        assert!(CategoryTypes::Income.is_credit_normal());
        assert!(CategoryTypes::Equity.is_credit_normal());
    }

    #[test]
    fn test_comprehensive_coverage() {
        // Test that all variants are covered in our functions
        for category_type in CategoryTypes::all() {
            // Every type should have a string representation
            assert!(!category_type.as_str().is_empty());
            
            // Every type should be parseable from its string representation
            assert_eq!(CategoryTypes::from_str(category_type.as_str()), Ok(category_type.clone()));
            
            // Every type should be either debit or credit normal (but not both)
            assert_ne!(category_type.is_debit_normal(), category_type.is_credit_normal());
            
            // Every type should be either balance sheet or income statement (but not both)
            assert_ne!(category_type.is_balance_sheet(), category_type.is_income_statement());
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
}