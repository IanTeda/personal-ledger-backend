use crate::database::{self, DatabaseResult};
use crate::domain;


impl database::Category {
    #[tracing::instrument(
        name = "Insert new Category into database: ",
        skip(self, pool),
        fields(
            id = % self.id,
            code = % self.code,
            name = % self.name,
            description = ? self.description,
            url_slug = ? self.url_slug,
            is_active = % self.is_active,
            created_on = % self.created_on,
            updated_on = % self.updated_on,
        ),
    )]
    pub async fn insert(&self, pool: &sqlx::Pool<sqlx::Sqlite>) -> DatabaseResult<Self> {
        // 1) INSERT: SQLite uses `?` placeholders and does not reliably support
        // `RETURNING *` for compile-time checked macros. Execute the insert first.
        let insert_query = sqlx::query!(
            r#"
                INSERT INTO categories (id, code, name, description, url_slug, is_active, created_on, updated_on)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            self.id,
            self.code,
            self.name,
            self.description,
            self.url_slug,
            self.is_active,
            self.created_on,
            self.updated_on
        );

        insert_query.execute(pool).await?;

        tracing::info!("New Category inserted into the database.");

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
                    url_slug    AS "url_slug?: domain::UrlSlug",
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

        tracing::debug!("Newly created Category retrived from the database.");

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
        let new_category = database::Category::mock();

        let database_record = new_category.insert(&pool).await?;

        assert_eq!(new_category, database_record);

        println!("Inserted category: {:?}", database_record);

        Ok(())
    }
}