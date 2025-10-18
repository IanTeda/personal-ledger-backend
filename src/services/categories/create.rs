//! # Category Creation Logic
//!
//! This module provides the conversion and service logic for creating categories
//! in the Personal Ledger backend. It includes:
//!
//! - Conversion from gRPC `CategoryCreateRequest` to domain/database `Category`
//! - Validation and error handling for all fields
//! - The async service handler for category creation, used by the gRPC service
//! - Comprehensive unit tests for all conversion and validation logic
//!
//! The core business logic is abstracted here to keep the gRPC service layer clean
//! and to enable easier testing and maintenance.

use crate::{database, domain, rpc, services::ServiceError};
use chrono::Utc;

/// Convert a gRPC `CategoryCreateRequest` into a domain/database `Category`.
///
/// This implementation performs all necessary validation and conversion of fields,
/// ensuring that required fields are present and valid, and that optional fields
/// are handled correctly. Any validation or parsing errors are returned as a
/// `ServiceError`.
impl TryFrom<rpc::CategoryCreateRequest> for database::Category {
    type Error = ServiceError;

    fn try_from(value: rpc::CategoryCreateRequest) -> Result<Self, Self::Error> {
        let category = value.category.ok_or_else(|| ServiceError::validation(
            "Category field is required in CategoryCreateRequest"
        ))?;

        // Generate new ID for the category
        let id = domain::RowID::new();

        // Validate and parse required fields
        let code = if category.code.trim().is_empty() {
            return Err(ServiceError::validation("Category code is required and cannot be empty"));
        } else {
            category.code
        };

        let name = if category.name.trim().is_empty() {
            return Err(ServiceError::validation("Category name is required and cannot be empty"));
        } else {
            category.name
        };

        // Parse optional description
        let description = category.description.filter(|s| !s.trim().is_empty());

        // Parse optional URL slug
        let url_slug = if let Some(slug) = category.url_slug.filter(|s| !s.trim().is_empty()) {
            Some(domain::UrlSlug::parse(slug)?)
        } else {
            None
        };

        // Parse category type
        let category_type = domain::CategoryTypes::from_rpc_i32(category.category_type)
            .map_err(|e| ServiceError::validation(&e))?;

        // Parse optional color
        let color = if let Some(color_str) = category.color.filter(|s| !s.trim().is_empty()) {
            Some(domain::HexColor::parse(color_str)?)
        } else {
            None
        };

        // Parse optional icon
        let icon = category.icon.filter(|s| !s.trim().is_empty());

        // Default to active for new categories
        let is_active = category.is_active;

        // Set timestamps
        let created_on = Utc::now();
        let updated_on = created_on;

        Ok(database::Category {
            id,
            code,
            name,
            description,
            url_slug,
            category_type,
            color,
            icon,
            is_active,
            created_on,
            updated_on,
        })
    }
}

