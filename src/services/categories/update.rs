//! # Category Update Logic
//!
//! This module provides the service logic for updating categories
//! in the Personal Ledger backend. It includes:
//!
//! - Partial and full category updates with field masking
//! - Proper validation and error handling

use crate::{database, rpc, services::ServiceError};
use prost_types::FieldMask;

/// Handle the category update logic for the gRPC service.
///
/// This function performs:
/// - Parsing the ID from the request
/// - Retrieving the existing category from the database
/// - Applying partial or full updates based on the field mask
/// - Updating the category in the database
/// - Converting the updated category back to gRPC response format
/// - Proper error handling for not found cases and database errors
///
/// # Arguments
/// * `service` - Reference to the `CategoriesService` (for DB access)
/// * `request` - The incoming gRPC request
///
/// # Returns
/// * `Ok(tonic::Response<CategoryUpdateResponse>)` on success
/// * `Err(tonic::Status)` on not found, validation error, or database error
pub async fn update_category(
    service: &super::CategoriesService,
    request: tonic::Request<rpc::CategoryUpdateRequest>,
) -> Result<tonic::Response<rpc::CategoryUpdateResponse>, tonic::Status> {
    // Extract the inner request
    let update_request = request.into_inner();

    // Get the category data from the request first
    let new_category_data = update_request.category.ok_or_else(|| {
        tonic::Status::invalid_argument("Category field is required in CategoryUpdateRequest")
    })?;

    // Parse the ID from string to RowID
    let category_id = match update_request.id.parse::<crate::domain::RowID>() {
        Ok(id) => id,
        Err(_) => {
            return Err(tonic::Status::invalid_argument("Invalid category ID format"));
        }
    };

    // Retrieve the existing category
    let existing_category = match database::Categories::find_by_id(category_id, service.database_ref()).await {
        Ok(Some(category)) => category,
        Ok(None) => {
            return Err(tonic::Status::not_found(format!("Category with ID '{}' not found", update_request.id)));
        }
        Err(db_error) => {
            tracing::error!("Failed to find category {}: {}", update_request.id, db_error);
            return Err(tonic::Status::internal("Failed to retrieve category"));
        }
    };

    // Apply the updates based on the field mask
    let updated_category = match apply_field_mask_updates(existing_category, new_category_data, update_request.update_mask) {
        Ok(category) => category,
        Err(service_error) => {
            // Convert ServiceError to tonic::Status
            let status_code = match service_error.http_status_code() {
                400 => tonic::Code::InvalidArgument,
                401 => tonic::Code::Unauthenticated,
                404 => tonic::Code::NotFound,
                422 => tonic::Code::FailedPrecondition,
                500 => tonic::Code::Internal,
                502 => tonic::Code::Unavailable,
                _ => tonic::Code::Internal,
            };
            return Err(tonic::Status::new(status_code, service_error.to_string()));
        }
    };

    // Update the category in the database
    let saved_category = match updated_category.update(service.database_ref()).await {
        Ok(category) => category,
        Err(database::DatabaseError::NotFound(_)) => {
            return Err(tonic::Status::not_found(format!("Category with ID '{}' not found", update_request.id)));
        }
        Err(database::DatabaseError::Validation(msg)) => {
            return Err(tonic::Status::invalid_argument(msg));
        }
        Err(db_error) => {
            tracing::error!("Failed to update category {}: {}", update_request.id, db_error);
            return Err(tonic::Status::internal("Failed to update category"));
        }
    };

    // Convert to RPC category and return response
    let rpc_category: rpc::Category = saved_category.into();
    let response = rpc::CategoryUpdateResponse {
        category: Some(rpc_category),
    };

    Ok(tonic::Response::new(response))
}

