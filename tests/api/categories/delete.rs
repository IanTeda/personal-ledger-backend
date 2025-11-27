use personal_ledger_backend::rpc;
use personal_ledger_backend::domain;

use crate::{categories, helpers};

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn delete_succeeds_with_existing_category(database_pool: sqlx::SqlitePool) -> Result<()> {
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

    // Now delete the category
    let delete_request_message = rpc::CategoryDeleteRequest {
        id: created_category.id.clone()
    };
    let delete_request = tonic::Request::new(delete_request_message);
    let delete_response = tonic_client.category().category_delete(delete_request).await?;
    let delete_response_message = delete_response.into_inner();

    // Assert that the response indicates 1 row was deleted
    assert_eq!(delete_response_message.rows_deleted, 1);

    // Verify the category is actually deleted by trying to get it
    let get_request_message = rpc::CategoryGetRequest {
        id: created_category.id.clone()
    };
    let get_request = tonic::Request::new(get_request_message);
    let get_error = tonic_client.category().category_get(get_request).await
        .expect_err("Expected get to fail after deletion");

    assert_eq!(get_error.code(), tonic::Code::NotFound);

    Ok(())
}

#[sqlx::test]
async fn delete_returns_zero_rows_for_nonexistent_category(database_pool: sqlx::SqlitePool) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to delete a category that doesn't exist
    let fake_id = domain::RowID::new().to_string();
    let delete_request_message = rpc::CategoryDeleteRequest {
        id: fake_id
    };
    let delete_request = tonic::Request::new(delete_request_message);
    let delete_response = tonic_client.category().category_delete(delete_request).await?;
    let delete_response_message = delete_response.into_inner();

    // Assert that the response indicates 0 rows were deleted
    assert_eq!(delete_response_message.rows_deleted, 0);

    Ok(())
}

#[sqlx::test]
async fn delete_fails_with_invalid_id_format(database_pool: sqlx::SqlitePool) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to delete with an invalid ID format
    let delete_request_message = rpc::CategoryDeleteRequest {
        id: "invalid-id-format".to_string()
    };
    let delete_request = tonic::Request::new(delete_request_message);

    // This should fail with an invalid argument error
    let error = tonic_client.category().category_delete(delete_request).await
        .expect_err("Expected delete to fail with invalid ID");

    assert_eq!(error.code(), tonic::Code::InvalidArgument);
    assert!(error.message().contains("Invalid category ID format"));

    Ok(())
}

#[sqlx::test]
async fn delete_category_twice_returns_zero_on_second_attempt(database_pool: sqlx::SqlitePool) -> Result<()> {
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

    // Delete the category once
    let delete_request_message = rpc::CategoryDeleteRequest {
        id: created_category.id.clone()
    };
    let delete_request = tonic::Request::new(delete_request_message.clone());
    let delete_response = tonic_client.category().category_delete(delete_request).await?;
    let delete_response_message = delete_response.into_inner();

    // Assert that the first delete indicates 1 row was deleted
    assert_eq!(delete_response_message.rows_deleted, 1);

    // Try to delete the same category again
    let delete_request2 = tonic::Request::new(delete_request_message);
    let delete_response2 = tonic_client.category().category_delete(delete_request2).await?;
    let delete_response_message2 = delete_response2.into_inner();

    // Assert that the second delete indicates 0 rows were deleted
    assert_eq!(delete_response_message2.rows_deleted, 0);

    Ok(())
}