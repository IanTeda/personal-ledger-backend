//! # URL Slug Domain Type
//!
//! This module defines the `UrlSlug` domain type for representing URL-safe identifiers.
//! URL slugs are used in web applications to create human-readable, SEO-friendly URLs
//! from titles and names by converting them to lowercase, alphanumeric strings with hyphens.
//!
//! ## Features
//!
//! - **URL Safety**: Ensures slugs contain only lowercase letters, numbers, and hyphens
//! - **Automatic Cleaning**: Parses strings into valid slugs by removing special characters
//! - **Validation**: Prevents invalid characters and formats
//! - **Type Safety**: Prevents mixing slugs with regular strings
//! - **SEO Friendly**: Creates readable, search-engine optimized identifiers
//!
//! ## Example Usage
//!
//! ```rust
//! use personal_ledger_backend::domain::UrlSlug;
//!
//! // Parse a title into a slug
//! let slug = UrlSlug::parse("Hello World! How are you?")?;
//! assert_eq!(slug.as_str(), "hello-world-how-are-you");
//!
//! // Create from a known valid slug
//! let slug = UrlSlug::new("valid-slug-123")?;
//! ```

use std::fmt;

/// Represents a URL-safe slug string.
///
/// A slug is a string that contains only lowercase letters, numbers, and hyphens,
/// making it safe for use in URLs. Slugs are typically created from human-readable
/// titles by removing special characters and replacing spaces with hyphens.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct UrlSlug(String);

/// Errors that can occur when working with URL slugs.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum UrlSlugError {
    /// The slug contains invalid characters (only lowercase letters, numbers, and hyphens allowed).
    #[error("Invalid slug characters: {0}")]
    InvalidCharacters(String),
    /// The slug is empty.
    #[error("Slug cannot be empty")]
    EmptySlug,
    /// The slug starts or ends with a hyphen.
    #[error("Slug cannot start or end with hyphen: {0}")]
    StartsOrEndsWithHyphen(String),
    /// The slug contains consecutive hyphens.
    #[error("Slug cannot contain consecutive hyphens: {0}")]
    ConsecutiveHyphens(String),
}

impl UrlSlug {
    /// Create a UrlSlug from a string that is already a valid slug.
    ///
    /// This method validates that the input string follows slug rules but does not
    /// perform any cleaning or transformation. Use `parse()` for automatic cleaning.
    ///
    /// # Errors
    ///
    /// Returns `UrlSlugError` if the string is not a valid slug.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::UrlSlug;
    ///
    /// let slug = UrlSlug::new("valid-slug-123")?;
    /// assert_eq!(slug.as_str(), "valid-slug-123");
    /// # Ok::<(), personal_ledger_backend::domain::UrlSlugError>(())
    /// ```
    pub fn new<S: Into<String>>(s: S) -> Result<Self, UrlSlugError> {
        let s = s.into();
        Self::validate_slug(&s)?;
        Ok(UrlSlug(s))
    }

    /// Parse a string into a URL-safe slug.
    ///
    /// This method performs automatic cleaning:
    /// - Converts to lowercase
    /// - Replaces spaces and underscores with hyphens
    /// - Removes special characters (keeps only letters, numbers, hyphens)
    /// - Removes leading/trailing hyphens
    /// - Collapses consecutive hyphens into single hyphens
    ///
    /// # Errors
    ///
    /// Returns `UrlSlugError::EmptySlug` if the cleaned string is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::UrlSlug;
    ///
    /// let slug = UrlSlug::parse("Hello World! How are you?")?;
    /// assert_eq!(slug.as_str(), "hello-world-how-are-you");
    ///
    /// let slug2 = UrlSlug::parse("C++ Programming & Web Dev")?;
    /// assert_eq!(slug2.as_str(), "c-programming-web-dev");
    /// # Ok::<(), personal_ledger_backend::domain::UrlSlugError>(())
    /// ```
    pub fn parse<S: Into<String>>(s: S) -> Result<Self, UrlSlugError> {
        let s = s.into();
        let cleaned = Self::clean_string(&s);

        if cleaned.is_empty() {
            return Err(UrlSlugError::EmptySlug);
        }

        Ok(UrlSlug(cleaned))
    }

    /// Get the slug as a string slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::UrlSlug;
    ///
    /// let slug = UrlSlug::new("test-slug")?;
    /// assert_eq!(slug.as_str(), "test-slug");
    /// # Ok::<(), personal_ledger_backend::domain::UrlSlugError>(())
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert the slug into its underlying string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::domain::UrlSlug;
    ///
    /// let slug = UrlSlug::new("test-slug")?;
    /// let string = slug.into_string();
    /// assert_eq!(string, "test-slug");
    /// # Ok::<(), personal_ledger_backend::domain::UrlSlugError>(())
    /// ```
    pub fn into_string(self) -> String {
        self.0
    }

