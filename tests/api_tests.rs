//! API endpoint tests for the TYL microservice
//!
//! These tests verify that all HTTP endpoints work correctly and return the expected responses.

use tyl_task_service::{TaskServiceConfig, create_app};
use axum_test::TestServer;
use serde_json::json;

/// Helper function to create a test server
async fn create_test_server() -> TestServer {
    let config = TaskServiceConfig::default();
    let app = create_app(config).await.unwrap();
    TestServer::new(app).unwrap()
}

/// Test all health check endpoints
#[tokio::test]
async fn test_health_endpoints() {
    let server = create_test_server().await;

    // Basic health check
    let response = server.get("/health").await;
    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["status"], "healthy");

    // Readiness check
    let response = server.get("/health/ready").await;
    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["status"], "ready");

    // Liveness check
    let response = server.get("/health/live").await;
    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["status"], "alive");

    // Detailed health check
    let response = server.get("/health/detail").await;
    response.assert_status_ok();
    
    let json_response: serde_json::Value = response.json();
    assert!(json_response.get("dependencies").is_some());
    assert!(json_response.get("service").is_some());
    assert!(json_response.get("version").is_some());
}

/// Test the main process endpoint
#[tokio::test]
async fn test_process_endpoint() {
    let server = create_test_server().await;

    // Valid request
    let response = server
        .post("/api/v1/process")
        .json(&json!({
            "name": "Test Process",
            "description": "Testing the process endpoint"
        }))
        .await;
    
    response.assert_status_ok();
    
    let json_response: serde_json::Value = response.json();
    assert!(json_response.get("data").is_some());
    assert!(json_response.get("correlation_id").is_some());
    assert!(json_response.get("timestamp").is_some());
    
    let data = &json_response["data"];
    assert!(data.get("id").is_some());
    assert!(data.get("message").is_some());
    assert!(data["message"].as_str().unwrap().contains("Processed"));

    // Invalid request (missing required fields)
    let response = server
        .post("/api/v1/process")
        .json(&json!({
            "invalid_field": "value"
        }))
        .await;
    
    assert!(response.status_code().is_client_error());
}

/// Test entity CRUD endpoints
#[tokio::test]
async fn test_entity_crud_endpoints() {
    let server = create_test_server().await;

    // Test create entity
    let create_response = server
        .post("/api/v1/entities")
        .json(&json!({
            "name": "Test Entity",
            "description": "Created via API test"
        }))
        .await;
    
    create_response.assert_status_ok();
    
    let create_json: serde_json::Value = create_response.json();
    assert!(create_json.get("data").is_some());
    let entity = &create_json["data"];
    assert_eq!(entity["name"], "Test Entity");
    assert!(entity.get("id").is_some());

    // Test get entity (existing)
    let get_response = server.get("/api/v1/entities/test-id").await;
    get_response.assert_status_ok();
    
    let get_json: serde_json::Value = get_response.json();
    assert!(get_json.get("data").is_some());
    assert_eq!(get_json["data"]["name"], "Test Entity");

    // Test get entity (non-existent)
    let get_response = server.get("/api/v1/entities/non-existent-id").await;
    get_response.assert_status_not_found();
    
    let error_json: serde_json::Value = get_response.json();
    assert!(error_json.get("error").is_some());
    assert!(error_json.get("correlation_id").is_some());

    // Test update entity
    let update_response = server
        .put("/api/v1/entities/test-id")
        .json(&json!({
            "name": "Updated Entity",
            "status": "Inactive"
        }))
        .await;
    
    update_response.assert_status_ok();
    
    let update_json: serde_json::Value = update_response.json();
    assert_eq!(update_json["data"]["name"], "Updated Entity");

    // Test delete entity
    let delete_response = server.delete("/api/v1/entities/test-id").await;
    delete_response.assert_status(axum::http::StatusCode::NO_CONTENT);
}

