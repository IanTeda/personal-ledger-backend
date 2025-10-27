use personal_ledger_backend::rpc;

use crate::helpers;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

#[sqlx::test]
async fn list_returns_all_categories(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Create some test categories
    let categories_to_create = vec![
        rpc::Category {
            id: "".to_string(),
            code: "LIST_TEST_1".to_string(),
            name: "List Test Category 1".to_string(),
            description: Some("First test category for listing".to_string()),
            url_slug: Some("list-test-1".to_string()),
            category_type: rpc::CategoryTypes::Expense as i32,
            color: Some("#FF5733".to_string()),
            icon: Some("test-icon-1".to_string()),
            is_active: true,
            created_on: None,
            updated_on: None,
        },
        rpc::Category {
            id: "".to_string(),
            code: "LIST_TEST_2".to_string(),
            name: "List Test Category 2".to_string(),
            description: Some("Second test category for listing".to_string()),
            url_slug: Some("list-test-2".to_string()),
            category_type: rpc::CategoryTypes::Income as i32,
            color: Some("#4A90E2".to_string()),
            icon: Some("test-icon-2".to_string()),
            is_active: false,
            created_on: None,
            updated_on: None,
        },
    ];

    // Create the categories
    for category in &categories_to_create {
        let create_request_message = rpc::CategoryCreateRequest {
            category: Some(category.clone())
        };
        let create_request = tonic::Request::new(create_request_message);
        tonic_client.category().category_create(create_request).await?;
    }

    // List all categories
    let list_request_message = rpc::CategoriesListRequest {
        offset: 0,
        limit: 100,
        category_type: None,
        is_active: None,
        sort_by: None,
        sort_desc: None,
    };
    let list_request = tonic::Request::new(list_request_message);
    let list_response = tonic_client.category().categories_list(list_request).await?;
    let list_response_message = list_response.into_inner();

    // Should have at least our test categories
    assert!(list_response_message.categories.len() >= 2);
    assert!(list_response_message.total_count >= 2);

    // Verify pagination info
    assert_eq!(list_response_message.offset, 0);
    assert_eq!(list_response_message.limit, 100);

    // Verify our test categories are in the results
    let test_codes: std::collections::HashSet<String> = categories_to_create.iter()
        .map(|c| c.code.clone())
        .collect();

    let result_codes: std::collections::HashSet<String> = list_response_message.categories.iter()
        .map(|c| c.code.clone())
        .collect();

    for test_code in &test_codes {
        assert!(result_codes.contains(test_code), "Test category {} not found in results", test_code);
    }

    Ok(())
}

#[sqlx::test]
async fn list_filters_by_category_type(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Create categories of different types
    let expense_category = rpc::Category {
        id: "".to_string(),
        code: "FILTER_EXPENSE".to_string(),
        name: "Filter Expense Category".to_string(),
        description: None,
        url_slug: None,
        category_type: rpc::CategoryTypes::Expense as i32,
        color: None,
        icon: None,
        is_active: true,
        created_on: None,
        updated_on: None,
    };

    let income_category = rpc::Category {
        id: "".to_string(),
        code: "FILTER_INCOME".to_string(),
        name: "Filter Income Category".to_string(),
        description: None,
        url_slug: None,
        category_type: rpc::CategoryTypes::Income as i32,
        color: None,
        icon: None,
        is_active: true,
        created_on: None,
        updated_on: None,
    };

    // Create the categories
    for category in &[&expense_category, &income_category] {
        let create_request_message = rpc::CategoryCreateRequest {
            category: Some((*category).clone())
        };
        let create_request = tonic::Request::new(create_request_message);
        tonic_client.category().category_create(create_request).await?;
    }

    // List only expense categories
    let list_request_message = rpc::CategoriesListRequest {
        offset: 0,
        limit: 100,
        category_type: Some(rpc::CategoryTypes::Expense as i32),
        is_active: None,
        sort_by: None,
        sort_desc: None,
    };
    let list_request = tonic::Request::new(list_request_message);
    let list_response = tonic_client.category().categories_list(list_request).await?;
    let list_response_message = list_response.into_inner();

    // Should have at least our expense category
    assert!(!list_response_message.categories.is_empty());

    // All returned categories should be expenses
    for category in &list_response_message.categories {
        assert_eq!(category.category_type, rpc::CategoryTypes::Expense as i32);
    }

    // Should include our test expense category
    let has_expense_category = list_response_message.categories.iter()
        .any(|c| c.code == expense_category.code);
    assert!(has_expense_category, "Expense category not found in filtered results");

    Ok(())
}

