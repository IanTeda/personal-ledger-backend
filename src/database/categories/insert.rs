use crate::database::{self, DatabaseResult};
use crate::domain;


impl database::Category {
    pub async fn insert(&self, pool: &sqlx::Pool<sqlx::Sqlite>) -> DatabaseResult<Self> {
        // 1) INSERT: SQLite uses `?` placeholders and does not reliably support
        // `RETURNING *` for compile-time checked macros. Execute the insert first.
        sqlx::query!(
            r#"
                INSERT INTO categories (id, code, name, description, is_active, created_on, updated_on)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            self.id,
            self.code,
            self.name,
            self.description,
            self.is_active,
            self.created_on,
            self.updated_on
        )
        .execute(pool)
        .await?;

        // 2) SELECT: Read back the inserted row with explicit type annotations
        // for UUID and chrono types to avoid NULL/mapping issues in SQLite.
        let category = sqlx::query_as!(
            database::Category,
            r#"
                SELECT
                    id          AS "id!: domain::RowID",
                    code,
                    name,
                    description,
                    is_active,
                    created_on  AS "created_on!: chrono::DateTime<chrono::Utc>",
                    updated_on  AS "updated_on!: chrono::DateTime<chrono::Utc>"
                FROM categories
                WHERE id = ?
            "#,
            self.id
        )
        .fetch_one(pool)
        .await?;

        Ok(category)
    }
}

#[cfg(test)]
pub mod tests {
    // Bring module into test scope
    use super::*;

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

        // Test inserting into database
    #[sqlx::test]
    async fn create_database_record(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let id = domain::RowID::now();
        let code: String = "TEST.001.001".to_string();
        let name: String = "Test Category".to_string();
        let description: Option<String> = Some("This is a test category".to_string());
        let is_active: bool = true;
        let created_on: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
        let updated_on: chrono::DateTime<chrono::Utc> = chrono::Utc::now();

        let new_category = database::Category {
            id,
            code,
            name,
            description,
            is_active,
            created_on,
            updated_on,
        };

        let database_record = new_category.insert(&pool).await?;

        assert_eq!(new_category, database_record);

        Ok(())
    }
}