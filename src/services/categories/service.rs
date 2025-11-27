//! Category service module for handling gRPC requests related to categories.
//!
//! This module provides the `CategoriesService` struct and its implementations
//! for managing category data through gRPC endpoints. It includes functionality
//! for creating, reading, updating, deleting, and batch operations on categories,
//! as well as conversions between database and RPC models.
//!
//! # Examples
//!
//! Basic usage involves creating a service instance and using it in a gRPC server:
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use sqlx::SqlitePool;
//! use personal_ledger_backend::{CategoriesService, LedgerConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let pool = Arc::new(SqlitePool::connect("sqlite::memory:").await?);
//! let config = LedgerConfig::default();
//! let service = CategoriesService::new(pool, config);
//! // Use service in gRPC server setup...
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use crate::{database, rpc, LedgerConfig};
use tonic;

/// Service for handling category-related gRPC requests.
///
/// Provides methods to interact with category data via gRPC, including CRUD operations
/// and batch processing. It holds references to the database pool and ledger configuration.
///
/// # Examples
///
/// Creating a new service instance:
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use sqlx::SqlitePool;
/// use personal_ledger_backend::{CategoriesService, LedgerConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = Arc::new(SqlitePool::connect("sqlite::memory:").await?);
/// let config = LedgerConfig::default();
/// let service = CategoriesService::new(pool, config);
/// # Ok(())
/// # }
/// ```
pub struct CategoriesService {
    database_pool: Arc<sqlx::SqlitePool>,
    #[allow(dead_code)]
    ledger_config: Arc<LedgerConfig>,
}

impl CategoriesService {
    /// Create a new CategoriesService passing in the Arc for the Sqlx database pool.
    ///
    /// # Arguments
    ///
    /// * `database_pool` - An Arc-wrapped SqlitePool for database operations.
    /// * `ledger_config` - The ledger configuration settings.
    ///
    /// # Returns
    ///
    /// A new instance of `CategoriesService`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use sqlx::SqlitePool;
    /// use personal_ledger_backend::{CategoriesService, LedgerConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = Arc::new(SqlitePool::connect("sqlite::memory:").await?);
    /// let config = LedgerConfig::default();
    /// let service = CategoriesService::new(pool, config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(database_pool: Arc<sqlx::SqlitePool>, ledger_config: Arc<LedgerConfig>) -> Self {
        Self { database_pool, ledger_config }
    }

    /// Shorthand for reference to database pool.
    ///
    /// # Returns
    ///
    /// A reference to the SqlitePool.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use sqlx::SqlitePool;
    /// use personal_ledger_backend::{CategoriesService, LedgerConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = Arc::new(SqlitePool::connect("sqlite::memory:").await?);
    /// let config = LedgerConfig::default();
    /// let service = CategoriesService::new(pool, config);
    /// let db_ref = service.database_ref();
    /// // Use db_ref for queries...
    /// # Ok(())
    /// # }
    /// ```
    pub fn database_ref(&self) -> &sqlx::SqlitePool {
        &self.database_pool
    }
}

/// Convert a database::Category into a Category Response message.
///
/// This implementation allows seamless conversion from the internal database model
/// to the gRPC response model, mapping fields appropriately.
///
/// # Examples
///
/// Converting a database category to RPC format:
///
/// ```rust
/// use personal_ledger_backend::database::Categories;
/// use personal_ledger_backend::rpc::Category;
/// use personal_ledger_backend::domain::{RowID, CategoryTypes};
/// use chrono::Utc;
///
/// let db_category = Categories {
///     id: RowID::new(),
///     code: "FOOD".to_string(),
///     name: "Food Expenses".to_string(),
///     description: Some("Expenses for food".to_string()),
///     url_slug: None,
///     category_type: CategoryTypes::Expense,
///     color: None,
///     icon: None,
///     is_active: true,
///     created_on: Utc::now(),
///     updated_on: Utc::now(),
/// };
///
/// let rpc_category: Category = db_category.into();
/// assert_eq!(rpc_category.code, "FOOD");
/// ```
impl From<database::Categories> for rpc::Category {
    fn from(category: database::Categories) -> Self {
        use prost_types::Timestamp;

        Self {
            id: category.id.to_string(),
            code: category.code,
            name: category.name,
            description: category.description,
            url_slug: category.url_slug.map(|s| s.to_string()),
            category_type: category.category_type.to_rpc_i32(),
            color: category.color.map(|c| c.to_string()),
            icon: category.icon,
            is_active: category.is_active,
            created_on: Some(Timestamp {
                seconds: category.created_on.timestamp(),
                nanos: category.created_on.timestamp_subsec_nanos() as i32,
            }),
            updated_on: Some(Timestamp {
                seconds: category.updated_on.timestamp(),
                nanos: category.updated_on.timestamp_subsec_nanos() as i32,
            }),
        }
    }
}