/// Handle the category creation logic for the gRPC service.
///
/// This function performs:
/// - Validation and conversion of the incoming gRPC request
/// - Insertion of the new category into the database
/// - Conversion of the inserted category back to gRPC response format
/// - Proper error handling and mapping to gRPC status codes
///
/// # Arguments
/// * `service` - Reference to the `CategoriesService` (for DB access)
/// * `request` - The incoming gRPC request
///
/// # Returns
/// * `Ok(tonic::Response<CategoryCreateResponse>)` on success
/// * `Err(tonic::Status)` on validation or database error
pub async fn create_category(
    service: &super::CategoriesService,
    request: tonic::Request<rpc::CategoryCreateRequest>,
) -> Result<tonic::Response<rpc::CategoryCreateResponse>, tonic::Status> {
    // Extract the inner request
    let create_request = request.into_inner();

    // Convert the request to a database category
    let category = match database::Category::try_from(create_request) {
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

    // Insert the category into the database
    let inserted_category = match category.insert(service.database_ref()).await {
        Ok(category) => category,
        Err(db_error) => {
            tracing::error!("Failed to insert category: {}", db_error);
            return Err(tonic::Status::internal("Failed to create category"));
        }
    };

    // Convert the database category back to RPC format
    let rpc_category: rpc::Category = inserted_category.into();

    // Create the response
    let response = rpc::CategoryCreateResponse {
        category: Some(rpc_category),
    };

    Ok(tonic::Response::new(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc;

    #[test]
    fn test_try_from_valid_category_create_request() {
        let request = rpc::CategoryCreateRequest {
            category: Some(rpc::Category {
                id: "will-be-ignored".to_string(),
                code: "GROCERY".to_string(),
                name: "Groceries".to_string(),
                description: Some("Weekly grocery expenses".to_string()),
                url_slug: Some("groceries".to_string()),
                category_type: rpc::CategoryTypes::Expense as i32,
                color: Some("#FF5733".to_string()),
                icon: Some("shopping-cart".to_string()),
                is_active: true,
                created_on: None, // Will be ignored
                updated_on: None, // Will be ignored
            }),
        };

        let result = database::Category::try_from(request);
        assert!(result.is_ok());

        let category = result.unwrap();
        assert_eq!(category.code, "GROCERY");
        assert_eq!(category.name, "Groceries");
        assert_eq!(category.description, Some("Weekly grocery expenses".to_string()));
        assert_eq!(category.url_slug.as_ref().unwrap().as_str(), "groceries");
        assert_eq!(category.category_type, domain::CategoryTypes::Expense);
        assert_eq!(category.color.as_ref().unwrap().as_str(), "#FF5733");
        assert_eq!(category.icon, Some("shopping-cart".to_string()));
        assert!(category.is_active);
    }

    #[test]
    fn test_try_from_minimal_category_create_request() {
        let request = rpc::CategoryCreateRequest {
            category: Some(rpc::Category {
                id: "".to_string(),
                code: "SAVINGS".to_string(),
                name: "Savings".to_string(),
                description: None,
                url_slug: None,
                category_type: rpc::CategoryTypes::Asset as i32,
                color: None,
                icon: None,
                is_active: false,
                created_on: None,
                updated_on: None,
            }),
        };

        let result = database::Category::try_from(request);
        assert!(result.is_ok());

        let category = result.unwrap();
        assert_eq!(category.code, "SAVINGS");
        assert_eq!(category.name, "Savings");
        assert!(category.description.is_none());
        assert!(category.url_slug.is_none());
        assert_eq!(category.category_type, domain::CategoryTypes::Asset);
        assert!(category.color.is_none());
        assert!(category.icon.is_none());
        assert!(!category.is_active);
    }

    #[test]
    fn test_try_from_missing_category_field() {
        let request = rpc::CategoryCreateRequest {
            category: None,
        };

        let result = database::Category::try_from(request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServiceError::Validation(_)));
    }

    #[test]
    fn test_try_from_empty_code() {
        let request = rpc::CategoryCreateRequest {
            category: Some(rpc::Category {
                id: "".to_string(),
                code: "".to_string(),
                name: "Test".to_string(),
                description: None,
                url_slug: None,
                category_type: rpc::CategoryTypes::Expense as i32,
                color: None,
                icon: None,
                is_active: true,
                created_on: None,
                updated_on: None,
            }),
        };

        let result = database::Category::try_from(request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServiceError::Validation(_)));
    }

    #[test]
    fn test_try_from_empty_name() {
        let request = rpc::CategoryCreateRequest {
            category: Some(rpc::Category {
                id: "".to_string(),
                code: "TEST".to_string(),
                name: "   ".to_string(), // whitespace only
                description: None,
                url_slug: None,
                category_type: rpc::CategoryTypes::Expense as i32,
                color: None,
                icon: None,
                is_active: true,
                created_on: None,
                updated_on: None,
            }),
        };

        let result = database::Category::try_from(request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServiceError::Validation(_)));
    }

    #[test]
    fn test_try_from_invalid_category_type() {
        let request = rpc::CategoryCreateRequest {
            category: Some(rpc::Category {
                id: "".to_string(),
                code: "TEST".to_string(),
                name: "Test".to_string(),
                description: None,
                url_slug: None,
                category_type: 999, // Invalid category type
                color: None,
                icon: None,
                is_active: true,
                created_on: None,
                updated_on: None,
            }),
        };

        let result = database::Category::try_from(request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServiceError::Validation(_)));
    }

    #[test]
    fn test_try_from_invalid_url_slug() {
        let request = rpc::CategoryCreateRequest {
            category: Some(rpc::Category {
                id: "".to_string(),
                code: "TEST".to_string(),
                name: "Test".to_string(),
                description: None,
                url_slug: Some("!@#$%^&*()".to_string()), // Only invalid characters
                category_type: rpc::CategoryTypes::Expense as i32,
                color: None,
                icon: None,
                is_active: true,
                created_on: None,
                updated_on: None,
            }),
        };

        let result = database::Category::try_from(request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServiceError::Validation(_)));
    }

    #[test]
    fn test_try_from_invalid_color() {
        let request = rpc::CategoryCreateRequest {
            category: Some(rpc::Category {
                id: "".to_string(),
                code: "TEST".to_string(),
                name: "Test".to_string(),
                description: None,
                url_slug: None,
                category_type: rpc::CategoryTypes::Expense as i32,
                color: Some("INVALID".to_string()),
                icon: None,
                is_active: true,
                created_on: None,
                updated_on: None,
            }),
        };

        let result = database::Category::try_from(request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServiceError::Validation(_)));
    }

    #[test]
    fn test_try_from_empty_optional_fields() {
        let request = rpc::CategoryCreateRequest {
            category: Some(rpc::Category {
                id: "".to_string(),
                code: "TEST".to_string(),
                name: "Test".to_string(),
                description: Some("".to_string()), // Empty string should become None
                url_slug: Some("   ".to_string()), // Whitespace should become None
                category_type: rpc::CategoryTypes::Expense as i32,
                color: Some("".to_string()), // Empty string should become None
                icon: Some("   ".to_string()), // Whitespace should become None
                is_active: true,
                created_on: None,
                updated_on: None,
            }),
        };

        let result = database::Category::try_from(request);
        assert!(result.is_ok());

        let category = result.unwrap();
        assert!(category.description.is_none());
        assert!(category.url_slug.is_none());
        assert!(category.color.is_none());
        assert!(category.icon.is_none());
    }
}