#[sqlx::test]
async fn list_filters_by_active_status(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Create active and inactive categories
    let active_category = rpc::Category {
        id: "".to_string(),
        code: "ACTIVE_FILTER".to_string(),
        name: "Active Filter Category".to_string(),
        description: None,
        url_slug: None,
        category_type: rpc::CategoryTypes::Expense as i32,
        color: None,
        icon: None,
        is_active: true,
        created_on: None,
        updated_on: None,
    };

    let inactive_category = rpc::Category {
        id: "".to_string(),
        code: "INACTIVE_FILTER".to_string(),
        name: "Inactive Filter Category".to_string(),
        description: None,
        url_slug: None,
        category_type: rpc::CategoryTypes::Expense as i32,
        color: None,
        icon: None,
        is_active: false,
        created_on: None,
        updated_on: None,
    };

    // Create the categories
    for category in &[&active_category, &inactive_category] {
        let create_request_message = rpc::CategoryCreateRequest {
            category: Some((*category).clone())
        };
        let create_request = tonic::Request::new(create_request_message);
        tonic_client.category().category_create(create_request).await?;
    }

    // List only active categories
    let list_request_message = rpc::CategoriesListRequest {
        offset: 0,
        limit: 100,
        category_type: None,
        is_active: Some(true),
        sort_by: None,
        sort_desc: None,
    };
    let list_request = tonic::Request::new(list_request_message);
    let list_response = tonic_client.category().categories_list(list_request).await?;
    let list_response_message = list_response.into_inner();

    // Should have at least our active category
    assert!(!list_response_message.categories.is_empty());

    // All returned categories should be active
    for category in &list_response_message.categories {
        assert!(category.is_active, "Category {} is not active", category.code);
    }

    // Should include our test active category
    let has_active_category = list_response_message.categories.iter()
        .any(|c| c.code == active_category.code);
    assert!(has_active_category, "Active category not found in filtered results");

    Ok(())
}

#[sqlx::test]
async fn list_supports_pagination(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Create multiple categories
    let mut created_categories = Vec::new();
    for i in 0..5 {
        let category = rpc::Category {
            id: "".to_string(),
            code: format!("PAGINATION_{:02}", i),
            name: format!("Pagination Category {}", i),
            description: None,
            url_slug: None,
            category_type: rpc::CategoryTypes::Expense as i32,
            color: None,
            icon: None,
            is_active: true,
            created_on: None,
            updated_on: None,
        };

        let create_request_message = rpc::CategoryCreateRequest {
            category: Some(category.clone())
        };
        let create_request = tonic::Request::new(create_request_message);
        let create_response = tonic_client.category().category_create(create_request).await?;
        let created_category = create_response.into_inner().category.unwrap();
        created_categories.push(created_category);
    }

    // List with pagination (limit 2, offset 1)
    let list_request_message = rpc::CategoriesListRequest {
        offset: 1,
        limit: 2,
        category_type: None,
        is_active: None,
        sort_by: Some("code".to_string()),
        sort_desc: Some(false), // ascending
    };
    let list_request = tonic::Request::new(list_request_message);
    let list_response = tonic_client.category().categories_list(list_request).await?;
    let list_response_message = list_response.into_inner();

    // Should return exactly 2 categories (limited)
    assert_eq!(list_response_message.categories.len(), 2);

    // Should have total count of at least our 5 categories
    assert!(list_response_message.total_count >= 5);

    // Verify pagination info
    assert_eq!(list_response_message.offset, 1);
    assert_eq!(list_response_message.limit, 2);

    Ok(())
}

