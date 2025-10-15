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
    #[cfg(test)]
    pub fn mock() -> Self {
        use fake::Fake;
        use fake::faker::boolean::en::Boolean;
        use fake::faker::lorem::en::Words;

        let random_id = domain::RowID::mock();

        let code:String = {
            use rand::Rng;

            // Generate 9 random alphanumeric chars, uppercase, then split into 3 groups
            let mut rng = rand::rng();
            let s: String = (&mut rng)
                .sample_iter(&rand::distr::Alphanumeric)
                .take(9)
                .map(|b| (b as char).to_ascii_uppercase())
                .collect();

            format!("{}.{}.{}", &s[0..3], &s[3..6], &s[6..9])
        };

        let words: Vec<String> = Words(1..3).fake();
        let name: String = words.join(" ");

        let description: Option<String> = {
            let is_some: bool = Boolean(50).fake(); // 50% chance of Some
            if is_some {
                let words: Vec<String> = Words(3..8).fake();
                Some(words.join(" "))
            } else {
                None
            }
        };

        let url_slug: Option<domain::UrlSlug> = Some(domain::UrlSlug::from(name.clone()));

        let category_type = domain::CategoryTypes::mock();

        let color: Option<domain::HexColor> = domain::HexColor::mock_with_option();

        let icon: Option<String> = {
            let is_some: bool = Boolean(50).fake(); // 50% chance of Some
            if is_some {
                let words: Vec<String> = Words(1..2).fake();
                Some(words.join(" "))
            } else {
                None
            }
        };

        let is_active: bool = true;

        let created_on: chrono::DateTime<chrono::Utc> = chrono::Utc::now();

        let updated_on: chrono::DateTime<chrono::Utc> = chrono::Utc::now();

        Self {
            id: random_id,
            code,
            name,
            description,
            url_slug,
            category_type,
            color,
            icon,
            is_active,
            created_on,
            updated_on,
        }
    }
}