#[tonic::async_trait]
impl crate::rpc::CategoriesService for CategoriesService {
    /// Activate a category by setting its active status to true.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing the category ID to activate.
    ///
    /// # Returns
    ///
    /// A gRPC response indicating success or an error status.
    async fn category_activate(
        &self,
        request: tonic::Request<crate::rpc::CategoryActivateRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryActivateResponse>, tonic::Status> {
        crate::services::categories::activate_category(self, request).await
    }

    /// Handle RPC requests to create a category in the database.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing category details to create.
    ///
    /// # Returns
    ///
    /// A gRPC response with the created category or an error status.
    async fn category_create(
        &self,
        request: tonic::Request<crate::rpc::CategoryCreateRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryCreateResponse>, tonic::Status> {
        crate::services::categories::create_category(self, request).await
    }

    /// Create multiple categories in batch.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing a list of categories to create.
    ///
    /// # Returns
    ///
    /// A gRPC response with the created categories or an error status.
    async fn categories_create_batch(
        &self,
        request: tonic::Request<crate::rpc::CategoriesCreateBatchRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoriesCreateBatchResponse>, tonic::Status> {
        crate::services::categories::create_batch_categories(self, request).await
    }

    /// Deactivate a category by setting its active status to false.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing the category ID to deactivate.
    ///
    /// # Returns
    ///
    /// A gRPC response indicating success or an error status.
    async fn category_deactivate(
        &self,
        request: tonic::Request<crate::rpc::CategoryDeactivateRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryDeactivateResponse>, tonic::Status> {
        crate::services::categories::deactivate_category(self, request).await
    }

    /// Delete a category from the database.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing the category ID to delete.
    ///
    /// # Returns
    ///
    /// A gRPC response indicating success or an error status.
    async fn category_delete(
        &self,
        request: tonic::Request<crate::rpc::CategoryDeleteRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryDeleteResponse>, tonic::Status> {
        crate::services::categories::delete_category(self, request).await
    }

    /// Delete multiple categories in batch.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing a list of category IDs to delete.
    ///
    /// # Returns
    ///
    /// A gRPC response indicating success or an error status.
    async fn categories_delete_batch(
        &self,
        request: tonic::Request<crate::rpc::CategoriesDeleteBatchRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoriesDeleteBatchResponse>, tonic::Status> {
        crate::services::categories::delete_categories_batch(self, request).await
    }

    /// List categories with optional filtering and pagination.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing list parameters.
    ///
    /// # Returns
    ///
    /// A gRPC response with a list of categories or an error status.
    async fn categories_list(
        &self,
        request: tonic::Request<crate::rpc::CategoriesListRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoriesListResponse>, tonic::Status> {
        crate::services::categories::list_categories(self, request).await
    }

    /// Get a category by its unique ID.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing the category ID.
    ///
    /// # Returns
    ///
    /// A gRPC response with the category details or an error status.
    async fn category_get(
        &self,
        request: tonic::Request<crate::rpc::CategoryGetRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryGetResponse>, tonic::Status> {
        crate::services::categories::get_category(self, request).await
    }

    /// Get a category by its code.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing the category code.
    ///
    /// # Returns
    ///
    /// A gRPC response with the category details or an error status.
    async fn category_get_by_code(
        &self,
        request: tonic::Request<crate::rpc::CategoryGetByCodeRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryGetByCodeResponse>, tonic::Status> {
        crate::services::categories::get_category_by_code(self, request).await
    }

    /// Get a category by its URL slug.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing the category slug.
    ///
    /// # Returns
    ///
    /// A gRPC response with the category details or an error status.
    async fn category_get_by_slug(
        &self,
        request: tonic::Request<crate::rpc::CategoryGetBySlugRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryGetBySlugResponse>, tonic::Status> {
        crate::services::categories::get_category_by_slug(self, request).await
    }

    /// Update an existing category in the database.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing updated category details.
    ///
    /// # Returns
    ///
    /// A gRPC response with the updated category or an error status.
    async fn category_update(
        &self,
        request: tonic::Request<crate::rpc::CategoryUpdateRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryUpdateResponse>, tonic::Status> {
        crate::services::categories::update_category(self, request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;
    use std::sync::Arc;
    use tokio;

    #[test]
    fn test_from_database_category_to_rpc_category() {
        // Create a mock database category
        let db_category = database::Categories::mock();

        // Convert to RPC category
        let rpc_category: rpc::Category = db_category.clone().into();

        // Assert all fields are correctly mapped
        assert_eq!(rpc_category.id, db_category.id.to_string());
        assert_eq!(rpc_category.code, db_category.code);
        assert_eq!(rpc_category.name, db_category.name);
        assert_eq!(rpc_category.description, db_category.description);
        assert_eq!(rpc_category.url_slug, db_category.url_slug.map(|s| s.to_string()));
        assert_eq!(rpc_category.color, db_category.color.map(|c| c.to_string()));
        assert_eq!(rpc_category.icon, db_category.icon);
        assert_eq!(rpc_category.is_active, db_category.is_active);

        // Check category_type mapping
        let expected_category_type = db_category.category_type.to_rpc_i32();
        assert_eq!(rpc_category.category_type, expected_category_type);

        // Check timestamps
        assert!(rpc_category.created_on.is_some());
        assert!(rpc_category.updated_on.is_some());

        let created_ts = rpc_category.created_on.as_ref().unwrap();
        assert_eq!(created_ts.seconds, db_category.created_on.timestamp());
        assert_eq!(created_ts.nanos as u32, db_category.created_on.timestamp_subsec_nanos());

        let updated_ts = rpc_category.updated_on.as_ref().unwrap();
        assert_eq!(updated_ts.seconds, db_category.updated_on.timestamp());
        assert_eq!(updated_ts.nanos as u32, db_category.updated_on.timestamp_subsec_nanos());
    }

    #[test]
    fn test_from_database_category_to_rpc_category_minimal_fields() {
        // Create a minimal database category
        let db_category = database::Categories {
            id: crate::domain::RowID::new(),
            code: "TEST".to_string(),
            name: "Test Category".to_string(),
            description: None,
            url_slug: None,
            category_type: crate::domain::CategoryTypes::Expense,
            color: None,
            icon: None,
            is_active: true,
            created_on: chrono::Utc::now(),
            updated_on: chrono::Utc::now(),
        };

        // Convert to RPC category
        let rpc_category: rpc::Category = db_category.clone().into();

        // Assert required fields are mapped
        assert_eq!(rpc_category.id, db_category.id.to_string());
        assert_eq!(rpc_category.code, db_category.code);
        assert_eq!(rpc_category.name, db_category.name);
        assert_eq!(rpc_category.category_type, db_category.category_type.to_rpc_i32());
        assert_eq!(rpc_category.is_active, db_category.is_active);

        // Assert optional fields are None/empty
        assert!(rpc_category.description.is_none());
        assert!(rpc_category.url_slug.is_none());
        assert!(rpc_category.color.is_none());
        assert!(rpc_category.icon.is_none());

        // Check timestamps are present
        assert!(rpc_category.created_on.is_some());
        assert!(rpc_category.updated_on.is_some());
    }

    #[test]
    fn test_from_database_category_to_rpc_category_with_all_fields() {
        // Create a fully populated database category
        let now = chrono::Utc::now();
        let db_category = database::Categories {
            id: crate::domain::RowID::new(),
            code: "FULL_TEST".to_string(),
            name: "Full Test Category".to_string(),
            description: Some("A comprehensive test category".to_string()),
            url_slug: Some(crate::domain::UrlSlug::from("full-test-category")),
            category_type: crate::domain::CategoryTypes::Income,
            color: Some(crate::domain::HexColor::parse("#FF5733").unwrap()),
            icon: Some("dollar-sign".to_string()),
            is_active: false,
            created_on: now,
            updated_on: now,
        };

        // Convert to RPC category
        let rpc_category: rpc::Category = db_category.clone().into();

        // Assert all fields are correctly mapped
        assert_eq!(rpc_category.id, db_category.id.to_string());
        assert_eq!(rpc_category.code, db_category.code);
        assert_eq!(rpc_category.name, db_category.name);
        assert_eq!(rpc_category.description, db_category.description);
        assert_eq!(rpc_category.url_slug, Some("full-test-category".to_string()));
        assert_eq!(rpc_category.color, Some("#FF5733".to_string()));
        assert_eq!(rpc_category.icon, db_category.icon);
        assert_eq!(rpc_category.is_active, db_category.is_active);
        assert_eq!(rpc_category.category_type, crate::domain::CategoryTypes::Income.to_rpc_i32());

        // Check timestamps
        assert!(rpc_category.created_on.is_some());
        assert!(rpc_category.updated_on.is_some());
    }

    #[tokio::test]
    async fn test_categories_service_new() {
        // Test that CategoriesService::new creates an instance correctly
        let pool = Arc::new(sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap());
        let config = Arc::new(crate::LedgerConfig::default());
        let _service = CategoriesService::new(pool.clone(), config.clone());

        // Verify the pool is shared
        assert_eq!(Arc::strong_count(&pool), 2);
        assert_eq!(Arc::strong_count(&config), 2);
    }

    #[tokio::test]
    async fn test_database_ref() {
        // Test that database_ref returns the correct reference
        let pool = Arc::new(sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap());
        let config = Arc::new(crate::LedgerConfig::default());
        let service = CategoriesService::new(pool.clone(), config);

        let db_ref = service.database_ref();
        // Check that the references point to the same pool instance
        assert!(std::ptr::eq(db_ref, &*pool));
    }
}
