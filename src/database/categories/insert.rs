use crate::database::{Category, DatabaseResult};

impl Category {
    /// Insert a new category into the database.
    ///
    /// This method inserts the current category instance into the database and returns
    /// the inserted record with any database-generated values. The category's UUID, timestamps,
    /// and all other fields are preserved.
    ///
    /// # Arguments
    ///
    /// * `database` - The SQLx connection pool for executing the insert query
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the inserted `Category` on success, or a `DatabaseError` on failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The database connection fails
    /// - A unique constraint is violated (duplicate code or slug)
    /// - The insert operation fails for any other reason
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use personal_ledger_backend::database::Category;
    /// # use personal_ledger_backend::domain::{RowID, CategoryTypes};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let pool = sqlx::Pool::<sqlx::Any>::connect("sqlite::memory:").await?;
    /// let category = Category {
    ///     id: RowID::new(),
    ///     code: "FOOD-001".to_string(),
    ///     name: "Groceries".to_string(),
    ///     description: Some("Food and grocery expenses".to_string()),
    ///     slug: None,
    ///     category_type: CategoryTypes::Expense,
    ///     color: Some("#4CAF50".to_string()),
    ///     icon: Some("shopping-cart".to_string()),
    ///     is_active: true,
    ///     created_on: chrono::Utc::now(),
    ///     updated_on: chrono::Utc::now(),
    /// };
    ///
    /// let inserted = category.insert(pool).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn insert(&self, database: sqlx::Pool<sqlx::Any>) -> DatabaseResult<Self> {
        let database_record = sqlx::query_as::<_, Category>(
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
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                RETURNING id, code, name, description, slug, category_type, color, icon, is_active, created_on, updated_on
            "#,
        )
        .bind(self.id)
        .bind(&self.code)
        .bind(&self.name)
        .bind(&self.description)
        .bind(self.slug.as_ref().map(|s| s.as_str()))
        .bind(self.category_type.as_str())
        .bind(&self.color)
        .bind(&self.icon)
        .bind(self.is_active)
        .bind(self.created_on.to_rfc3339())
        .bind(self.updated_on.to_rfc3339())
        .fetch_one(&database)
        .await?;

        Ok(database_record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CategoryTypes, RowID, UrlSlug};

    /// Helper to convert test migrations from Sqlite pool to Any pool
    async fn get_test_db() -> sqlx::Pool<sqlx::Any> {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let db_url = format!("sqlite:file:memdb{}?mode=memory&cache=shared", id);
        
        sqlx::Pool::<sqlx::Any>::connect(&db_url)
            .await
            .expect("Failed to create test database")
    }

    async fn setup_test_db(pool: &sqlx::Pool<sqlx::Any>) {
        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .expect("Failed to run migrations");
    }

    #[tokio::test]
    async fn test_category_insert_basic() {
        let pool = get_test_db().await;
        setup_test_db(&pool).await;
        // Create a basic category
        let category = Category::builder()
            .code("TEST-001")
            .name("Test Category")
            .category_type(CategoryTypes::Expense)
            .build();

        // Insert the category
        let inserted = category.insert(pool).await.unwrap();

        // Verify the inserted category
        assert_eq!(inserted.id, category.id);
        assert_eq!(inserted.code, "TEST-001");
        assert_eq!(inserted.name, "Test Category");
        assert_eq!(inserted.category_type, CategoryTypes::Expense);
        assert!(inserted.is_active);
        assert_eq!(inserted.created_on, category.created_on);
        assert_eq!(inserted.updated_on, category.updated_on);
    }

    #[tokio::test]
    async fn test_category_insert_with_all_fields() {
        let pool = get_test_db().await;
        setup_test_db(&pool).await;
        // Create a category with all optional fields
        let slug = UrlSlug::parse("test-category-full").unwrap();
        let category = Category::builder()
            .code("FULL-001")
            .name("Full Test Category")
            .description("A comprehensive test category")
            .slug(slug.clone())
            .category_type(CategoryTypes::Asset)
            .color("#FF5733")
            .icon("dollar-sign")
            .with_active(false)
            .build();

        // Insert the category
        let inserted = category.insert(pool).await.unwrap();

        // Verify all fields
        assert_eq!(inserted.code, "FULL-001");
        assert_eq!(inserted.name, "Full Test Category");
        assert_eq!(inserted.description, Some("A comprehensive test category".to_string()));
        assert_eq!(inserted.slug, Some(slug));
        assert_eq!(inserted.category_type, CategoryTypes::Asset);
        assert_eq!(inserted.color, Some("#FF5733".to_string()));
        assert_eq!(inserted.icon, Some("dollar-sign".to_string()));
        assert!(!inserted.is_active);
    }

    #[tokio::test]
    async fn test_category_insert_minimal_fields() {
        let pool = get_test_db().await;
        setup_test_db(&pool).await;
        // Create a category with only required fields
        let category = Category::builder()
            .code("MIN-001")
            .name("Minimal Category")
            .category_type(CategoryTypes::Income)
            .build();

        // Insert the category
        let inserted = category.insert(pool).await.unwrap();

        // Verify required fields and defaults
        assert_eq!(inserted.code, "MIN-001");
        assert_eq!(inserted.name, "Minimal Category");
        assert_eq!(inserted.category_type, CategoryTypes::Income);
        assert!(inserted.is_active); // Default value
        assert!(inserted.description.is_none());
        assert!(inserted.slug.is_none());
        assert!(inserted.color.is_none());
        assert!(inserted.icon.is_none());
    }

    #[tokio::test]
    async fn test_category_insert_unique_code_constraint() {
        let pool = get_test_db().await;
        setup_test_db(&pool).await;
        // Insert first category
        let category1 = Category::builder()
            .code("DUPE-001")
            .name("First Category")
            .category_type(CategoryTypes::Expense)
            .build();
        category1.insert(pool.clone()).await.unwrap();

        // Try to insert second category with same code - should fail
        let category2 = Category::builder()
            .code("DUPE-001") // Same code
            .name("Second Category")
            .category_type(CategoryTypes::Asset)
            .build();

        let result = category2.insert(pool).await;
        assert!(result.is_err(), "Expected unique constraint violation");
    }

    #[tokio::test]
    async fn test_category_insert_preserves_timestamps() {
        let pool = get_test_db().await;
        setup_test_db(&pool).await;
        let now = chrono::Utc::now();
        let category = Category {
            id: RowID::new(),
            code: "TIME-001".to_string(),
            name: "Timestamp Test".to_string(),
            description: None,
            slug: None,
            category_type: CategoryTypes::Liability,
            color: None,
            icon: None,
            is_active: true,
            created_on: now,
            updated_on: now,
        };

        let inserted = category.insert(pool).await.unwrap();

        // Timestamps should be preserved
        assert_eq!(inserted.created_on, now);
        assert_eq!(inserted.updated_on, now);
    }
}