/// Apply field mask updates to an existing category.
///
/// This function takes an existing category and applies updates from the new category data,
/// but only for the fields specified in the field mask. If no field mask is provided,
/// all fields are updated.
///
/// # Arguments
/// * `existing` - The existing category from the database
/// * `new_data` - The new category data from the request
/// * `field_mask` - Optional field mask specifying which fields to update
///
/// # Returns
/// * `Ok(database::Categories)` with the updated category
/// * `Err(ServiceError)` on validation errors
fn apply_field_mask_updates(
    mut existing: database::Categories,
    new_data: rpc::Category,
    field_mask: Option<FieldMask>,
) -> Result<database::Categories, ServiceError> {
    // If no field mask is provided, update all fields
    let field_mask = field_mask.unwrap_or_else(|| FieldMask {
        paths: vec![
            "code".to_string(),
            "name".to_string(),
            "description".to_string(),
            "url_slug".to_string(),
            "category_type".to_string(),
            "color".to_string(),
            "icon".to_string(),
            "is_active".to_string(),
        ],
    });

    // Apply updates for each field in the mask
    for path in &field_mask.paths {
        match path.as_str() {
            "code" => {
                if new_data.code.trim().is_empty() {
                    return Err(ServiceError::validation("Category code cannot be empty"));
                }
                existing.code = new_data.code.clone();
            }
            "name" => {
                if new_data.name.trim().is_empty() {
                    return Err(ServiceError::validation("Category name cannot be empty"));
                }
                existing.name = new_data.name.clone();
            }
            "description" => {
                existing.description = new_data.description.clone();
            }
            "url_slug" => {
                existing.url_slug = if let Some(slug) = &new_data.url_slug {
                    if slug.trim().is_empty() {
                        None
                    } else {
                        Some(crate::domain::UrlSlug::parse(slug.clone())?)
                    }
                } else {
                    None
                };
            }
            "category_type" => {
                existing.category_type = crate::domain::CategoryTypes::from_rpc_i32(new_data.category_type)
                    .map_err(|e| ServiceError::validation(&e))?;
            }
            "color" => {
                existing.color = if let Some(color_str) = &new_data.color {
                    if color_str.trim().is_empty() {
                        None
                    } else {
                        Some(crate::domain::HexColor::parse(color_str.clone())?)
                    }
                } else {
                    None
                };
            }
            "icon" => {
                existing.icon = new_data.icon.clone();
            }
            "is_active" => {
                existing.is_active = new_data.is_active;
            }
            _ => {
                return Err(ServiceError::validation(format!("Unknown field in update mask: {}", path)));
            }
        }
    }

    // Always update the updated_on timestamp
    existing.updated_on = chrono::Utc::now();

    Ok(existing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain;
    use prost_types::FieldMask;

    #[test]
    fn test_apply_field_mask_updates_full_update() {
        // Create a mock existing category
        let existing = database::Categories::mock();

        // Create new data
        let new_data = rpc::Category {
            id: "different-id".to_string(),
            code: "NEWCODE".to_string(),
            name: "New Name".to_string(),
            description: Some("New description".to_string()),
            url_slug: Some("new-slug".to_string()),
            category_type: rpc::CategoryTypes::Income as i32,
            color: Some("#FF0000".to_string()),
            icon: Some("new-icon".to_string()),
            is_active: false,
            created_on: None,
            updated_on: None,
        };

        // No field mask means full update
        let result = apply_field_mask_updates(existing.clone(), new_data, None);

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.code, "NEWCODE");
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.description, Some("New description".to_string()));
        assert_eq!(updated.category_type, domain::CategoryTypes::Income);
        assert!(!updated.is_active);
    }

    #[test]
    fn test_apply_field_mask_updates_partial_update() {
        // Create a mock existing category
        let existing = database::Categories::mock();

        // Create new data with only name change
        let new_data = rpc::Category {
            id: "different-id".to_string(),
            code: "SAMECODE".to_string(), // This should be ignored
            name: "Updated Name".to_string(),
            description: None,
            url_slug: None,
            category_type: existing.category_type.to_rpc_i32(),
            color: None,
            icon: None,
            is_active: true,
            created_on: None,
            updated_on: None,
        };

        // Field mask for only name
        let field_mask = FieldMask {
            paths: vec!["name".to_string()],
        };

        let result = apply_field_mask_updates(existing.clone(), new_data, Some(field_mask));

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.name, "Updated Name");
        // Other fields should remain unchanged
        assert_eq!(updated.code, existing.code);
        assert_eq!(updated.is_active, existing.is_active);
    }

    #[test]
    fn test_apply_field_mask_updates_empty_code() {
        let existing = database::Categories::mock();
        let new_data = rpc::Category {
            code: "".to_string(),
            ..existing.clone().into()
        };

        let field_mask = FieldMask {
            paths: vec!["code".to_string()],
        };

        let result = apply_field_mask_updates(existing, new_data, Some(field_mask));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("code cannot be empty"));
    }

    #[test]
    fn test_apply_field_mask_updates_invalid_field() {
        let existing = database::Categories::mock();
        let new_data = rpc::Category {
            ..existing.clone().into()
        };

        let field_mask = FieldMask {
            paths: vec!["invalid_field".to_string()],
        };

        let result = apply_field_mask_updates(existing, new_data, Some(field_mask));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown field"));
    }
}