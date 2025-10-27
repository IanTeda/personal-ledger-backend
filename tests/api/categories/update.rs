use personal_ledger_backend::rpc;

use crate::{categories, helpers};

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn update_succeeds_with_existing_category(database_pool: sqlx::SqlitePool) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Create a category first
    let rpc_category = categories::mock_rpc_category();
    let create_request_message = rpc::CategoryCreateRequest {
        category: Some(rpc_category.clone())
    };
    let create_request = tonic::Request::new(create_request_message);
    let create_response = tonic_client.category().category_create(create_request).await?;
    let created_category = create_response.into_inner().category.unwrap();

    // Prepare updated data
    let mut updated_category = created_category.clone();
    updated_category.name = "Updated Category Name".to_string();
    updated_category.description = Some("Updated description".to_string());
    updated_category.is_active = false;

    // Now update the category
    let update_request_message = rpc::CategoryUpdateRequest {
        id: created_category.id.clone(),
        category: Some(updated_category.clone()),
        update_mask: None, // Update all fields
    };
    let update_request = tonic::Request::new(update_request_message);
    let update_response = tonic_client.category().category_update(update_request).await?;
    let update_response_message = update_response.into_inner();

    // Assert that the response contains the updated category
    assert!(update_response_message.category.is_some());
    let result_category = update_response_message.category.unwrap();

    // Verify that the category was updated
    assert_eq!(result_category.id, created_category.id);
    assert_eq!(result_category.name, "Updated Category Name");
    assert_eq!(result_category.description, Some("Updated description".to_string()));
    assert!(!result_category.is_active);

    // Verify that unchanged fields remain the same
    assert_eq!(result_category.code, created_category.code);
    assert_eq!(result_category.url_slug, created_category.url_slug);
    assert_eq!(result_category.category_type, created_category.category_type);
    assert_eq!(result_category.color, created_category.color);
    assert_eq!(result_category.icon, created_category.icon);

    // Verify that updated_on is newer than created_on
    assert!(result_category.updated_on.as_ref().unwrap().seconds >= created_category.updated_on.as_ref().unwrap().seconds);

    Ok(())
}

#[sqlx::test]
async fn update_fails_with_nonexistent_category(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to update a category with a fake ID
    let fake_id = helpers::mock_row_id().to_string();
    let update_request_message = rpc::CategoryUpdateRequest {
        id: fake_id,
        category: Some(categories::mock_rpc_category()),
        update_mask: None,
    };
    let update_request = tonic::Request::new(update_request_message);
    let result = tonic_client.category().category_update(update_request).await;
    assert!(result.is_err());

    Ok(())
}
