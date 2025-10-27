// #![allow(unused)] // For development only

use std::sync::Arc;

use crate::{database, rpc, LedgerConfig};
use tonic;

pub struct CategoriesService {
    database_pool: Arc<sqlx::SqlitePool>,
    ledger_config: Arc<LedgerConfig>,
}

impl CategoriesService {
    /// Create a new CategoriesService passing in the Arc for the Sqlx database pool
    pub fn new(database_pool: Arc<sqlx::SqlitePool>, ledger_config: Arc<LedgerConfig>) -> Self {
        Self { database_pool, ledger_config }
    }

    /// Shorthand for reference to database pool
    // https://github.com/radhas-kitchen/radhas-kitchen/blob/fe0cc02ddd9275d9b6aa97300701a53618980c9f/src-grpc/src/services/auth.rs#L10
    pub fn database_ref(&self) -> &sqlx::SqlitePool {
        &self.database_pool
    }

    #[allow(dead_code)]
    fn config_ref(&self) -> &LedgerConfig {
        &self.ledger_config
    }
}

/// Convert a database::Category into a Category Response message
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
    /// Handle rpc requests to create a category in the database
    async fn category_create(
        &self,
        request: tonic::Request<crate::rpc::CategoryCreateRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryCreateResponse>, tonic::Status> {
        crate::services::categories::create::create_category(self, request).await
    }

    async fn category_get(
        &self,
        _request: tonic::Request<crate::rpc::CategoryGetRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryGetResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("category_get not implemented"))
    }

    async fn category_get_by_code(
        &self,
        _request: tonic::Request<crate::rpc::CategoryGetByCodeRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryGetByCodeResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("category_get_by_code not implemented"))
    }

    async fn category_get_by_slug(
        &self,
        _request: tonic::Request<crate::rpc::CategoryGetBySlugRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryGetBySlugResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("category_get_by_slug not implemented"))
    }

    async fn categories_list(
        &self,
        _request: tonic::Request<crate::rpc::CategoriesListRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoriesListResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("categories_list not implemented"))
    }

    async fn category_update(
        &self,
        _request: tonic::Request<crate::rpc::CategoryUpdateRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryUpdateResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("category_update not implemented"))
    }

    async fn category_delete(
        &self,
        _request: tonic::Request<crate::rpc::CategoryDeleteRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryDeleteResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("category_delete not implemented"))
    }

    async fn category_activate(
        &self,
        _request: tonic::Request<crate::rpc::CategoryActivateRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryActivateResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("category_activate not implemented"))
    }

    async fn category_deactivate(
        &self,
        _request: tonic::Request<crate::rpc::CategoryDeactivateRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoryDeactivateResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("category_deactivate not implemented"))
    }

    async fn categories_create_batch(
        &self,
        request: tonic::Request<crate::rpc::CategoriesCreateBatchRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoriesCreateBatchResponse>, tonic::Status> {
        crate::services::categories::create::create_batch_categories(self, request).await
    }

    async fn categories_delete_batch(
        &self,
        _request: tonic::Request<crate::rpc::CategoriesDeleteBatchRequest>,
    ) -> Result<tonic::Response<crate::rpc::CategoriesDeleteBatchResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("categories_delete_batch not implemented"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

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
}
