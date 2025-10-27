use personal_ledger_backend::rpc;
use personal_ledger_backend::domain;

use crate::{categories, helpers};

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn get_returns_existing_category(database_pool: sqlx::SqlitePool) -> Result<()> {
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

    // Now get the category by ID
    let get_request_message = rpc::CategoryGetRequest {
        id: created_category.id.clone()
    };
    let get_request = tonic::Request::new(get_request_message);
    let get_response = tonic_client.category().category_get(get_request).await?;
    let get_response_message = get_response.into_inner();

    // Assert that the response contains the expected category
    assert!(get_response_message.category.is_some());
    let retrieved_category = get_response_message.category.unwrap();

    // Verify that the retrieved category matches the created one
    assert_eq!(retrieved_category.id, created_category.id);
    assert_eq!(retrieved_category.code, created_category.code);
    assert_eq!(retrieved_category.name, created_category.name);
    assert_eq!(retrieved_category.description, created_category.description);
    assert_eq!(retrieved_category.url_slug, created_category.url_slug);
    assert_eq!(retrieved_category.category_type, created_category.category_type);
    assert_eq!(retrieved_category.color, created_category.color);
    assert_eq!(retrieved_category.icon, created_category.icon);
    assert_eq!(retrieved_category.is_active, created_category.is_active);

    // Verify timestamps are present
    assert!(retrieved_category.created_on.is_some());
    assert!(retrieved_category.updated_on.is_some());

    Ok(())
}

#[sqlx::test]
async fn get_fails_with_nonexistent_id(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to get a category with a non-existent ID
    let nonexistent_id = domain::RowID::new().to_string();
    let get_request_message = rpc::CategoryGetRequest {
        id: nonexistent_id.clone()
    };
    let get_request = tonic::Request::new(get_request_message);
    let result = tonic_client.category().category_get(get_request).await;

    // Should fail with NotFound
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);
    assert!(status.message().contains("not found"));

    Ok(())
}

#[sqlx::test]
async fn get_fails_with_invalid_id_format(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to get a category with an invalid ID format
    let get_request_message = rpc::CategoryGetRequest {
        id: "invalid-uuid".to_string()
    };
    let get_request = tonic::Request::new(get_request_message);
    let result = tonic_client.category().category_get(get_request).await;

    // Should fail with InvalidArgument
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(status.message().contains("Invalid category ID format"));

    Ok(())
}

#[sqlx::test]
async fn get_by_code_returns_existing_category(database_pool: sqlx::SqlitePool) -> Result<()> {
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

    // Now get the category by code
    let get_request_message = rpc::CategoryGetByCodeRequest {
        code: created_category.code.clone()
    };
    let get_request = tonic::Request::new(get_request_message);
    let get_response = tonic_client.category().category_get_by_code(get_request).await?;
    let get_response_message = get_response.into_inner();

    // Assert that the response contains the expected category
    assert!(get_response_message.category.is_some());
    let retrieved_category = get_response_message.category.unwrap();

    // Verify that the retrieved category matches the created one
    assert_eq!(retrieved_category.id, created_category.id);
    assert_eq!(retrieved_category.code, created_category.code);
    assert_eq!(retrieved_category.name, created_category.name);

    Ok(())
}

#[sqlx::test]
async fn get_by_code_fails_with_nonexistent_code(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to get a category with a non-existent code
    let get_request_message = rpc::CategoryGetByCodeRequest {
        code: "NONEXISTENT".to_string()
    };
    let get_request = tonic::Request::new(get_request_message);
    let result = tonic_client.category().category_get_by_code(get_request).await;

    // Should fail with NotFound
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);
    assert!(status.message().contains("not found"));

    Ok(())
}

#[sqlx::test]
async fn get_by_code_fails_with_empty_code(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to get a category with an empty code
    let get_request_message = rpc::CategoryGetByCodeRequest {
        code: "".to_string()
    };
    let get_request = tonic::Request::new(get_request_message);
    let result = tonic_client.category().category_get_by_code(get_request).await;

    // Should fail with InvalidArgument
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(status.message().contains("cannot be empty"));

    Ok(())
}

#[sqlx::test]
async fn get_by_slug_returns_existing_category(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Create a category with a URL slug
    let mut rpc_category = categories::mock_rpc_category();
    rpc_category.url_slug = Some("test-category-slug".to_string());

    let create_request_message = rpc::CategoryCreateRequest {
        category: Some(rpc_category.clone())
    };
    let create_request = tonic::Request::new(create_request_message);
    let create_response = tonic_client.category().category_create(create_request).await?;
    let created_category = create_response.into_inner().category.unwrap();

    // Now get the category by slug
    let get_request_message = rpc::CategoryGetBySlugRequest {
        url_slug: created_category.url_slug.clone().unwrap()
    };
    let get_request = tonic::Request::new(get_request_message);
    let get_response = tonic_client.category().category_get_by_slug(get_request).await?;
    let get_response_message = get_response.into_inner();

    // Assert that the response contains the expected category
    assert!(get_response_message.category.is_some());
    let retrieved_category = get_response_message.category.unwrap();

    // Verify that the retrieved category matches the created one
    assert_eq!(retrieved_category.id, created_category.id);
    assert_eq!(retrieved_category.url_slug, created_category.url_slug);

    Ok(())
}

#[sqlx::test]
async fn get_by_slug_fails_with_nonexistent_slug(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to get a category with a non-existent slug
    let get_request_message = rpc::CategoryGetBySlugRequest {
        url_slug: "nonexistent-slug".to_string()
    };
    let get_request = tonic::Request::new(get_request_message);
    let result = tonic_client.category().category_get_by_slug(get_request).await;

    // Should fail with NotFound
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);
    assert!(status.message().contains("not found"));

    Ok(())
}

#[sqlx::test]
async fn get_by_slug_fails_with_empty_slug(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to get a category with an empty slug
    let get_request_message = rpc::CategoryGetBySlugRequest {
        url_slug: "".to_string()
    };
    let get_request = tonic::Request::new(get_request_message);
    let result = tonic_client.category().category_get_by_slug(get_request).await;

    // Should fail with InvalidArgument
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(status.message().contains("cannot be empty"));

    Ok(())
}

#[sqlx::test]
async fn get_by_slug_fails_with_invalid_slug_format(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Try to get a category with an invalid slug format
    let get_request_message = rpc::CategoryGetBySlugRequest {
        url_slug: "!@#$%^&*()".to_string()
    };
    let get_request = tonic::Request::new(get_request_message);
    let result = tonic_client.category().category_get_by_slug(get_request).await;

    // Should fail with InvalidArgument
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(status.message().contains("Invalid URL slug format"));

    Ok(())
}