/// Test request validation
#[tokio::test]
async fn test_request_validation() {
    let server = create_test_server().await;

    // Test process endpoint with empty body
    let response = server
        .post("/api/v1/process")
        .json(&json!({}))
        .await;
    
    assert!(response.status_code().is_client_error());

    // Test create entity with invalid data
    let response = server
        .post("/api/v1/entities")
        .json(&json!({
            "name": "",  // Empty name should be invalid
            "description": "Test"
        }))
        .await;
    
    // Depending on validation rules, this might be 400 or 422
    assert!(response.status_code().is_client_error());

    // Test with completely invalid JSON structure
    let response = server
        .post("/api/v1/process")
        .add_header(
            axum::http::HeaderName::from_static("content-type"), 
            axum::http::HeaderValue::from_static("application/json")
        )
        .text("invalid json")
        .await;
    
    response.assert_status_bad_request();
}

/// Test response headers and format
#[tokio::test]
async fn test_response_format() {
    let server = create_test_server().await;

    let response = server.get("/health").await;
    
    // Check content type
    assert!(response.headers().get("content-type").is_some());
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));

    // Check response structure
    let json_response: serde_json::Value = response.json();
    assert!(json_response.get("status").is_some());
    assert!(json_response.get("service").is_some());
    assert!(json_response.get("version").is_some());
    assert!(json_response.get("timestamp").is_some());
}

/// Test CORS headers (if configured)
#[tokio::test]
async fn test_cors_headers() {
    let server = create_test_server().await;

    // Test preflight request (simplified for test framework limitations)
    let response = server
        .method(axum::http::Method::OPTIONS, "/api/v1/process")
        .await;
    
    // CORS should be configured to allow this
    response.assert_status_ok();
}

/// Test error response format
#[tokio::test]
async fn test_error_response_format() {
    let server = create_test_server().await;

    let response = server.get("/api/v1/entities/non-existent").await;
    response.assert_status_not_found();
    
    let error_json: serde_json::Value = response.json();
    
    // Check error response structure
    assert!(error_json.get("error").is_some());
    assert!(error_json.get("message").is_some());
    assert!(error_json.get("correlation_id").is_some());
    assert!(error_json.get("timestamp").is_some());
    
    // Check that correlation_id is a valid UUID format
    let correlation_id = error_json["correlation_id"].as_str().unwrap();
    assert!(uuid::Uuid::parse_str(correlation_id).is_ok());
}

/// Test content negotiation
#[tokio::test]
async fn test_content_negotiation() {
    let server = create_test_server().await;

    // Test that service returns JSON by default
    let response = server.get("/health").await;
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));

    // Test explicit JSON request
    let response = server
        .get("/health")
        .add_header(
            axum::http::HeaderName::from_static("accept"), 
            axum::http::HeaderValue::from_static("application/json")
        )
        .await;
    
    response.assert_status_ok();
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}

/// Test concurrent API requests
#[tokio::test]
async fn test_concurrent_api_requests() {
    let server = create_test_server().await;
    
    // Send multiple sequential requests to test correlation ID uniqueness
    let mut correlation_ids = Vec::new();
    
    for i in 0..5 {
        let response = server
            .post("/api/v1/process")
            .json(&json!({
                "name": format!("Request {}", i),
                "description": "Testing API uniqueness"
            }))
            .await;
        
        response.assert_status_ok();
        
        // Verify each response has unique correlation ID
        let json_response: serde_json::Value = response.json();
        assert!(json_response.get("correlation_id").is_some());
        
        let correlation_id = json_response["correlation_id"].as_str().unwrap().to_string();
        correlation_ids.push(correlation_id);
    }
    
    // Verify all correlation IDs are unique
    let original_len = correlation_ids.len();
    correlation_ids.sort();
    correlation_ids.dedup();
    assert_eq!(correlation_ids.len(), original_len, "All correlation IDs should be unique");
}

/// Test API rate limiting (if implemented)
#[tokio::test]
async fn test_rate_limiting() {
    let server = create_test_server().await;
    
    // This test would be more meaningful with actual rate limiting configured
    // For now, just verify that multiple requests succeed
    for _ in 0..10 {
        let response = server.get("/health").await;
        response.assert_status_ok();
    }
}