use crate::database::{self, DatabaseResult};
use crate::domain;


impl database::Category {
    /// Inserts a new category into the database.
    ///
    /// This function performs a single category insertion, reading back the inserted
    /// record to ensure data consistency and return any database-generated values.
    /// The operation is atomic and will either succeed completely or fail without
    /// side effects.
    ///
    /// # Arguments
    ///
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns the inserted category as read back from the database, or a
    /// `DatabaseError` if the insertion fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The category violates database constraints (duplicate code, name, or url_slug)
    /// - The category_type is invalid (not in the allowed enum values)
    /// - The color format is invalid (checked by database constraint)
    /// - Database connection fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a database connection
    /// let db = DatabasePool::new("sqlite::memory:")
    ///     .connect()
    ///     .await?;
    /// let pool = db.get_pool()?;
    ///
    /// // Create a new category
    /// let category = Category {
    ///     id: personal_ledger_backend::domain::RowID::new(),
    ///     code: "FOOD.001".to_string(),
    ///     name: "Groceries".to_string(),
    ///     description: Some("Food and beverage expenses".to_string()),
    ///     url_slug: None, // Will be auto-generated if not provided
    ///     category_type: CategoryTypes::Expense,
    ///     color: Some("#FF5733".parse()?),
    ///     icon: Some("shopping-cart".to_string()),
    ///     is_active: true,
    ///     created_on: chrono::Utc::now(),
    ///     updated_on: chrono::Utc::now(),
    /// };
    ///
    /// // Insert into database
    /// let inserted = category.insert(pool).await?;
    ///
    /// // The returned category includes any database-generated values
    /// assert_eq!(inserted.code, "FOOD.001");
    /// assert_eq!(inserted.name, "Groceries");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Using the builder pattern for cleaner construction:
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::{Category, CategoriesBuilder};
    /// use personal_ledger_backend::database::DatabasePool;
    /// use personal_ledger_backend::domain::CategoryTypes;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a database connection
    /// let db = DatabasePool::new("sqlite::memory:")
    ///     .connect()
    ///     .await?;
    /// let pool = db.get_pool()?;
    ///
    /// let category = CategoriesBuilder::new()
    ///     .with_name("Transportation")
    ///     .with_category_type(CategoryTypes::Expense)
    ///     .with_description("Travel and commuting costs")
    ///     .with_icon("car")
    ///     .build()?;
    ///
    /// let inserted = category.insert(pool).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Insert new Category into database: ",
        skip(self, pool),
        fields(
            id = % self.id,
            code = % self.code,
            name = % self.name,
            description = ? self.description,
            url_slug = ? self.url_slug,
            category_type = % self.category_type,
            color = ? self.color,
            icon = ? self.icon,
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
                INSERT INTO categories (id, code, name, description, url_slug, category_type, color, icon, is_active, created_on, updated_on)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            self.id,
            self.code,
            self.name,
            self.description,
            self.url_slug,
            self.category_type,
            self.color,
            self.icon,
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

        tracing::debug!("Newly created Category retrived from the database.");

        Ok(category)
    }