    /// Check if the slug is empty.
    ///
    /// Note: Empty slugs are not allowed, so this should always return false
    /// for valid UrlSlug instances.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the length of the slug in characters.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Clean a string to make it URL-safe.
    ///
    /// This is the internal cleaning logic used by `parse()`.
    fn clean_string(s: &str) -> String {
        s.chars()
            // Convert to lowercase
            .map(|c| c.to_lowercase().collect::<String>())
            .collect::<String>()
            // Replace spaces and underscores with hyphens
            .replace([' ', '_'], "-")
            // Keep only ASCII alphanumeric characters and hyphens
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect::<String>()
            // Remove leading/trailing hyphens
            .trim_matches('-')
            // Collapse consecutive hyphens
            .split('-')
            .filter(|part| !part.is_empty())
            .collect::<Vec<&str>>()
            .join("-")
    }

    /// Validate that a string is a valid slug.
    fn validate_slug(s: &str) -> Result<(), UrlSlugError> {
        if s.is_empty() {
            return Err(UrlSlugError::EmptySlug);
        }

        if s.starts_with('-') || s.ends_with('-') {
            return Err(UrlSlugError::StartsOrEndsWithHyphen(s.to_string()));
        }

        if s.contains("--") {
            return Err(UrlSlugError::ConsecutiveHyphens(s.to_string()));
        }

        // Check for invalid characters
        for c in s.chars() {
            if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' {
                return Err(UrlSlugError::InvalidCharacters(s.to_string()));
            }
        }

        Ok(())
    }
}

impl std::str::FromStr for UrlSlug {
    type Err = UrlSlugError;

    /// Parse a string into a UrlSlug using the same logic as `parse()`.
    ///
    /// This allows using `"string".parse::<UrlSlug>()` syntax.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        UrlSlug::parse(s)
    }
}

