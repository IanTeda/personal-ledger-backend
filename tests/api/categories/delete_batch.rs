use personal_ledger_backend::rpc;
use personal_ledger_backend::domain;

use crate::{categories, helpers};

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn delete_batch_succeeds_with_existing_categories(database_pool: sqlx::SqlitePool) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Create multiple categories first
    let mut created_ids = Vec::new();
    for i in 0..3 {
        let mut rpc_category = categories::mock_rpc_category();
        rpc_category.name = format!("Test Category {}", i);
        rpc_category.code = format!("TEST{}", i);

        let create_request_message = rpc::CategoryCreateRequest {
            category: Some(rpc_category.clone())
        };
        let create_request = tonic::Request::new(create_request_message);
        let create_response = tonic_client.category().category_create(create_request).await?;
        let created_category = create_response.into_inner().category.unwrap();
        created_ids.push(created_category.id.clone());
    }

    // Now delete the categories in batch
    let delete_batch_request_message = rpc::CategoriesDeleteBatchRequest {
        ids: created_ids.clone()
    };
    let delete_batch_request = tonic::Request::new(delete_batch_request_message);
    let delete_batch_response = tonic_client.category().categories_delete_batch(delete_batch_request).await?;
    let delete_batch_response_message = delete_batch_response.into_inner();

    // Assert that the response indicates 3 rows were deleted
    assert_eq!(delete_batch_response_message.rows_deleted, 3);

    // Verify all categories are actually deleted by trying to get them
    for id in &created_ids {
        let get_request_message = rpc::CategoryGetRequest {
            id: id.clone()
        };
        let get_request = tonic::Request::new(get_request_message);
        let get_error = tonic_client.category().category_get(get_request).await
            .expect_err("Expected get to fail after deletion");

        assert_eq!(get_error.code(), tonic::Code::NotFound);
    }

    Ok(())
}

#[sqlx::test]
async fn delete_batch_returns_partial_count_for_mixed_existing_and_nonexistent(database_pool: sqlx::SqlitePool) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Create 2 categories
    let mut created_ids = Vec::new();
    for i in 0..2 {
        let mut rpc_category = categories::mock_rpc_category();
        rpc_category.name = format!("Test Category {}", i);
        rpc_category.code = format!("TEST{}", i);

        let create_request_message = rpc::CategoryCreateRequest {
            category: Some(rpc_category.clone())
        };
        let create_request = tonic::Request::new(create_request_message);
        let create_response = tonic_client.category().category_create(create_request).await?;
        let created_category = create_response.into_inner().category.unwrap();
        created_ids.push(created_category.id.clone());
    }

    // Add some fake IDs that don't exist
    let mut all_ids = created_ids.clone();
    all_ids.push(domain::RowID::new().to_string()); // Non-existent ID
    all_ids.push(domain::RowID::new().to_string()); // Another non-existent ID

    // Now delete the batch (some exist, some don't)
    let delete_batch_request_message = rpc::CategoriesDeleteBatchRequest {
        ids: all_ids
    };
    let delete_batch_request = tonic::Request::new(delete_batch_request_message);
    let delete_batch_response = tonic_client.category().categories_delete_batch(delete_batch_request).await?;
    let delete_batch_response_message = delete_batch_response.into_inner();

    // Assert that the response indicates 2 rows were deleted (only the existing ones)
    assert_eq!(delete_batch_response_message.rows_deleted, 2);

    // Verify the existing categories are deleted
    for id in &created_ids {
        let get_request_message = rpc::CategoryGetRequest {
            id: id.clone()
        };
        let get_request = tonic::Request::new(get_request_message);
        let get_error = tonic_client.category().category_get(get_request).await
            .expect_err("Expected get to fail after deletion");

        assert_eq!(get_error.code(), tonic::Code::NotFound);
    }

    Ok(())
}

#[sqlx::test]
async fn delete_batch_returns_zero_for_empty_list(database_pool: sqlx::SqlitePool) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to delete an empty batch
    let delete_batch_request_message = rpc::CategoriesDeleteBatchRequest {
        ids: vec![]
    };
    let delete_batch_request = tonic::Request::new(delete_batch_request_message);
    let delete_batch_response = tonic_client.category().categories_delete_batch(delete_batch_request).await?;
    let delete_batch_response_message = delete_batch_response.into_inner();

    // Assert that the response indicates 0 rows were deleted
    assert_eq!(delete_batch_response_message.rows_deleted, 0);

    Ok(())
}

#[sqlx::test]
async fn delete_batch_fails_with_invalid_id_format(database_pool: sqlx::SqlitePool) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to delete batch with an invalid ID format
    let delete_batch_request_message = rpc::CategoriesDeleteBatchRequest {
        ids: vec!["invalid-id-format".to_string()]
    };
    let delete_batch_request = tonic::Request::new(delete_batch_request_message);

    // This should fail with an invalid argument error
    let error = tonic_client.category().categories_delete_batch(delete_batch_request).await
        .expect_err("Expected delete_batch to fail with invalid ID");

    assert_eq!(error.code(), tonic::Code::InvalidArgument);
    assert!(error.message().contains("Invalid category ID format"));

    Ok(())
}

#[sqlx::test]
async fn delete_batch_returns_zero_for_only_nonexistent_categories(database_pool: sqlx::SqlitePool) -> Result<()> {
    //-- Setup and Fixtures (Arrange)
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to delete batch with only non-existent IDs
    let fake_ids = vec![
        domain::RowID::new().to_string(),
        domain::RowID::new().to_string(),
    ];

    let delete_batch_request_message = rpc::CategoriesDeleteBatchRequest {
        ids: fake_ids
    };
    let delete_batch_request = tonic::Request::new(delete_batch_request_message);
    let delete_batch_response = tonic_client.category().categories_delete_batch(delete_batch_request).await?;
    let delete_batch_response_message = delete_batch_response.into_inner();

    // Assert that the response indicates 0 rows were deleted
    assert_eq!(delete_batch_response_message.rows_deleted, 0);

    Ok(())
}