    /// Inserts multiple categories into the database in a single transaction.
    ///
    /// This function provides atomic bulk insertion - either all categories are inserted
    /// successfully, or none are inserted if any operation fails. This is useful for
    /// seeding data or importing multiple categories at once.
    ///
    /// # Arguments
    ///
    /// * `categories` - A slice of categories to insert
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns a vector of the inserted categories in the same order as provided,
    /// or a `DatabaseError` if any insertion fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Any category violates database constraints (duplicate code/name/url_slug)
    /// - Database connection fails
    /// - Transaction fails to commit
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a database connection
    /// let db = DatabasePool::new("sqlite::memory:")
    ///     .connect()
    ///     .await?;
    /// let pool = db.get_pool()?;
    ///
    /// let categories = vec![
    ///     Category::mock(),
    ///     Category::mock(),
    /// ];
    ///
    /// let inserted = Category::insert_many(&categories, pool).await?;
    /// assert_eq!(inserted.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Bulk insert categories into database",
        skip(categories, pool),
        fields(count = categories.len())
    )]
    pub async fn insert_many(
        categories: &[Self],
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<Vec<Self>> {
        if categories.is_empty() {
            return Ok(Vec::new());
        }

        // Use a transaction for atomicity
        let mut tx = pool.begin().await?;

        let mut inserted_categories = Vec::with_capacity(categories.len());

        for category in categories {
            // Insert each category
            let insert_query = sqlx::query!(
                r#"
                    INSERT INTO categories (id, code, name, description, url_slug, category_type, color, icon, is_active, created_on, updated_on)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                category.id,
                category.code,
                category.name,
                category.description,
                category.url_slug,
                category.category_type,
                category.color,
                category.icon,
                category.is_active,
                category.created_on,
                category.updated_on
            );

            insert_query.execute(&mut *tx).await?;

            // Read back the inserted category
            let inserted = sqlx::query_as!(
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
                category.id
            )
            .fetch_one(&mut *tx)
            .await?;

            inserted_categories.push(inserted);
        }

        // Commit the transaction
        tx.commit().await?;

        tracing::info!("Successfully inserted {} categories into database", inserted_categories.len());

        Ok(inserted_categories)
    }

    /// Inserts a category or updates it if it already exists (upsert).
    ///
    /// This function attempts to insert a new category. If a category with the same
    /// `id` already exists, it updates the existing record instead. This is useful
    /// for idempotent operations or data synchronization.
    ///
    /// Note: This function updates all fields except `id` and `created_on` when
    /// performing an update.
    ///
    /// # Arguments
    ///
    /// * `category` - The category to insert or update
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    ///
    /// Returns the inserted or updated category.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Database constraints are violated (e.g., duplicate code/name on different records)
    /// - Database connection fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use personal_ledger_backend::database::categories::Category;
    /// use personal_ledger_backend::database::DatabasePool;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a database connection
    /// let db = DatabasePool::new("sqlite::memory:")
    ///     .connect()
    ///     .await?;
    /// let pool = db.get_pool()?;
    ///
    /// let category = Category::mock();
    ///
    /// // First call inserts
    /// let result1 = Category::insert_or_update(&category, pool).await?;
    ///
    /// // Second call with same ID updates
    /// let result2 = Category::insert_or_update(&category, pool).await?;
    ///
    /// assert_eq!(result1.id, result2.id);
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(
        name = "Insert or update category in database",
        skip(category, pool),
        fields(id = %category.id, code = %category.code)
    )]
    pub async fn insert_or_update(
        category: &Self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> DatabaseResult<Self> {
        // Use SQLite's UPSERT syntax (INSERT ... ON CONFLICT)
        let upsert_query = sqlx::query!(
            r#"
                INSERT INTO categories (id, code, name, description, url_slug, category_type, color, icon, is_active, created_on, updated_on)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    code = excluded.code,
                    name = excluded.name,
                    description = excluded.description,
                    url_slug = excluded.url_slug,
                    category_type = excluded.category_type,
                    color = excluded.color,
                    icon = excluded.icon,
                    is_active = excluded.is_active,
                    updated_on = excluded.updated_on
                WHERE id = excluded.id
            "#,
            category.id,
            category.code,
            category.name,
            category.description,
            category.url_slug,
            category.category_type,
            category.color,
            category.icon,
            category.is_active,
            category.created_on,
            category.updated_on
        );

        upsert_query.execute(pool).await?;

        // Read back the inserted/updated category
        let result = sqlx::query_as!(
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
            category.id
        )
        .fetch_one(pool)
        .await?;

        tracing::info!("Category upserted successfully: {}", result.id);

        Ok(result)
    }
}

#[cfg(test)]
pub mod tests {
    // Bring module into test scope
    use super::*;

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    // Helper functions for generating fake test data
    #[cfg(test)]
    fn generate_fake_code() -> String {
        use fake::Fake;
        use fake::faker::lorem::en::Word;

        // Generate a unique code using fake words
        let word1: String = Word().fake();
        let word2: String = Word().fake();
        format!("{}.{}", word1.to_uppercase(), word2.to_uppercase())
    }