impl fmt::Display for UrlSlug {
    /// Format the slug as a string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for UrlSlug {
    /// Get the slug as a string slice.
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<UrlSlug> for String {
    /// Convert a UrlSlug into a String.
    fn from(slug: UrlSlug) -> Self {
        slug.0
    }
}

impl From<String> for UrlSlug {
    /// Convert a String into a UrlSlug by parsing it.
    fn from(s: String) -> Self {
        UrlSlug::parse(s).expect("Failed to parse string into UrlSlug")
    }
}

impl From<&str> for UrlSlug {
    /// Convert a &str into a UrlSlug by parsing it.
    fn from(s: &str) -> Self {
        UrlSlug::parse(s).expect("Failed to parse string into UrlSlug")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid_slug() {
        let slug = UrlSlug::new("valid-slug-123").unwrap();
        assert_eq!(slug.as_str(), "valid-slug-123");
    }

    #[test]
    fn test_new_invalid_characters() {
        let result = UrlSlug::new("invalid slug!");
        assert!(matches!(result, Err(UrlSlugError::InvalidCharacters(_))));
    }

    #[test]
    fn test_new_empty_slug() {
        let result = UrlSlug::new("");
        assert!(matches!(result, Err(UrlSlugError::EmptySlug)));
    }

    #[test]
    fn test_new_starts_with_hyphen() {
        let result = UrlSlug::new("-invalid");
        assert!(matches!(result, Err(UrlSlugError::StartsOrEndsWithHyphen(_))));
    }

    #[test]
    fn test_new_ends_with_hyphen() {
        let result = UrlSlug::new("invalid-");
        assert!(matches!(result, Err(UrlSlugError::StartsOrEndsWithHyphen(_))));
    }

    #[test]
    fn test_new_consecutive_hyphens() {
        let result = UrlSlug::new("invalid--slug");
        assert!(matches!(result, Err(UrlSlugError::ConsecutiveHyphens(_))));
    }

    #[test]
    fn test_parse_basic() {
        let slug = UrlSlug::parse("Hello World").unwrap();
        assert_eq!(slug.as_str(), "hello-world");
    }

    #[test]
    fn test_parse_special_characters() {
        let slug = UrlSlug::parse("C++ Programming & Web Dev!").unwrap();
        assert_eq!(slug.as_str(), "c-programming-web-dev");
    }

    #[test]
    fn test_parse_underscores() {
        let slug = UrlSlug::parse("snake_case_string").unwrap();
        assert_eq!(slug.as_str(), "snake-case-string");
    }

    #[test]
    fn test_parse_uppercase() {
        let slug = UrlSlug::parse("UPPERCASE TITLE").unwrap();
        assert_eq!(slug.as_str(), "uppercase-title");
    }

    #[test]
    fn test_parse_numbers() {
        let slug = UrlSlug::parse("Test 123 Numbers").unwrap();
        assert_eq!(slug.as_str(), "test-123-numbers");
    }

    #[test]
    fn test_parse_leading_trailing_spaces() {
        let slug = UrlSlug::parse("  spaced title  ").unwrap();
        assert_eq!(slug.as_str(), "spaced-title");
    }

    #[test]
    fn test_parse_consecutive_spaces() {
        let slug = UrlSlug::parse("multiple   spaces").unwrap();
        assert_eq!(slug.as_str(), "multiple-spaces");
    }

    #[test]
    fn test_parse_only_special_chars() {
        let result = UrlSlug::parse("!!!@@@###");
        assert!(matches!(result, Err(UrlSlugError::EmptySlug)));
    }

    #[test]
    fn test_parse_empty_string() {
        let result = UrlSlug::parse("");
        assert!(matches!(result, Err(UrlSlugError::EmptySlug)));
    }

    #[test]
    fn test_parse_only_spaces() {
        let result = UrlSlug::parse("   ");
        assert!(matches!(result, Err(UrlSlugError::EmptySlug)));
    }

    #[test]
    fn test_from_str_trait() {
        let slug: UrlSlug = "Test String".parse().unwrap();
        assert_eq!(slug.as_str(), "test-string");
    }

    #[test]
    fn test_display() {
        let slug = UrlSlug::new("test-slug").unwrap();
        assert_eq!(format!("{}", slug), "test-slug");
    }

    #[test]
    fn test_as_ref() {
        let slug = UrlSlug::new("test-slug").unwrap();
        let s: &str = slug.as_ref();
        assert_eq!(s, "test-slug");
    }

    #[test]
    fn test_into_string() {
        let slug = UrlSlug::new("test-slug").unwrap();
        let s: String = slug.into_string();
        assert_eq!(s, "test-slug");
    }

    #[test]
    fn test_from_into_string() {
        let original = "test-slug".to_string();
        let slug = UrlSlug::new(&original).unwrap();
        let result: String = slug.into();
        assert_eq!(result, original);
    }

    #[test]
    fn test_equality() {
        let slug1 = UrlSlug::new("test-slug").unwrap();
        let slug2 = UrlSlug::new("test-slug").unwrap();
        let slug3 = UrlSlug::new("different-slug").unwrap();

        assert_eq!(slug1, slug2);
        assert_ne!(slug1, slug3);
    }

    #[test]
    fn test_clone() {
        let slug1 = UrlSlug::new("test-slug").unwrap();
        let slug2 = slug1.clone();
        assert_eq!(slug1, slug2);
    }

    #[test]
    fn test_serialization() {
        let slug = UrlSlug::new("test-slug").unwrap();
        let serialized = serde_json::to_string(&slug).unwrap();
        assert_eq!(serialized, "\"test-slug\"");

        let deserialized: UrlSlug = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, slug);
    }

    #[test]
    fn test_complex_parsing() {
        let test_cases = vec![
            ("Hello World!", "hello-world"),
            ("C++ & Rust Programming", "c-rust-programming"),
            ("Multiple   Spaces", "multiple-spaces"),
            ("123 Numbers & Symbols!", "123-numbers-symbols"),
            ("_underscores_and-hyphens-", "underscores-and-hyphens"),
            ("Café résumé naïve", "caf-rsum-nave"), // Unicode characters removed
        ];

        for (input, expected) in test_cases {
            let slug = UrlSlug::parse(input).unwrap();
            assert_eq!(slug.as_str(), expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_edge_cases() {
        // Single character
        let slug = UrlSlug::parse("a").unwrap();
        assert_eq!(slug.as_str(), "a");

        // Only numbers
        let slug = UrlSlug::parse("123").unwrap();
        assert_eq!(slug.as_str(), "123");

        // Mixed alphanumeric
        let slug = UrlSlug::parse("abc123def").unwrap();
        assert_eq!(slug.as_str(), "abc123def");

        // Already valid slug
        let slug = UrlSlug::parse("already-valid-slug").unwrap();
        assert_eq!(slug.as_str(), "already-valid-slug");
    }
}