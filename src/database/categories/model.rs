use crate::{database, domain};


#[derive(Debug, sqlx::FromRow, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub struct Category {
    pub id: domain::RowID,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_on: chrono::DateTime<chrono::Utc>,
    pub updated_on: chrono::DateTime<chrono::Utc>,
}

impl database::Category {
    #[cfg(test)]
    pub fn mock() -> Self {
        let now_id = domain::RowID::now();

        let code: String = "TEST.001.001".to_string();

        let name: String = "Test Category".to_string();

        let description: Option<String> = Some("This is a test category".to_string());

        let is_active: bool = true;

        let created_on: chrono::DateTime<chrono::Utc> = chrono::Utc::now();

        let updated_on: chrono::DateTime<chrono::Utc> = chrono::Utc::now();

        Self {
            id: now_id,
            code,
            name,
            description,
            is_active,
            created_on,
            updated_on,
        }
    }
}