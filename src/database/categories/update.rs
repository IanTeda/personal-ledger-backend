use crate::database::{self, DatabaseResult};
use crate::domain;

/// Update operations for Category database records.
///
/// This module provides functions for updating existing category records in the database,
/// including single record updates, bulk updates, and specialized update operations.
impl database::Categories {
    /// Updates an existing category in the database.
    ///
    /// This function updates all fields of the category record identified by the `id` field.
    /// The operation is atomic and will either succeed completely or fail without side effects.
    ///
    /// # Arguments
    ///
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns the updated category as read back from the database, or a
    /// `DatabaseError` if the update fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The category with the given ID does not exist
    /// - The updated category violates database constraints (duplicate code, name, or url_slug)
    /// - The category_type is invalid
    /// - The color format is invalid
    /// - Database connection fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// # async fn example(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// // First create a category
    /// let mut category = Category::mock();
    /// let inserted = category.insert(pool).await?;
    ///
    /// // Update the category
    /// let updated_category = Category {
    ///     name: "Updated Category Name".to_string(),
    ///     ..inserted
    /// };
    ///
    /// let result = updated_category.update(pool).await?;
    /// assert_eq!(result.name, "Updated Category Name");
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Update category in database",
        skip(self, pool),
        fields(
            id = % self.id,
            code = % self.code,
            name = % self.name
        ),
        err
    )]
    pub async fn update(&self, pool: &sqlx::Pool<sqlx::Sqlite>) -> DatabaseResult<Self> {
        // Update the category record
        let update_query = sqlx::query!(
            r#"
                UPDATE categories
                SET code = ?, name = ?, description = ?, url_slug = ?, category_type = ?,
                    color = ?, icon = ?, is_active = ?, updated_on = ?
                WHERE id = ?
            "#,
            self.code,
            self.name,
            self.description,
            self.url_slug,
            self.category_type,
            self.color,
            self.icon,
            self.is_active,
            self.updated_on,
            self.id
        );

        let rows_affected = update_query.execute(pool).await?.rows_affected();

        if rows_affected == 0 {
            return Err(database::DatabaseError::NotFound(format!(
                "Category with id {} not found",
                self.id
            )));
        }

        tracing::info!("Updated category {} in database", self.id);

        // Read back the updated category
        let updated = sqlx::query_as!(
            database::Categories,
            r#"
                SELECT
                    id              AS "id!: domain::RowID",
                    code,
                    name,
                    description,
                    url_slug        AS "url_slug?: domain::UrlSlug",
                    category_type   AS "category_type!: domain::CategoryTypes",
                    color           AS "color?: domain::HexColor",
                    icon,
                    is_active       AS "is_active!: bool",
                    created_on      AS "created_on!: chrono::DateTime<chrono::Utc>",
                    updated_on      AS "updated_on!: chrono::DateTime<chrono::Utc>"
                FROM categories
                WHERE id = ?
            "#,
            self.id
        )
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }

    /// Updates multiple categories in the database in a single transaction.
    ///
    /// This function provides atomic bulk updates - either all categories are updated
    /// successfully, or none are updated if any operation fails. This is useful for
    /// batch operations or data synchronization.
    ///
    /// # Arguments
    ///
    /// * `categories` - A slice of categories to update
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns a vector of the updated categories in the same order as provided,
    /// or a `DatabaseError` if any update fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Any category with the given ID does not exist
    /// - Any updated category violates database constraints
    /// - Database connection fails
    /// - Transaction fails to commit
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    ///
    /// # async fn example(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// // Create some categories first
    /// let categories = vec![Category::mock(), Category::mock()];
    /// let inserted = Category::insert_many(&categories, pool).await?;
    ///
    /// // Update them
    /// let updates = inserted.into_iter()
    ///     .map(|cat| Category {
    ///         name: format!("Updated {}", cat.name),
    ///         ..cat
    ///     })
    ///     .collect::<Vec<_>>();
    ///
    /// let updated = Category::update_many(&updates, pool).await?;
    /// assert_eq!(updated.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Bulk update categories in database",
        skip(categories, pool),
        fields(count = categories.len()),
        err
    )]
    pub async fn update_many(
        categories: &[Self],
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<Vec<Self>> {
        if categories.is_empty() {
            return Ok(Vec::new());
        }

        // Use a transaction for atomicity
        let mut tx = pool.begin().await?;

        let mut updated_categories = Vec::with_capacity(categories.len());

        for category in categories {
            // Update each category
            let update_query = sqlx::query!(
                r#"
                    UPDATE categories
                    SET code = ?, name = ?, description = ?, url_slug = ?, category_type = ?,
                        color = ?, icon = ?, is_active = ?, updated_on = ?
                    WHERE id = ?
                "#,
                category.code,
                category.name,
                category.description,
                category.url_slug,
                category.category_type,
                category.color,
                category.icon,
                category.is_active,
                category.updated_on,
                category.id
            );

            let rows_affected = update_query.execute(&mut *tx).await?.rows_affected();

            if rows_affected == 0 {
                return Err(database::DatabaseError::NotFound(format!(
                    "Category with id {} not found",
                    category.id
                )));
            }

            // Read back the updated category
            let updated = sqlx::query_as!(
                database::Categories,
                r#"
                    SELECT
                        id              AS "id!: domain::RowID",
                        code,
                        name,
                        description,
                        url_slug        AS "url_slug?: domain::UrlSlug",
                        category_type   AS "category_type!: domain::CategoryTypes",
                        color           AS "color?: domain::HexColor",
                        icon,
                        is_active       AS "is_active!: bool",
                        created_on      AS "created_on!: chrono::DateTime<chrono::Utc>",
                        updated_on      AS "updated_on!: chrono::DateTime<chrono::Utc>"
                    FROM categories
                    WHERE id = ?
                "#,
                category.id
            )
            .fetch_one(&mut *tx)
            .await?;

            updated_categories.push(updated);
        }

        // Commit the transaction
        tx.commit().await?;

        tracing::info!("Successfully updated {} categories in database", updated_categories.len());

        Ok(updated_categories)
    }

    /// Updates the active status of a category.
    ///
    /// This is a convenience function for toggling category active/inactive status,
    /// which is a common operation in category management.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the category to update
    /// * `is_active` - The new active status
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns the updated category, or a `DatabaseError` if the update fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The category with the given ID does not exist
    /// - Database connection fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// # async fn example(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// // Assuming we have a category ID
    /// let category_id = RowID::new();
    ///
    /// // Deactivate the category
    /// let updated = Category::update_active_status(category_id, false, pool).await?;
    /// assert!(!updated.is_active);
    ///
    /// // Reactivate the category
    /// let updated = Category::update_active_status(category_id, true, pool).await?;
    /// assert!(updated.is_active);
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Update category active status",
        skip(pool),
        fields(id = %id, is_active = %is_active),
        err
    )]
    pub async fn update_active_status(
        id: domain::RowID,
        is_active: bool,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<Self> {
        // Update only the active status and updated_on timestamp
        let update_query = sqlx::query!(
            r#"
                UPDATE categories
                SET is_active = ?, updated_on = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                WHERE id = ?
            "#,
            is_active,
            id
        );

        let rows_affected = update_query.execute(pool).await?.rows_affected();

        if rows_affected == 0 {
            return Err(database::DatabaseError::NotFound(format!(
                "Category with id {} not found",
                id
            )));
        }

        tracing::info!("Updated active status for category {} to {}", id, is_active);

        // Read back the updated category
        let updated = sqlx::query_as!(
            database::Categories,
            r#"
                SELECT
                    id              AS "id!: domain::RowID",
                    code,
                    name,
                    description,
                    url_slug        AS "url_slug?: domain::UrlSlug",
                    category_type   AS "category_type!: domain::CategoryTypes",
                    color           AS "color?: domain::HexColor",
                    icon,
                    is_active       AS "is_active!: bool",
                    created_on      AS "created_on!: chrono::DateTime<chrono::Utc>",
                    updated_on      AS "updated_on!: chrono::DateTime<chrono::Utc>"
                FROM categories
                WHERE id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }
}

#[cfg(test)]
pub mod tests {
    // Bring module into test scope
    use super::*;

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    #[sqlx::test]
    async fn update_existing_category_success(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        // First insert a category
        let original = database::Categories::mock();
        let inserted = original.insert(&pool).await?;

        // Update the category
        let updated_name = "Updated Category Name".to_string();
        let updated_description = Some("Updated description".to_string());
        let original_code = inserted.code.clone();
        let original_created_on = inserted.created_on;

        let updated_category = database::Categories {
            name: updated_name.clone(),
            description: updated_description.clone(),
            updated_on: chrono::Utc::now(),
            ..inserted
        };

        let result = updated_category.update(&pool).await?;

        // Verify the update
        assert_eq!(result.id, inserted.id);
        assert_eq!(result.code, original_code); // Code should remain the same
        assert_eq!(result.name, updated_name);
        assert_eq!(result.description, updated_description);
        assert_eq!(result.created_on, original_created_on); // Created time should be preserved
        assert_ne!(result.updated_on, inserted.updated_on); // Updated time should change

        Ok(())
    }

    #[sqlx::test]
    async fn update_nonexistent_category_fails(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let fake_category = database::Categories::mock();

        let result = fake_category.update(&pool).await;

        assert!(result.is_err());
        // The error should be a NotFound error
        assert!(matches!(result, Err(database::DatabaseError::NotFound(_))));

        Ok(())
    }

    #[sqlx::test]
    async fn update_category_with_duplicate_name_fails(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        // Insert two categories
        let category1 = database::Categories::mock();
        let inserted1 = category1.insert(&pool).await?;

        let category2 = database::Categories::mock();
        let inserted2 = category2.insert(&pool).await?;

        // Try to update category2 with the same name as category1
        let updated_category2 = database::Categories {
            name: inserted1.name.clone(),
            updated_on: chrono::Utc::now(),
            ..inserted2
        };

        let result = updated_category2.update(&pool).await;
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn update_many_success(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        // Insert multiple categories
        let categories = vec![database::Categories::mock(), database::Categories::mock()];
        let inserted = database::Categories::insert_many(&categories, &pool).await?;
        assert_eq!(inserted.len(), 2);

        // Update them
        let updates = inserted
            .into_iter()
            .map(|cat| database::Categories {
                name: format!("Updated {}", cat.name),
                description: Some(format!("Updated description for {}", cat.name)),
                updated_on: chrono::Utc::now(),
                ..cat
            })
            .collect::<Vec<_>>();

        let updated = database::Categories::update_many(&updates, &pool).await?;

        assert_eq!(updated.len(), 2);
        for cat in updated.iter() {
            assert!(cat.name.starts_with("Updated "));
            assert!(cat.description.as_ref().unwrap().starts_with("Updated description"));
        }

        Ok(())
    }

    #[sqlx::test]
    async fn update_many_empty_list(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let categories: Vec<database::Categories> = vec![];

        let result = database::Categories::update_many(&categories, &pool).await?;

        assert!(result.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn update_many_with_nonexistent_category_fails(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        // Insert one category
        let category = database::Categories::mock();
        let inserted = category.insert(&pool).await?;

        // Create updates including a nonexistent category
        let fake_category = database::Categories::mock();
        let updates = vec![
            database::Categories {
                name: "Updated existing".to_string(),
                updated_on: chrono::Utc::now(),
                ..inserted
            },
            database::Categories {
                name: "Nonexistent".to_string(),
                updated_on: chrono::Utc::now(),
                ..fake_category
            },
        ];

        let result = database::Categories::update_many(&updates, &pool).await;
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn update_active_status_success(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        // Insert a category
        let mut category = database::Categories::mock();
        category.is_active = true;
        let inserted = category.insert(&pool).await?;
        assert!(inserted.is_active);

        // Deactivate it
        let deactivated = database::Categories::update_active_status(inserted.id, false, &pool).await?;
        assert_eq!(deactivated.id, inserted.id);
        assert!(!deactivated.is_active);
        assert_ne!(deactivated.updated_on, inserted.updated_on);

        // Reactivate it
        let reactivated = database::Categories::update_active_status(inserted.id, true, &pool).await?;
        assert_eq!(reactivated.id, inserted.id);
        assert!(reactivated.is_active);
        assert_ne!(reactivated.updated_on, deactivated.updated_on);

        Ok(())
    }

    #[sqlx::test]
    async fn update_active_status_nonexistent_category_fails(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let fake_id = domain::RowID::new();

        let result = database::Categories::update_active_status(fake_id, false, &pool).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(database::DatabaseError::NotFound(_))));

        Ok(())
    }

    #[sqlx::test]
    async fn update_preserves_created_on_timestamp(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        // Insert a category
        let category = database::Categories::mock();
        let inserted = category.insert(&pool).await?;
        let original_created_on = inserted.created_on;

        // Wait a bit and update
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let updated_category = database::Categories {
            name: "Updated Name".to_string(),
            updated_on: chrono::Utc::now(),
            ..inserted
        };

        let result = updated_category.update(&pool).await?;

        // Created timestamp should be preserved
        assert_eq!(result.created_on, original_created_on);
        // Updated timestamp should be newer
        assert!(result.updated_on > original_created_on);

        Ok(())
    }
}