    #[cfg(test)]
    fn generate_fake_name() -> String {
        use fake::Fake;
        use fake::faker::lorem::en::Words;

        let words: Vec<String> = Words(1..4).fake();
        words.join(" ")
    }

    #[cfg(test)]
    fn generate_fake_description() -> Option<String> {
        use fake::Fake;
        use fake::faker::boolean::en::Boolean;
        use fake::faker::lorem::en::Sentence;

        let has_description: bool = Boolean(70).fake(); // 70% chance of having description
        if has_description {
            Some(Sentence(3..8).fake())
        } else {
            None
        }
    }

    #[cfg(test)]
    fn generate_fake_icon() -> Option<String> {
        use fake::Fake;
        use fake::faker::boolean::en::Boolean;
        use fake::faker::lorem::en::Word;

        let has_icon: bool = Boolean(60).fake(); // 60% chance of having icon
        if has_icon {
            Some(Word().fake())
        } else {
            None
        }
    }

    #[cfg(test)]
    fn generate_fake_category() -> database::Category {
        use fake::Fake;
        use fake::faker::boolean::en::Boolean;

        let category_type = match Boolean(50).fake() {
            true => domain::CategoryTypes::Expense,
            false => domain::CategoryTypes::Income,
        };

        database::Category {
            id: domain::RowID::new(),
            code: generate_fake_code(),
            name: generate_fake_name(),
            description: generate_fake_description(),
            url_slug: Some(domain::UrlSlug::from(generate_fake_name())),
            category_type,
            color: domain::HexColor::mock_with_option(),
            icon: generate_fake_icon(),
            is_active: Boolean(85).fake(), // 85% chance of active
            created_on: chrono::Utc::now(),
            updated_on: chrono::Utc::now(),
        }
    }

    #[sqlx::test]
    async fn insert_single_category_success(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let category = database::Category::mock();

        let inserted = category.insert(&pool).await?;

        assert_eq!(category.id, inserted.id);
        assert_eq!(category.code, inserted.code);
        assert_eq!(category.name, inserted.name);
        assert_eq!(category.description, inserted.description);
        assert_eq!(category.url_slug, inserted.url_slug);
        assert_eq!(category.category_type, inserted.category_type);
        assert_eq!(category.color, inserted.color);
        assert_eq!(category.icon, inserted.icon);
        assert_eq!(category.is_active, inserted.is_active);

        Ok(())
    }

    #[sqlx::test]
    async fn insert_category_with_minimal_fields(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let fake_code = generate_fake_code();
        let fake_name = generate_fake_name();

        let category = database::Category {
            id: domain::RowID::new(),
            code: fake_code.clone(),
            name: fake_name.clone(),
            description: None,
            url_slug: None,
            category_type: domain::CategoryTypes::Expense,
            color: None,
            icon: None,
            is_active: true,
            created_on: chrono::Utc::now(),
            updated_on: chrono::Utc::now(),
        };

        let inserted = category.insert(&pool).await?;

        assert_eq!(inserted.code, fake_code);
        assert_eq!(inserted.name, fake_name);
        assert!(inserted.description.is_none());
        assert!(inserted.url_slug.is_none());
        assert!(inserted.color.is_none());
        assert!(inserted.icon.is_none());
        assert!(inserted.is_active);

        Ok(())
    }

    #[sqlx::test]
    async fn insert_category_with_all_fields(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let fake_code = generate_fake_code();
        let fake_name = generate_fake_name();
        let fake_description = generate_fake_description().unwrap_or_else(|| "Test description".to_string());
        let fake_icon = generate_fake_icon().unwrap_or_else(|| "test-icon".to_string());

        let color = domain::HexColor::parse("#FF5733")?;
        let slug = domain::UrlSlug::parse("test-category")?;

        let category = database::Category {
            id: domain::RowID::new(),
            code: fake_code.clone(),
            name: fake_name.clone(),
            description: Some(fake_description.clone()),
            url_slug: Some(slug.clone()),
            category_type: domain::CategoryTypes::Income,
            color: Some(color.clone()),
            icon: Some(fake_icon.clone()),
            is_active: false,
            created_on: chrono::Utc::now(),
            updated_on: chrono::Utc::now(),
        };

        let inserted = category.insert(&pool).await?;

        assert_eq!(inserted.description.as_deref(), Some(fake_description.as_str()));
        assert_eq!(inserted.url_slug.as_ref(), Some(&slug));
        assert_eq!(inserted.color.as_ref(), Some(&color));
        assert_eq!(inserted.icon.as_deref(), Some(fake_icon.as_str()));
        assert!(!inserted.is_active);

        Ok(())
    }

