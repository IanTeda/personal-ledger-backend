use personal_ledger_backend::rpc;
use personal_ledger_backend::domain;

use crate::{categories, helpers};

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn activate_succeeds_with_existing_inactive_category(database_pool: sqlx::SqlitePool) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Create an inactive category first
    let mut rpc_category = categories::mock_rpc_category();
    rpc_category.is_active = false; // Ensure it's inactive
    let create_request_message = rpc::CategoryCreateRequest {
        category: Some(rpc_category.clone())
    };
    let create_request = tonic::Request::new(create_request_message);
    let create_response = tonic_client.category().category_create(create_request).await?;
    let created_category = create_response.into_inner().category.unwrap();

    // Verify the category was created as inactive
    assert!(!created_category.is_active);

    // Now activate the category
    let activate_request_message = rpc::CategoryActivateRequest {
        id: created_category.id.clone()
    };
    let activate_request = tonic::Request::new(activate_request_message);
    let activate_response = tonic_client.category().category_activate(activate_request).await?;
    let activate_response_message = activate_response.into_inner();

    // Assert that the response contains the activated category
    assert!(activate_response_message.category.is_some());
    let activated_category = activate_response_message.category.unwrap();

    // Verify that the category is now active
    assert!(activated_category.is_active);

    // Verify other fields remain unchanged
    assert_eq!(activated_category.id, created_category.id);
    assert_eq!(activated_category.code, created_category.code);
    assert_eq!(activated_category.name, created_category.name);
    assert_eq!(activated_category.description, created_category.description);
    assert_eq!(activated_category.url_slug, created_category.url_slug);
    assert_eq!(activated_category.category_type, created_category.category_type);
    assert_eq!(activated_category.color, created_category.color);
    assert_eq!(activated_category.icon, created_category.icon);

    // Verify timestamps are present and updated_on is newer
    assert!(activated_category.created_on.is_some());
    assert!(activated_category.updated_on.is_some());

    let created_time = created_category.created_on.as_ref().unwrap();
    let activated_time = activated_category.updated_on.as_ref().unwrap();
    assert!(activated_time.seconds >= created_time.seconds);

    Ok(())
}

#[sqlx::test]
async fn activate_succeeds_with_already_active_category(database_pool: sqlx::SqlitePool) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Create an active category first
    let mut rpc_category = categories::mock_rpc_category();
    rpc_category.is_active = true; // Ensure it's active
    let create_request_message = rpc::CategoryCreateRequest {
        category: Some(rpc_category.clone())
    };
    let create_request = tonic::Request::new(create_request_message);
    let create_response = tonic_client.category().category_create(create_request).await?;
    let created_category = create_response.into_inner().category.unwrap();

    // Verify the category was created as active
    assert!(created_category.is_active);

    // Now activate the already active category (should still work)
    let activate_request_message = rpc::CategoryActivateRequest {
        id: created_category.id.clone()
    };
    let activate_request = tonic::Request::new(activate_request_message);
    let activate_response = tonic_client.category().category_activate(activate_request).await?;
    let activate_response_message = activate_response.into_inner();

    // Assert that the response contains the category
    assert!(activate_response_message.category.is_some());
    let activated_category = activate_response_message.category.unwrap();

    // Verify that the category is still active
    assert!(activated_category.is_active);

    // Verify other fields remain unchanged
    assert_eq!(activated_category.id, created_category.id);
    assert_eq!(activated_category.code, created_category.code);
    assert_eq!(activated_category.name, created_category.name);

    Ok(())
}

#[sqlx::test]
async fn activate_fails_with_invalid_id_format(database_pool: sqlx::SqlitePool) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to activate with an invalid ID format
    let activate_request_message = rpc::CategoryActivateRequest {
        id: "invalid-id-format".to_string()
    };
    let activate_request = tonic::Request::new(activate_request_message);

    // This should fail with an invalid argument error
    let error = tonic_client.category().category_activate(activate_request).await
        .expect_err("Expected activate to fail with invalid ID");

    assert_eq!(error.code(), tonic::Code::InvalidArgument);
    assert!(error.message().contains("Invalid category ID format"));

    Ok(())
}

#[sqlx::test]
async fn activate_fails_with_nonexistent_category(database_pool: sqlx::SqlitePool) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to activate a category that doesn't exist
    let fake_id = domain::RowID::new().to_string();
    let activate_request_message = rpc::CategoryActivateRequest {
        id: fake_id.clone()
    };
    let activate_request = tonic::Request::new(activate_request_message);

    // This should fail with a not found error
    let error = tonic_client.category().category_activate(activate_request).await
        .expect_err("Expected activate to fail with nonexistent category");

    assert_eq!(error.code(), tonic::Code::NotFound);
    assert!(error.message().contains(&format!("Category with ID '{}' not found", fake_id)));

    Ok(())
}