use crate::{database, domain};


// TODO: Move code into a domain type
#[derive(Debug, sqlx::FromRow, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub struct Category {
    pub id: domain::RowID,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub url_slug: Option<domain::UrlSlug>,
    pub is_active: bool,
    pub created_on: chrono::DateTime<chrono::Utc>,
    pub updated_on: chrono::DateTime<chrono::Utc>,
}

impl database::Category {
    #[cfg(test)]
    pub fn mock() -> Self {
        let random_id = domain::RowID::mock();

        let code: String = "TEST.001.001".to_string();

        let name: String = "Test Category".to_string();

        let description: Option<String> = Some("This is a test category".to_string());

        let url_slug: Option<domain::UrlSlug> = Some(domain::UrlSlug::from(name.clone()));

        let is_active: bool = true;

        let created_on: chrono::DateTime<chrono::Utc> = chrono::Utc::now();

        let updated_on: chrono::DateTime<chrono::Utc> = chrono::Utc::now();

        Self {
            id: random_id,
            code,
            name,
            description,
            url_slug,
            is_active,
            created_on,
            updated_on,
        }
    }
}