    #[sqlx::test]
    async fn insert_fails_on_duplicate_code(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let duplicate_code = generate_fake_code();
        let fake_name1 = generate_fake_name();
        let fake_name2 = generate_fake_name();

        let category1 = database::Category {
            id: domain::RowID::new(),
            code: duplicate_code.clone(),
            name: fake_name1,
            description: None,
            url_slug: None,
            category_type: domain::CategoryTypes::Expense,
            color: None,
            icon: None,
            is_active: true,
            created_on: chrono::Utc::now(),
            updated_on: chrono::Utc::now(),
        };

        let category2 = database::Category {
            id: domain::RowID::new(),
            code: duplicate_code, // Same code
            name: fake_name2,
            description: None,
            url_slug: None,
            category_type: domain::CategoryTypes::Expense,
            color: None,
            icon: None,
            is_active: true,
            created_on: chrono::Utc::now(),
            updated_on: chrono::Utc::now(),
        };

        // First insert should succeed
        category1.insert(&pool).await?;

        // Second insert should fail due to duplicate code
        let result = category2.insert(&pool).await;
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn insert_fails_on_duplicate_name(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let duplicate_name = generate_fake_name();
        let fake_code1 = generate_fake_code();
        let fake_code2 = generate_fake_code();

        let category1 = database::Category {
            id: domain::RowID::new(),
            code: fake_code1,
            name: duplicate_name.clone(),
            description: None,
            url_slug: None,
            category_type: domain::CategoryTypes::Expense,
            color: None,
            icon: None,
            is_active: true,
            created_on: chrono::Utc::now(),
            updated_on: chrono::Utc::now(),
        };

        let category2 = database::Category {
            id: domain::RowID::new(),
            code: fake_code2,
            name: duplicate_name, // Same name
            description: None,
            url_slug: None,
            category_type: domain::CategoryTypes::Expense,
            color: None,
            icon: None,
            is_active: true,
            created_on: chrono::Utc::now(),
            updated_on: chrono::Utc::now(),
        };

        // First insert should succeed
        category1.insert(&pool).await?;

        // Second insert should fail due to duplicate name
        let result = category2.insert(&pool).await;
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn insert_fails_on_invalid_color_format(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        // Test that invalid colors are rejected at the domain level
        let invalid_color_result = domain::HexColor::parse("#12345"); // Too short
        assert!(invalid_color_result.is_err());

        let invalid_color_result2 = domain::HexColor::parse("#GGGGGG"); // Invalid characters
        assert!(invalid_color_result2.is_err());

        // Valid color should work
        let valid_color = domain::HexColor::parse("#123456")?;
        let fake_code = generate_fake_code();
        let fake_name = generate_fake_name();

        let category = database::Category {
            id: domain::RowID::new(),
            code: fake_code,
            name: fake_name,
            description: None,
            url_slug: None,
            category_type: domain::CategoryTypes::Expense,
            color: Some(valid_color),
            icon: None,
            is_active: true,
            created_on: chrono::Utc::now(),
            updated_on: chrono::Utc::now(),
        };

        // This should succeed since our domain type validates the color
        let result = category.insert(&pool).await;
        assert!(result.is_ok());

        Ok(())
    }

    #[sqlx::test]
    async fn insert_many_empty_list(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let categories: Vec<database::Category> = vec![];

        let result = database::Category::insert_many(&categories, &pool).await?;

        assert!(result.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn insert_many_success(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let categories = vec![
            generate_fake_category(),
            generate_fake_category(),
            generate_fake_category(),
        ];

        let inserted = database::Category::insert_many(&categories, &pool).await?;

        assert_eq!(inserted.len(), 3);
        for (original, inserted_cat) in categories.iter().zip(inserted.iter()) {
            assert_eq!(original.id, inserted_cat.id);
            assert_eq!(original.code, inserted_cat.code);
            assert_eq!(original.name, inserted_cat.name);
        }

        Ok(())
    }

    #[sqlx::test]
    async fn insert_many_atomic_failure(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let duplicate_code = generate_fake_code();

        let mut category1 = generate_fake_category();
        category1.code = duplicate_code.clone();

        let mut category2 = generate_fake_category();
        category2.code = duplicate_code.clone(); // Duplicate code

        let categories = vec![category1, category2];

        let result = database::Category::insert_many(&categories, &pool).await;
        assert!(result.is_err());

        // Verify neither category was inserted due to transaction rollback
        let code_prefix = format!("{}%", duplicate_code.split('.').next().unwrap_or(""));
        let count_query = sqlx::query!("SELECT COUNT(*) as count FROM categories WHERE code LIKE ?", code_prefix)
            .fetch_one(&pool)
            .await?;

        assert_eq!(count_query.count, 0);

        Ok(())
    }

    #[sqlx::test]
    async fn insert_or_update_creates_new(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let category = generate_fake_category();

        let result = database::Category::insert_or_update(&category, &pool).await?;

        assert_eq!(result.id, category.id);
        assert_eq!(result.code, category.code);

        Ok(())
    }

    #[sqlx::test]
    async fn insert_or_update_updates_existing(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let mut category = generate_fake_category();
        let fake_code = generate_fake_code();
        let original_name = generate_fake_name();
        let updated_name = generate_fake_name();

        category.code = fake_code.clone();
        category.name = original_name.clone();

        // Insert initially
        let inserted = database::Category::insert_or_update(&category, &pool).await?;
        assert_eq!(inserted.name, original_name);

        // Update the category
        category.name = updated_name.clone();
        category.updated_on = chrono::Utc::now();

        // Upsert should update
        let updated = database::Category::insert_or_update(&category, &pool).await?;
        assert_eq!(updated.id, category.id);
        assert_eq!(updated.code, fake_code); // Unchanged
        assert_eq!(updated.name, updated_name); // Updated

        Ok(())
    }

    #[sqlx::test]
    async fn insert_or_update_preserves_created_on(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let category = generate_fake_category();
        let original_created_on = category.created_on;

        // Insert
        let inserted = database::Category::insert_or_update(&category, &pool).await?;
        assert_eq!(inserted.created_on, original_created_on);

        // Update
        let updated_category = database::Category {
            updated_on: chrono::Utc::now(),
            ..category
        };

        let updated = database::Category::insert_or_update(&updated_category, &pool).await?;
        assert_eq!(updated.created_on, original_created_on); // Should be preserved
        assert_ne!(updated.updated_on, original_created_on); // Should be updated

        Ok(())
    }

    #[sqlx::test]
    async fn insert_or_update_fails_on_constraint_violation(pool: sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
        let duplicate_code = generate_fake_code();
        let fake_name1 = generate_fake_name();
        let fake_name2 = generate_fake_name();

        // Insert first category
        let category1 = database::Category {
            id: domain::RowID::new(),
            code: duplicate_code.clone(),
            name: fake_name1,
            description: None,
            url_slug: None,
            category_type: domain::CategoryTypes::Expense,
            color: None,
            icon: None,
            is_active: true,
            created_on: chrono::Utc::now(),
            updated_on: chrono::Utc::now(),
        };

        database::Category::insert_or_update(&category1, &pool).await?;

        // Try to upsert a different category with same code (but different ID)
        let category2 = database::Category {
            id: domain::RowID::new(), // Different ID
            code: duplicate_code, // Same code - should fail
            name: fake_name2,
            description: None,
            url_slug: None,
            category_type: domain::CategoryTypes::Expense,
            color: None,
            icon: None,
            is_active: true,
            created_on: chrono::Utc::now(),
            updated_on: chrono::Utc::now(),
        };

        let result = database::Category::insert_or_update(&category2, &pool).await;
        assert!(result.is_err());

        Ok(())
    }

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