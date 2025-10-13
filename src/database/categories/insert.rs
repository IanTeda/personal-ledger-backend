use crate::database::{Category, DatabaseResult};

impl Category {
        pub async fn insert(
        &self,
        database: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<Self> {
        // Use a transaction to insert, then select the inserted row. Using
        // a separate SELECT avoids RETURNING-related macro complexity on
        // SQLite while preserving compile-time checked `query_as!` for the read.
    // Perform insert and select using the pool as executor. Using a
    // transaction here caused Executor trait mismatch in this context,
    // and for our unit tests it's acceptable to perform separate
    // operations.
    sqlx::query!(
            r#"
                INSERT INTO categories (
                    id,
                    code,
                    name,
                    description,
                    slug,
                    category_type,
                    color,
                    icon,
                    is_active,
                    created_on,
                    updated_on
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            // Bind primitive/owned types
            self.id.to_string(),
            self.code,
            self.name,
            self.description,
            self.slug.as_ref().map(|s| s.as_ref().to_string()),
            self.category_type.as_str(),
            self.color,
            self.icon,
            self.is_active,
            self.created_on,
            self.updated_on
        )
        .execute(database)
        .await?;

        // Read back the inserted row with a compile-time checked mapping
        let database_record = sqlx::query_as!(
            Category,
            r#"
                SELECT
                    id              AS "id!: crate::domain::RowID",
                    code,
                    name,
                    description,
                    slug            AS "slug: Option<crate::domain::UrlSlug>",
                    category_type   AS "category_type!: crate::domain::CategoryTypes",
                    color,
                    icon,
                    is_active,
                    created_on      AS "created_on!: chrono::DateTime<chrono::Utc>",
                    updated_on      AS "updated_on!: chrono::DateTime<chrono::Utc>"
                FROM categories
                WHERE id = ?
            "#,
            self.id.to_string()
        )
        .fetch_one(database)
        .await?;
        Ok(database_record)
    }
}