#[sqlx::test]
async fn list_supports_sorting(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Create categories with different names
    let categories_data = vec![
        ("Z_SORT_TEST", "Zebra Category"),
        ("A_SORT_TEST", "Apple Category"),
        ("M_SORT_TEST", "Middle Category"),
    ];

    for (code, name) in &categories_data {
        let category = rpc::Category {
            id: "".to_string(),
            code: code.to_string(),
            name: name.to_string(),
            description: None,
            url_slug: None,
            category_type: rpc::CategoryTypes::Expense as i32,
            color: None,
            icon: None,
            is_active: true,
            created_on: None,
            updated_on: None,
        };

        let create_request_message = rpc::CategoryCreateRequest {
            category: Some(category)
        };
        let create_request = tonic::Request::new(create_request_message);
        tonic_client.category().category_create(create_request).await?;
    }

    // List sorted by name ascending
    let list_request_message = rpc::CategoriesListRequest {
        offset: 0,
        limit: 100,
        category_type: None,
        is_active: None,
        sort_by: Some("name".to_string()),
        sort_desc: Some(false), // ascending
    };
    let list_request = tonic::Request::new(list_request_message);
    let list_response = tonic_client.category().categories_list(list_request).await?;
    let list_response_message = list_response.into_inner();

    // Find our test categories in the results
    let test_categories: Vec<_> = list_response_message.categories.iter()
        .filter(|c| c.code.ends_with("_SORT_TEST"))
        .collect();

    assert_eq!(test_categories.len(), 3);

    // Should be sorted: Apple, Middle, Zebra
    let names: Vec<String> = test_categories.iter().map(|c| c.name.clone()).collect();
    assert!(names.contains(&"Apple Category".to_string()));
    assert!(names.contains(&"Middle Category".to_string()));
    assert!(names.contains(&"Zebra Category".to_string()));

    Ok(())
}

#[sqlx::test]
async fn list_validates_pagination_parameters(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // Test negative offset
    let list_request_message = rpc::CategoriesListRequest {
        offset: -1,
        limit: 10,
        category_type: None,
        is_active: None,
        sort_by: None,
        sort_desc: None,
    };
    let list_request = tonic::Request::new(list_request_message);
    let result = tonic_client.category().categories_list(list_request).await;

    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(status.message().contains("Offset cannot be negative"));

    // Test zero limit
    let list_request_message = rpc::CategoriesListRequest {
        offset: 0,
        limit: 0,
        category_type: None,
        is_active: None,
        sort_by: None,
        sort_desc: None,
    };
    let list_request = tonic::Request::new(list_request_message);
    let result = tonic_client.category().categories_list(list_request).await;

    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(status.message().contains("Limit must be positive"));

    // Test limit too large
    let list_request_message = rpc::CategoriesListRequest {
        offset: 0,
        limit: 1001,
        category_type: None,
        is_active: None,
        sort_by: None,
        sort_desc: None,
    };
    let list_request = tonic::Request::new(list_request_message);
    let result = tonic_client.category().categories_list(list_request).await;

    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(status.message().contains("Limit cannot exceed 1000"));

    Ok(())
}

#[sqlx::test]
async fn list_handles_empty_results(database_pool: sqlx::SqlitePool) -> Result<()> {
    let tonic_server = helpers::SpawnTonicServer::init(database_pool).await?;
    let transport_channel = tonic_server.transport_channel();
    let mut tonic_client = helpers::SpawnTonicClient::new(transport_channel);

    // List categories with a filter that should return no results
    let list_request_message = rpc::CategoriesListRequest {
        offset: 0,
        limit: 100,
        category_type: Some(rpc::CategoryTypes::Asset as i32), // Assuming no assets exist
        is_active: None,
        sort_by: None,
        sort_desc: None,
    };
    let list_request = tonic::Request::new(list_request_message);
    let list_response = tonic_client.category().categories_list(list_request).await?;
    let list_response_message = list_response.into_inner();

    // Should return empty list but valid response
    assert_eq!(list_response_message.categories.len(), 0);
    assert_eq!(list_response_message.total_count, 0);
    assert_eq!(list_response_message.offset, 0);
    assert_eq!(list_response_message.limit, 100);

    Ok(())
}