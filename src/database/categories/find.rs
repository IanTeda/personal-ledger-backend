use crate::database::{self, DatabaseResult};
use crate::domain;

/// Read operations for Category database records.
///
/// This module provides functions for retrieving existing category records from the database,
/// including single record lookups, bulk retrieval, and filtered queries.
impl database::Category {
    /// Finds a category by its ID.
    ///
    /// This function retrieves a single category record from the database by its unique identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the category to find
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns `Some(Category)` if the category exists, or `None` if not found.
    /// Returns a `DatabaseError` if the query fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    /// use personal_ledger_backend::domain::RowID;
    ///
    /// # async fn example(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// let category_id = RowID::new();
    ///
    /// if let Some(category) = Category::find_by_id(category_id, pool).await? {
    ///     println!("Found category: {}", category.name);
    /// } else {
    ///     println!("Category not found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Find category by ID",
        skip(pool),
        fields(id = %id),
        err
    )]
    pub async fn find_by_id(
        id: domain::RowID,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<Option<Self>> {
        let category = sqlx::query_as!(
            database::Category,
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
        .fetch_optional(pool)
        .await?;

        Ok(category)
    }

    /// Finds a category by its code.
    ///
    /// This function retrieves a single category record from the database by its unique code.
    /// Category codes are case-sensitive and must be unique.
    ///
    /// # Arguments
    ///
    /// * `code` - The code of the category to find
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns `Some(Category)` if the category exists, or `None` if not found.
    /// Returns a `DatabaseError` if the query fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    ///
    /// # async fn example(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// if let Some(category) = Category::find_by_code("FOOD.001", pool).await? {
    ///     println!("Found category: {}", category.name);
    /// } else {
    ///     println!("Category not found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Find category by code",
        skip(pool),
        fields(code = %code),
        err
    )]
    pub async fn find_by_code(
        code: &str,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<Option<Self>> {
        let category = sqlx::query_as!(
            database::Category,
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
                WHERE code = ?
            "#,
            code
        )
        .fetch_optional(pool)
        .await?;

        Ok(category)
    }

    /// Finds a category by its URL slug.
    ///
    /// This function retrieves a single category record from the database by its URL slug.
    /// URL slugs are case-sensitive and must be unique.
    ///
    /// # Arguments
    ///
    /// * `slug` - The URL slug of the category to find
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns `Some(Category)` if the category exists, or `None` if not found.
    /// Returns a `DatabaseError` if the query fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    /// use personal_ledger_backend::domain::UrlSlug;
    ///
    /// # async fn example(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// let slug = UrlSlug::parse("groceries")?;
    /// if let Some(category) = Category::find_by_url_slug(&slug, pool).await? {
    ///     println!("Found category: {}", category.name);
    /// } else {
    ///     println!("Category not found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Find category by URL slug",
        skip(pool),
        fields(slug = %slug),
        err
    )]
    pub async fn find_by_url_slug(
        slug: &domain::UrlSlug,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<Option<Self>> {
        let category = sqlx::query_as!(
            database::Category,
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
                WHERE url_slug = ?
            "#,
            slug
        )
        .fetch_optional(pool)
        .await?;

        Ok(category)
    }

    /// Retrieves all categories from the database.
    ///
    /// This function returns all category records ordered by creation date (newest first).
    /// Use this function when you need to display all categories or perform bulk operations.
    ///
    /// # Arguments
    ///
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns a vector of all categories, or a `DatabaseError` if the query fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    ///
    /// # async fn example(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// let all_categories = Category::find_all(pool).await?;
    /// println!("Found {} categories", all_categories.len());
    ///
    /// for category in all_categories {
    ///     println!("- {} ({})", category.name, category.code);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Find all categories",
        skip(pool),
        err
    )]
    pub async fn find_all(
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<Vec<Self>> {
        let categories = sqlx::query_as!(
            database::Category,
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
                ORDER BY created_on DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        tracing::info!("Retrieved {} categories from database", categories.len());

        Ok(categories)
    }

    /// Retrieves all active categories from the database.
    ///
    /// This function returns only categories that are marked as active (is_active = true),
    /// ordered by creation date (newest first). This is useful for displaying categories
    /// in user interfaces where inactive categories should be hidden.
    ///
    /// # Arguments
    ///
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns a vector of active categories, or a `DatabaseError` if the query fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    ///
    /// # async fn example(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// let active_categories = Category::find_all_active(pool).await?;
    /// println!("Found {} active categories", active_categories.len());
    ///
    /// for category in active_categories {
    ///     println!("- {} ({})", category.name, category.code);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Find all active categories",
        skip(pool),
        err
    )]
    pub async fn find_all_active(
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<Vec<Self>> {
        let categories = sqlx::query_as!(
            database::Category,
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
                WHERE is_active = true
                ORDER BY created_on DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        tracing::info!("Retrieved {} active categories from database", categories.len());

        Ok(categories)
    }

    /// Retrieves all categories of a specific type.
    ///
    /// This function returns categories filtered by their category type (Expense or Income),
    /// ordered by creation date (newest first). This is useful for separating expense
    /// and income categories in financial applications.
    ///
    /// # Arguments
    ///
    /// * `category_type` - The type of categories to retrieve
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns a vector of categories of the specified type, or a `DatabaseError` if the query fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// # async fn example(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// let expense_categories = Category::find_by_type(CategoryTypes::Expense, pool).await?;
    /// println!("Found {} expense categories", expense_categories.len());
    ///
    /// let income_categories = Category::find_by_type(CategoryTypes::Income, pool).await?;
    /// println!("Found {} income categories", income_categories.len());
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Find categories by type",
        skip(pool),
        fields(category_type = %category_type),
        err
    )]
    pub async fn find_by_type(
        category_type: domain::CategoryTypes,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<Vec<Self>> {
        let categories = sqlx::query_as!(
            database::Category,
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
                WHERE category_type = ?
                ORDER BY created_on DESC
            "#,
            category_type
        )
        .fetch_all(pool)
        .await?;

        tracing::info!("Retrieved {} categories of type {} from database", categories.len(), category_type);

        Ok(categories)
    }

    /// Retrieves all active categories of a specific type.
    ///
    /// This function returns active categories filtered by their category type (Expense or Income),
    /// ordered by creation date (newest first). This combines the filtering of `find_by_type`
    /// and `find_all_active` for convenience.
    ///
    /// # Arguments
    ///
    /// * `category_type` - The type of categories to retrieve
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns a vector of active categories of the specified type, or a `DatabaseError` if the query fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// # async fn example(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// let active_expense_categories = Category::find_active_by_type(CategoryTypes::Expense, pool).await?;
    /// println!("Found {} active expense categories", active_expense_categories.len());
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Find active categories by type",
        skip(pool),
        fields(category_type = %category_type),
        err
    )]
    pub async fn find_active_by_type(
        category_type: domain::CategoryTypes,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<Vec<Self>> {
        let categories = sqlx::query_as!(
            database::Category,
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
                WHERE category_type = ? AND is_active = true
                ORDER BY created_on DESC
            "#,
            category_type
        )
        .fetch_all(pool)
        .await?;

        tracing::info!("Retrieved {} active categories of type {} from database", categories.len(), category_type);

        Ok(categories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        for i in 0..count {
            let mut category = database::Category::mock();
            // Override specific fields for test scenarios
            category.code = format!("TEST.{:03}", i);
            category.name = format!("Test Category {}", i);
            category.description = Some(format!("Description for category {}", i));
            category.url_slug = Some(domain::UrlSlug::from(format!("test-category-{}", i)));
            category.category_type = if i % 2 == 0 { domain::CategoryTypes::Expense } else { domain::CategoryTypes::Income };
            category.is_active = i % 3 != 0; // Every 3rd category is inactive
            database::Category::insert(&category, pool).await.unwrap();
            categories.push(category);
        }
        categories
    }

    #[sqlx::test]
    async fn test_find_by_id_existing_category(pool: SqlitePool) {
        // Create a test category
        let category = create_test_category(&pool).await;

        // Find it by ID
        let found = database::Category::find_by_id(category.id, &pool).await.unwrap();

        // Verify it's the same category
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, category.id);
        assert_eq!(found.code, category.code);
        assert_eq!(found.name, category.name);
        assert_eq!(found.description, category.description);
        assert_eq!(found.url_slug, category.url_slug);
        assert_eq!(found.category_type, category.category_type);
        assert_eq!(found.color, category.color);
        assert_eq!(found.icon, category.icon);
        assert_eq!(found.is_active, category.is_active);
    }

    #[sqlx::test]
    async fn test_find_by_id_nonexistent_category(pool: SqlitePool) {
        // Try to find a category that doesn't exist
        let fake_id = domain::RowID::new();
        let result = database::Category::find_by_id(fake_id, &pool).await.unwrap();

        // Should return None
        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn test_find_by_code_existing_category(pool: SqlitePool) {
        // Create a test category
        let category = create_test_category(&pool).await;

        // Find it by code
        let found = database::Category::find_by_code(&category.code, &pool).await.unwrap();

        // Verify it's the same category
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, category.id);
        assert_eq!(found.code, category.code);
    }

    #[sqlx::test]
    async fn test_find_by_code_nonexistent_category(pool: SqlitePool) {
        // Try to find a category that doesn't exist
        let result = database::Category::find_by_code("NONEXISTENT.CODE", &pool).await.unwrap();

        // Should return None
        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn test_find_by_code_case_sensitive(pool: SqlitePool) {
        // Create a test category with uppercase code
        let mut category = create_test_category(&pool).await;
        category.code = category.code.to_uppercase();

        // Update it in the database
        database::Category::update(&category, &pool).await.unwrap();

        // Try to find with lowercase version - should fail
        let lowercase_code = category.code.to_lowercase();
        let result = database::Category::find_by_code(&lowercase_code, &pool).await.unwrap();
        assert!(result.is_none());

        // Find with correct case should work
        let result = database::Category::find_by_code(&category.code, &pool).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, category.id);
    }

    #[sqlx::test]
    async fn test_find_by_url_slug_existing_category(pool: SqlitePool) {
        // Create a test category
        let category = create_test_category(&pool).await;

        // Find it by URL slug (assuming it has one)
        if let Some(ref slug) = category.url_slug {
            let found = database::Category::find_by_url_slug(slug, &pool).await.unwrap();

            // Verify it's the same category
            assert!(found.is_some());
            let found = found.unwrap();
            assert_eq!(found.id, category.id);
            assert_eq!(found.url_slug, category.url_slug);
        } else {
            // If no slug, create one with a slug
            let category_with_slug = database::Category {
                url_slug: Some(domain::UrlSlug::from("test-slug")),
                ..category
            };
            database::Category::update(&category_with_slug, &pool).await.unwrap();

            let found = database::Category::find_by_url_slug(&domain::UrlSlug::from("test-slug"), &pool).await.unwrap();
            assert!(found.is_some());
            assert_eq!(found.unwrap().id, category.id);
        }
    }

    #[sqlx::test]
    async fn test_find_by_url_slug_nonexistent_category(pool: SqlitePool) {
        // Try to find a category with a slug that doesn't exist
        let fake_slug = domain::UrlSlug::from("nonexistent-slug");
        let result = database::Category::find_by_url_slug(&fake_slug, &pool).await.unwrap();

        // Should return None
        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn test_find_all_with_categories(pool: SqlitePool) {
        // Create some test categories
        let test_categories = create_test_categories(5, &pool).await;

        // Find all categories
        let all_categories = database::Category::find_all(&pool).await.unwrap();

        // Should have at least our test categories
        assert!(all_categories.len() >= test_categories.len());

        // Verify our test categories are in the results
        for test_cat in &test_categories {
            let found = all_categories.iter().find(|c| c.id == test_cat.id);
            assert!(found.is_some(), "Test category {} not found in results", test_cat.id);
        }
    }

    #[sqlx::test]
    async fn test_find_all_empty_database(pool: SqlitePool) {
        // Find all categories in empty database
        let all_categories = database::Category::find_all(&pool).await.unwrap();

        // Should return empty vector
        assert!(all_categories.is_empty());
    }

    #[sqlx::test]
    async fn test_find_all_active_with_mixed_categories(pool: SqlitePool) {
        // Create test categories (some active, some inactive)
        let test_categories = create_test_categories(9, &pool).await; // 3 inactive, 6 active

        // Find all active categories
        let active_categories = database::Category::find_all_active(&pool).await.unwrap();

        // Should have exactly the active ones
        let expected_active_count = test_categories.iter().filter(|c| c.is_active).count();
        assert_eq!(active_categories.len(), expected_active_count);

        // Verify all returned categories are active
        for category in &active_categories {
            assert!(category.is_active, "Category {} is not active", category.id);
        }

        // Verify no inactive categories are returned
        for test_cat in &test_categories {
            if !test_cat.is_active {
                let found_in_active = active_categories.iter().find(|c| c.id == test_cat.id);
                assert!(found_in_active.is_none(), "Inactive category {} found in active results", test_cat.id);
            }
        }
    }

    #[sqlx::test]
    async fn test_find_all_active_no_active_categories(pool: SqlitePool) {
        // Create only inactive categories
        let mut inactive_categories = Vec::new();
        for i in 0..3 {
            let category = database::Category {
                id: domain::RowID::new(),
                code: format!("INACTIVE.{:03}", i),
                name: format!("Inactive Category {}", i),
                description: Some(format!("Inactive description {}", i)),
                url_slug: Some(domain::UrlSlug::from(format!("inactive-category-{}", i))),
                category_type: domain::CategoryTypes::Expense,
                color: domain::HexColor::mock_with_option(),
                icon: Some("test-icon".to_string()),
                is_active: false, // Inactive
                created_on: chrono::Utc::now(),
                updated_on: chrono::Utc::now(),
            };
            database::Category::insert(&category, &pool).await.unwrap();
            inactive_categories.push(category);
        }

        // Find all active categories
        let active_categories = database::Category::find_all_active(&pool).await.unwrap();

        // Should return empty vector
        assert!(active_categories.is_empty());
    }

    #[sqlx::test]
    async fn test_find_by_type_expense_categories(pool: SqlitePool) {
        // Create test categories with mixed types
        let test_categories = create_test_categories(10, &pool).await;

        // Find expense categories
        let expense_categories = database::Category::find_by_type(domain::CategoryTypes::Expense, &pool).await.unwrap();

        // Verify all returned categories are expenses
        for category in &expense_categories {
            assert_eq!(category.category_type, domain::CategoryTypes::Expense);
        }

        // Verify count matches expected
        let expected_expense_count = test_categories.iter()
            .filter(|c| c.category_type == domain::CategoryTypes::Expense)
            .count();
        assert_eq!(expense_categories.len(), expected_expense_count);
    }

    #[sqlx::test]
    async fn test_find_by_type_income_categories(pool: SqlitePool) {
        // Create test categories with mixed types
        let test_categories = create_test_categories(10, &pool).await;

        // Find income categories
        let income_categories = database::Category::find_by_type(domain::CategoryTypes::Income, &pool).await.unwrap();

        // Verify all returned categories are income
        for category in &income_categories {
            assert_eq!(category.category_type, domain::CategoryTypes::Income);
        }

        // Verify count matches expected
        let expected_income_count = test_categories.iter()
            .filter(|c| c.category_type == domain::CategoryTypes::Income)
            .count();
        assert_eq!(income_categories.len(), expected_income_count);
    }

    #[sqlx::test]
    async fn test_find_by_type_no_categories_of_type(pool: SqlitePool) {
        // Create only expense categories
        for i in 0..3 {
            let category = database::Category {
                id: domain::RowID::new(),
                code: format!("EXPENSE.{:03}", i),
                name: format!("Expense Category {}", i),
                description: Some(format!("Expense description {}", i)),
                url_slug: Some(domain::UrlSlug::from(format!("expense-category-{}", i))),
                category_type: domain::CategoryTypes::Expense, // Only expenses
                color: domain::HexColor::mock_with_option(),
                icon: Some("test-icon".to_string()),
                is_active: true,
                created_on: chrono::Utc::now(),
                updated_on: chrono::Utc::now(),
            };
            database::Category::insert(&category, &pool).await.unwrap();
        }

        // Try to find income categories
        let income_categories = database::Category::find_by_type(domain::CategoryTypes::Income, &pool).await.unwrap();

        // Should return empty vector
        assert!(income_categories.is_empty());
    }

    #[sqlx::test]
    async fn test_find_active_by_type_mixed_categories(pool: SqlitePool) {
        // Create test categories with mixed types and activity status
        let test_categories = create_test_categories(12, &pool).await; // 4 inactive, 8 active

        // Find active expense categories
        let active_expense_categories = database::Category::find_active_by_type(domain::CategoryTypes::Expense, &pool).await.unwrap();

        // Verify all returned categories are active expenses
        for category in &active_expense_categories {
            assert!(category.is_active);
            assert_eq!(category.category_type, domain::CategoryTypes::Expense);
        }

        // Verify count matches expected
        let expected_active_expense_count = test_categories.iter()
            .filter(|c| c.is_active && c.category_type == domain::CategoryTypes::Expense)
            .count();
        assert_eq!(active_expense_categories.len(), expected_active_expense_count);

        // Find active income categories
        let active_income_categories = database::Category::find_active_by_type(domain::CategoryTypes::Income, &pool).await.unwrap();

        // Verify all returned categories are active income
        for category in &active_income_categories {
            assert!(category.is_active);
            assert_eq!(category.category_type, domain::CategoryTypes::Income);
        }

        // Verify count matches expected
        let expected_active_income_count = test_categories.iter()
            .filter(|c| c.is_active && c.category_type == domain::CategoryTypes::Income)
            .count();
        assert_eq!(active_income_categories.len(), expected_active_income_count);
    }

    #[sqlx::test]
    async fn test_find_active_by_type_no_active_categories_of_type(pool: SqlitePool) {
        // Create only inactive income categories
        for i in 0..3 {
            let category = database::Category {
                id: domain::RowID::new(),
                code: format!("INCOME.{:03}", i),
                name: format!("Income Category {}", i),
                description: Some(format!("Income description {}", i)),
                url_slug: Some(domain::UrlSlug::from(format!("income-category-{}", i))),
                category_type: domain::CategoryTypes::Income,
                color: domain::HexColor::mock_with_option(),
                icon: Some("test-icon".to_string()),
                is_active: false, // Inactive
                created_on: chrono::Utc::now(),
                updated_on: chrono::Utc::now(),
            };
            database::Category::insert(&category, &pool).await.unwrap();
        }

        // Try to find active income categories
        let active_income_categories = database::Category::find_active_by_type(domain::CategoryTypes::Income, &pool).await.unwrap();

        // Should return empty vector
        assert!(active_income_categories.is_empty());
    }
}