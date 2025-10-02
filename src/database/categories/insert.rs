use crate::{config::{ConnectionPool, DbEngine}, database::{Category, DatabaseError}};

impl Category {

    pub async fn insert(
        &self,
        database: ConnectionPool,
    ) -> Result<Self, DatabaseError> {
        match database.kind {
            DbEngine::Postgres => {
                // Use runtime-checked query for Postgres and RETURNING to get the created record
                let record = sqlx::query_as::<_, Category>(
                    r#"
                    INSERT INTO categories (name, color, slug)
                    VALUES ($1, $2, $3)
                    RETURNING id, name, color, slug
                    "#
                )
                .bind(&self.name)
                .bind(&self.color)
                .bind(&self.slug)
                .fetch_one(&database.pool)
                .await?;
                Ok(record)
            }
            DbEngine::Sqlite => {
                // SQLite does not reliably support RETURNING across all versions; insert then fetch last id
                sqlx::query(
                    r#"
                    INSERT INTO categories (name, color, slug)
                    VALUES (?, ?, ?)
                    "#
                )
                .bind(&self.name)
                .bind(&self.color)
                .bind(&self.slug)
                .execute(&database.pool)
                .await?;

                // last_insert_rowid() returns an i64 for the row id
                let id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
                    .fetch_one(&database.pool)
                    .await?;

                let record = sqlx::query_as::<_, Category>(
                    r#"
                    SELECT id, name, color, slug FROM categories WHERE id = ?
                    "#
                )
                .bind(id)
                .fetch_one(&database.pool)
                .await?;
                Ok(record)
            }
        }
    }
}