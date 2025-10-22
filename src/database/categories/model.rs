use crate::{database, domain};


// TODO: Move code into a domain type
#[derive(Debug, sqlx::FromRow, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub struct Category {
    pub id: domain::RowID,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub url_slug: Option<domain::UrlSlug>,
    pub category_type: domain::CategoryTypes,
    pub color: Option<domain::HexColor>,
    pub icon: Option<String>,
    pub is_active: bool,
    pub created_on: chrono::DateTime<chrono::Utc>,
    pub updated_on: chrono::DateTime<chrono::Utc>,
}

impl database::Category {
    /// Generates a mock `Category` instance with randomized test data.
    ///
    /// This function creates realistic test data for categories, using the `fake` crate
    /// to randomise optional fields and text content. Useful for unit and integration tests.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use personal_ledger_backend::database::categories::model::Category;
    ///
    /// let mock_category = Category::mock();
    /// assert!(!mock_category.name.is_empty());
    /// ```
    #[cfg(test)]
    pub fn mock() -> Self {
        use crate::database::categories::CategoryBuilder;

        CategoryBuilder::new()
            .with_id(domain::RowID::mock())
            .with_code_opt(Some(Self::generate_mock_code()))
            .with_name(Self::generate_mock_name())
            .with_description_opt(Self::generate_mock_description())
            .with_url_slug_opt(Self::generate_mock_url_slug())
            .with_category_type(domain::CategoryTypes::mock())
            .with_color_opt(domain::HexColor::mock_with_option())
            .with_icon_opt(Self::generate_mock_icon())
            .with_is_active_opt(Some(Self::generate_mock_is_active()))
            .with_created_on_opt(Some(chrono::Utc::now()))
            .with_updated_on_opt(Some(chrono::Utc::now()))
            .build()
            .expect("Mock category should always build successfully")
    }

    #[cfg(test)]
    fn generate_mock_code() -> String {
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

    #[cfg(test)]
    fn generate_mock_name() -> String {
        use fake::Fake;
        use fake::faker::lorem::en::Words;

        let words: Vec<String> = Words(1..3).fake();
        words.join(" ")
    }

    #[cfg(test)]
    fn generate_mock_description() -> Option<String> {
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

    #[cfg(test)]
    fn generate_mock_url_slug() -> Option<domain::UrlSlug> {
        Some(domain::UrlSlug::from(Self::generate_mock_name()))
    }

    #[cfg(test)]
    fn generate_mock_icon() -> Option<String> {
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

    #[cfg(test)]
    fn generate_mock_is_active() -> bool {
        use fake::Fake;
        use fake::faker::boolean::en::Boolean;

        Boolean(80).fake() // 80% chance of active for more realistic data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_generates_valid_category() {
        let cat = Category::mock();
        assert!(!cat.name.is_empty());
        assert!(!cat.code.is_empty());
        assert!(cat.code.contains('.'));
        assert!(cat.url_slug.is_some());
        assert!(cat.created_on <= chrono::Utc::now());
        assert!(cat.updated_on <= chrono::Utc::now());
    }

    #[test]
    fn mock_randomizes_optional_fields() {
        // Run multiple times to check randomization
        let mut has_some_description = false;
        let mut has_none_description = false;
        let mut has_some_icon = false;
        let mut has_none_icon = false;
        let mut has_inactive = false;

        for _ in 0..50 {
            let cat = Category::mock();
            if cat.description.is_some() {
                has_some_description = true;
            } else {
                has_none_description = true;
            }
            if cat.icon.is_some() {
                has_some_icon = true;
            } else {
                has_none_icon = true;
            }
            if !cat.is_active {
                has_inactive = true;
            }
        }

        assert!(has_some_description && has_none_description, "Description should randomize");
        assert!(has_some_icon && has_none_icon, "Icon should randomize");
        assert!(has_inactive, "is_active should sometimes be false");
    }

    #[test]
    fn generate_mock_code_produces_valid_format() {
        let code = Category::generate_mock_code();
        assert_eq!(code.len(), 11); // XXX.XXX.XXX
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric() || c == '.'));
        let parts: Vec<&str> = code.split('.').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].len(), 3);
        assert_eq!(parts[1].len(), 3);
        assert_eq!(parts[2].len(), 3);
    }

    #[test]
    fn generate_mock_name_produces_non_empty_string() {
        let name = Category::generate_mock_name();
        assert!(!name.is_empty());
        assert!(name.chars().all(|c| c.is_alphabetic() || c.is_whitespace()));
    }

    #[test]
    fn generate_mock_description_randomizes() {
        let mut has_some = false;
        let mut has_none = false;
        for _ in 0..20 {
            let desc = Category::generate_mock_description();
            if desc.is_some() {
                has_some = true;
                assert!(!desc.as_ref().unwrap().is_empty());
            } else {
                has_none = true;
            }
        }
        assert!(has_some && has_none);
    }

    #[test]
    fn generate_mock_url_slug_uses_name() {
        let slug = Category::generate_mock_url_slug();
        assert!(slug.is_some());
        // Since UrlSlug::from handles parsing, just check it's not empty
        assert!(!slug.as_ref().unwrap().as_str().is_empty());
    }

    #[test]
    fn generate_mock_icon_randomizes() {
        let mut has_some = false;
        let mut has_none = false;
        for _ in 0..20 {
            let icon = Category::generate_mock_icon();
            if icon.is_some() {
                has_some = true;
                assert!(!icon.as_ref().unwrap().is_empty());
                assert!(icon.as_ref().unwrap().chars().all(|c| c.is_alphabetic()));
            } else {
                has_none = true;
            }
        }
        assert!(has_some && has_none);
    }

    #[test]
    fn generate_mock_is_active_randomizes() {
        let mut has_true = false;
        let mut has_false = false;
        for _ in 0..20 {
            let active = Category::generate_mock_is_active();
            if active {
                has_true = true;
            } else {
                has_false = true;
            }
        }
        assert!(has_true && has_false);
    }

    #[test]
    fn category_struct_derives_work() {
        let cat1 = Category::mock();
        let cat2 = cat1.clone();
        assert_eq!(cat1, cat2);

        // Test Debug (implicitly by using in assert)
        let debug_str = format!("{:?}", cat1);
        assert!(debug_str.contains("Category"));

        // Test Serialize/Deserialize
        let json = serde_json::to_string(&cat1).unwrap();
        let deserialized: Category = serde_json::from_str(&json).unwrap();
        assert_eq!(cat1, deserialized);
    }
}