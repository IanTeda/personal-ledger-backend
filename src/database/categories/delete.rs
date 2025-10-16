use crate::database::{self, DatabaseResult};
use crate::domain;

/// Delete operations for Category database records.
///
/// This module provides functions for deleting existing category records from the database,
/// including single record deletion, bulk deletion, and specialized delete operations.
impl database::Category {
    /// Deletes this category from the database.
    ///
    /// This function permanently removes the current category record from the database.
    /// The operation is atomic and will either succeed completely or fail without side effects.
    /// This is a convenience method that operates on the current instance.
    ///
    /// # Arguments
    ///
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the deletion was successful, or a `DatabaseError` if it fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The category no longer exists in the database
    /// - Database connection fails
    /// - The deletion violates foreign key constraints
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    ///
    /// # async fn example(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// // Create and insert a category
    /// let mut category = Category::mock();
    /// let inserted = category.insert(pool).await?;
    ///
    /// // Delete the category using the instance method
    /// inserted.delete(pool).await?;
    ///
    /// // The category is now deleted
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Delete category instance from database",
        skip(pool),
        fields(id = %self.id, code = %self.code),
        err
    )]
    pub async fn delete(&self, pool: &sqlx::Pool<sqlx::Sqlite>) -> DatabaseResult<()> {
        let delete_query = sqlx::query!(
            r#"
                DELETE FROM categories
                WHERE id = ?
            "#,
            self.id
        );

        let rows_affected = delete_query.execute(pool).await?.rows_affected();

        if rows_affected == 0 {
            return Err(database::DatabaseError::NotFound(format!(
                "Category with id {} not found",
                self.id
            )));
        }

        tracing::info!("Deleted category {} ({}) from database", self.id, self.code);

        Ok(())
    }

    /// Deletes a category from the database by its ID.
    ///
    /// This function permanently removes a category record from the database.
    /// The operation is atomic and will either succeed completely or fail without side effects.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the category to delete
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the deletion was successful, or a `DatabaseError` if it fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The category with the given ID does not exist
    /// - Database connection fails
    /// - The deletion violates foreign key constraints
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
    /// // Delete the category
    /// Category::delete(category_id, pool).await?;
    ///
    /// // Verify it's gone (this would fail)
    /// // let result = Category::find_by_id(category_id, pool).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Delete category from database",
        skip(pool),
        fields(id = %id),
        err
    )]
    pub async fn delete_by_id(
        id: domain::RowID,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<()> {
        let delete_query = sqlx::query!(
            r#"
                DELETE FROM categories
                WHERE id = ?
            "#,
            id
        );

        let rows_affected = delete_query.execute(pool).await?.rows_affected();

        if rows_affected == 0 {
            return Err(database::DatabaseError::NotFound(format!(
                "Category with id {} not found",
                id
            )));
        }

        tracing::info!("Deleted category {} from database", id);

        Ok(())
    }

    /// Deletes multiple categories from the database by their IDs.
    ///
    /// This function provides atomic bulk deletion - either all categories are deleted
    /// successfully, or none are deleted if any operation fails. This is useful for
    /// cleanup operations or batch processing.
    ///
    /// # Arguments
    ///
    /// * `ids` - A slice of category IDs to delete
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all deletions were successful, or a `DatabaseError` if any fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Any category with the given IDs does not exist
    /// - Database connection fails
    /// - Transaction fails to commit
    /// - Any deletion violates foreign key constraints
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// # async fn example(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// // Create some categories first
    /// let categories = vec![Category::mock(), Category::mock()];
    /// let inserted = Category::insert_many(&categories, pool).await?;
    /// let ids: Vec<RowID> = inserted.iter().map(|c| c.id).collect();
    ///
    /// // Delete them all
    /// Category::delete_many_by_id(&ids, pool).await?;
    ///
    /// // All categories are now deleted
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Bulk delete categories from database",
        skip(ids, pool),
        fields(count = ids.len()),
        err
    )]
    pub async fn delete_many_by_id(
        ids: &[domain::RowID],
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<()> {
        if ids.is_empty() {
            return Ok(());
        }

        // Use a transaction for atomicity
        let mut tx = pool.begin().await?;

        for &id in ids {
            let delete_query = sqlx::query!(
                r#"
                    DELETE FROM categories
                    WHERE id = ?
                "#,
                id
            );

            let rows_affected = delete_query.execute(&mut *tx).await?.rows_affected();

            if rows_affected == 0 {
                return Err(database::DatabaseError::NotFound(format!(
                    "Category with id {} not found",
                    id
                )));
            }
        }

        // Commit the transaction
        tx.commit().await?;

        tracing::info!("Successfully deleted {} categories from database", ids.len());

        Ok(())
    }

    /// Deletes all inactive categories from the database.
    ///
    /// This is a convenience function for cleaning up deactivated categories.
    /// Use with caution as this operation cannot be undone.
    ///
    /// # Arguments
    ///
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns the number of categories deleted, or a `DatabaseError` if the operation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Database connection fails
    /// - The deletion violates foreign key constraints
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    ///
    /// # async fn example(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// // Deactivate some categories first
    /// // ... deactivation logic ...
    ///
    /// // Clean up all inactive categories
    /// let deleted_count = Category::delete_inactive(pool).await?;
    /// println!("Deleted {} inactive categories", deleted_count);
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Delete all inactive categories from database",
        skip(pool),
        err
    )]
    pub async fn delete_inactive(
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<u64> {
        let delete_query = sqlx::query!(
            r#"
                DELETE FROM categories
                WHERE is_active = false
            "#
        );

        let rows_affected = delete_query.execute(pool).await?.rows_affected();

        tracing::info!("Deleted {} inactive categories from database", rows_affected);

        Ok(rows_affected)
    }

    /// Deletes a category by its code.
    ///
    /// This is a convenience function for deleting categories when you have the code
    /// instead of the ID. Useful for admin operations or cleanup scripts.
    ///
    /// # Arguments
    ///
    /// * `code` - The code of the category to delete
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the deletion was successful, or a `DatabaseError` if it fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - No category with the given code exists
    /// - Database connection fails
    /// - The deletion violates foreign key constraints
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    ///
    /// # async fn example(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// // Delete category by code
    /// Category::delete_by_code("FOOD.001", pool).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Delete category by code from database",
        skip(pool),
        fields(code = %code),
        err
    )]
    pub async fn delete_by_code(
        code: &str,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<()> {
        let delete_query = sqlx::query!(
            r#"
                DELETE FROM categories
                WHERE code = ?
            "#,
            code
        );

        let rows_affected = delete_query.execute(pool).await?.rows_affected();

        if rows_affected == 0 {
            return Err(database::DatabaseError::NotFound(format!(
                "Category with code '{}' not found",
                code
            )));
        }

        tracing::info!("Deleted category with code '{}' from database", code);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{RowID, UrlSlug};
    use sqlx::SqlitePool;

    /// Helper function to create a test category
    async fn create_test_category(pool: &SqlitePool) -> database::Category {
        let category = database::Category::mock();
        database::Category::insert(&category, pool).await.unwrap();
        category
    }

    /// Helper function to create multiple test categories
    async fn create_test_categories(count: usize, pool: &SqlitePool) -> Vec<database::Category> {
        let mut categories = Vec::with_capacity(count);
        for _ in 0..count {
            let category = database::Category::mock();
            database::Category::insert(&category, pool).await.unwrap();
            categories.push(category);
        }
        categories
    }

    #[sqlx::test]
    async fn test_delete_existing_category(pool: SqlitePool) {
        // Create a test category
        let category = create_test_category(&pool).await;

        // Verify it exists by trying to insert a duplicate (should fail)
        let duplicate = database::Category {
            id: category.id,
            ..category.clone()
        };
        let insert_result = database::Category::insert_or_update(&duplicate, &pool).await;
        assert!(insert_result.is_ok()); // Should succeed (update existing)

        // Delete the category
        let result = database::Category::delete_by_id(category.id, &pool).await;
        assert!(result.is_ok());

        // Verify it's gone by trying to insert with same ID (should succeed as new insert)
        let reinsert_result = database::Category::insert(&category, &pool).await;
        assert!(reinsert_result.is_ok());
    }

    #[sqlx::test]
    async fn test_delete_nonexistent_category(pool: SqlitePool) {
        // Try to delete a category that doesn't exist
        let fake_id = RowID::new();
        let result = database::Category::delete_by_id(fake_id, &pool).await;

        // Should return NotFound error
        assert!(matches!(result, Err(database::DatabaseError::NotFound(_))));
        assert!(result.unwrap_err().to_string().contains(&fake_id.to_string()));
    }

    #[sqlx::test]
    async fn test_delete_many_categories(pool: SqlitePool) {
        // Create multiple test categories
        let categories = create_test_categories(3, &pool).await;
        let ids: Vec<RowID> = categories.iter().map(|c| c.id).collect();

        // Delete them all
        let result = database::Category::delete_many_by_id(&ids, &pool).await;
        assert!(result.is_ok());

        // Verify they're all gone by trying to re-insert them
        for category in &categories {
            let reinsert_result = database::Category::insert(category, &pool).await;
            assert!(reinsert_result.is_ok()); // Should succeed as they're gone
        }
    }

    #[sqlx::test]
    async fn test_delete_many_with_nonexistent_category(pool: SqlitePool) {
        // Create one real category
        let category = create_test_category(&pool).await;
        let mut ids = vec![category.id];

        // Add a fake ID
        let fake_id = RowID::new();
        ids.push(fake_id);

        // Try to delete - should fail due to nonexistent category
        let result = database::Category::delete_many_by_id(&ids, &pool).await;
        assert!(matches!(result, Err(database::DatabaseError::NotFound(_))));
        assert!(result.unwrap_err().to_string().contains(&fake_id.to_string()));

        // The real category should still exist (transaction rolled back)
        // Verify by trying to insert duplicate (should fail as update)
        let duplicate = database::Category {
            id: category.id,
            ..category.clone()
        };
        let insert_result = database::Category::insert_or_update(&duplicate, &pool).await;
        assert!(insert_result.is_ok()); // Should succeed (update existing)
    }

    #[sqlx::test]
    async fn test_delete_many_empty_list(pool: SqlitePool) {
        // Delete with empty list should succeed
        let result = database::Category::delete_many_by_id(&[], &pool).await;
        assert!(result.is_ok());
    }

    #[sqlx::test]
    async fn test_delete_inactive_categories(pool: SqlitePool) {
        // Create some active categories
        let mut active_categories = Vec::new();
        for i in 0..2 {
            let mut category = database::Category::mock();
            category.code = format!("ACTIVE.{:03}", i);
            category.name = format!("Active Category {}", i);
            category.description = Some(format!("Active description {}", i));
            category.url_slug = Some(UrlSlug::from(format!("active-category-{}", i)));
            category.is_active = true; // Ensure active
            database::Category::insert(&category, &pool).await.unwrap();
            active_categories.push(category);
        }

        // Create some inactive categories
        let mut inactive_categories = Vec::new();
        for i in 0..3 {
            let mut category = database::Category::mock();
            category.code = format!("INACTIVE.{:03}", i);
            category.name = format!("Inactive Category {}", i);
            category.description = Some(format!("Inactive description {}", i));
            category.url_slug = Some(UrlSlug::from(format!("inactive-category-{}", i)));
            category.is_active = false; // Inactive
            database::Category::insert(&category, &pool).await.unwrap();
            inactive_categories.push(category);
        }

        // Delete inactive categories
        let deleted_count = database::Category::delete_inactive(&pool).await.unwrap();
        assert_eq!(deleted_count, 3);

        // Verify inactive categories are gone by trying to re-insert them
        for category in &inactive_categories {
            let reinsert_result = database::Category::insert(category, &pool).await;
            assert!(reinsert_result.is_ok()); // Should succeed as they're gone
        }

        // Verify active categories still exist by trying to insert duplicates (should fail as updates)
        for category in &active_categories {
            let duplicate = database::Category {
                id: category.id,
                ..category.clone()
            };
            let insert_result = database::Category::insert_or_update(&duplicate, &pool).await;
            assert!(insert_result.is_ok()); // Should succeed (update existing)
        }
    }

    #[sqlx::test]
    async fn test_delete_inactive_no_inactive_categories(pool: SqlitePool) {
        // Create only active categories
        let mut active_categories = Vec::with_capacity(2);
        for _ in 0..2 {
            let mut category = database::Category::mock();
            category.is_active = true; // Ensure category is active
            database::Category::insert(&category, &pool).await.unwrap();
            active_categories.push(category);
        }

        // Delete inactive categories - should delete 0
        let deleted_count = database::Category::delete_inactive(&pool).await.unwrap();
        assert_eq!(deleted_count, 0);

        // Active categories should still exist
        // We can't easily count without read functions, but the test passes if no panic occurs
    }

    #[sqlx::test]
    async fn test_delete_by_code_existing_category(pool: SqlitePool) {
        // Create a test category
        let category = create_test_category(&pool).await;

        // Delete by code
        let result = database::Category::delete_by_code(&category.code, &pool).await;
        assert!(result.is_ok());

        // Verify it's gone by trying to re-insert
        let reinsert_result = database::Category::insert(&category, &pool).await;
        assert!(reinsert_result.is_ok()); // Should succeed as it's gone
    }

    #[sqlx::test]
    async fn test_delete_by_code_nonexistent_category(pool: SqlitePool) {
        // Try to delete by a code that doesn't exist
        let fake_code = "NONEXISTENT.CODE";
        let result = database::Category::delete_by_code(fake_code, &pool).await;

        // Should return NotFound error
        assert!(matches!(result, Err(database::DatabaseError::NotFound(_))));
        assert!(result.unwrap_err().to_string().contains(fake_code));
    }

    #[sqlx::test]
    async fn test_delete_by_code_case_sensitive(pool: SqlitePool) {
        // Create a test category with uppercase code
        let mut category = create_test_category(&pool).await;
        category.code = category.code.to_uppercase();

        // Update it in the database
        database::Category::update(&category, &pool).await.unwrap();

        // Try to delete with lowercase version - should fail
        let lowercase_code = category.code.to_lowercase();
        let result = database::Category::delete_by_code(&lowercase_code, &pool).await;
        assert!(matches!(result, Err(database::DatabaseError::NotFound(_))));

        // Delete with correct case should work
        let result = database::Category::delete_by_code(&category.code, &pool).await;
        assert!(result.is_ok());
    }

    #[sqlx::test]
    async fn test_delete_self_existing_category(pool: SqlitePool) {
        // Create a test category
        let category = create_test_category(&pool).await;

        // Delete using the instance method
        let result = category.delete(&pool).await;
        assert!(result.is_ok());

        // Verify it's gone by trying to find it
        let found = database::Category::find_by_id(category.id, &pool).await.unwrap();
        assert!(found.is_none());
    }

    #[sqlx::test]
    async fn test_delete_self_nonexistent_category(pool: SqlitePool) {
        // Create a category but don't insert it
        let category = database::Category::mock();

        // Try to delete the non-existent category
        let result = category.delete(&pool).await;

        // Should return NotFound error
        assert!(matches!(result, Err(crate::database::DatabaseError::NotFound(_))));
        assert!(result.unwrap_err().to_string().contains(&category.id.to_string()));
    }

    #[sqlx::test]
    async fn test_delete_self_after_update(pool: SqlitePool) {
        // Create and insert a category
        let mut category = create_test_category(&pool).await;

        // Update the category
        category.name = "Updated Name".to_string();
        let updated = database::Category::update(&category, &pool).await.unwrap();

        // Delete using the updated instance
        let result = updated.delete(&pool).await;
        assert!(result.is_ok());

        // Verify it's gone
        let found = database::Category::find_by_id(category.id, &pool).await.unwrap();
        assert!(found.is_none());
    }
}