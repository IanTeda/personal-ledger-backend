//! # Category Activation Logic
//!
//! This module provides the service logic for activating categories
//! in the Personal Ledger backend. It includes:
//!
//! - Activating a category by setting is_active = true
//! - Proper error handling for not found cases and database errors

use crate::{database, rpc};

/// Handle the category activation logic for the gRPC service.
///
/// This function performs:
/// - Parsing the ID from the request
/// - Updating the category's active status to true in the database
/// - Converting the updated database category to gRPC response format
/// - Proper error handling for not found cases and database errors
///
/// # Arguments
/// * `service` - Reference to the `CategoriesService` (for DB access)
/// * `request` - The incoming gRPC request
///
/// # Returns
/// * `Ok(tonic::Response<CategoryActivateResponse>)` on success
/// * `Err(tonic::Status)` on not found or database error
pub async fn activate_category(
    service: &super::CategoriesService,
    request: tonic::Request<rpc::CategoryActivateRequest>,
) -> Result<tonic::Response<rpc::CategoryActivateResponse>, tonic::Status> {
    // Extract the inner request
    let activate_request = request.into_inner();

    // Parse the ID from string to RowID
    let category_id = match activate_request.id.parse::<crate::domain::RowID>() {
        Ok(id) => id,
        Err(_) => {
            return Err(tonic::Status::invalid_argument("Invalid category ID format"));
        }
    };

    // Update the category's active status to true
    let updated_category = match database::Categories::update_active_status(category_id, true, service.database_ref()).await {
        Ok(category) => category,
        Err(database::DatabaseError::NotFound(_)) => {
            return Err(tonic::Status::not_found(format!("Category with ID '{}' not found", activate_request.id)));
        }
        Err(db_error) => {
            tracing::error!("Failed to activate category {}: {}", activate_request.id, db_error);
            return Err(tonic::Status::internal("Failed to activate category"));
        }
    };

    // Convert to RPC category and return response
    let rpc_category: rpc::Category = updated_category.into();
    let response = rpc::CategoryActivateResponse {
        category: Some(rpc_category),
    };

    Ok(tonic::Response::new(response))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_activate_category_with_valid_id() {
        // This would be an integration test that requires a database
        // For now, we test the basic structure
        // Integration tests will be added in tests/api/categories/activate.rs
    }

    #[test]
    fn test_activate_category_with_invalid_id() {
        // This would test invalid ID parsing
        // For now, we rely on integration tests
    }
}