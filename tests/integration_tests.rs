//! Integration tests for the TYL microservice
//!
//! These tests verify that different components of the microservice work together correctly.

use tyl_task_service:{
    TaskServiceConfig, create_app, 
    domain::{TaskService, MockTaskService, CreateTaskRequest, UpdateTaskRequest},
    adapters::{InMemoryTaskRepository, TaskRepository},
};
use axum_test::TestServer;
use std::sync::Arc;

/// Test the complete microservice application startup
#[tokio::test]
async fn test_microservice_startup() {
    let config = UserServiceConfig::default();
    let app = create_app(config).await;
    
    assert!(app.is_ok(), "Microservice should start successfully");
}

/// Test end-to-end API flow
#[tokio::test]
async fn test_end_to_end_api_flow() {
    let config = UserServiceConfig::default();
    let app = create_app(config).await.unwrap();
    let server = TestServer::new(app).unwrap();

    // Test health check
    let response = server.get("/health").await;
    response.assert_status_ok();
    
    // Test main processing endpoint
    let response = server
        .post("/api/v1/process")
        .json(&serde_json::json!({
            "name": "Integration Test",
            "description": "End-to-end test"
        }))
        .await;
    response.assert_status_ok();
    
    // Test entity creation
    let response = server
        .post("/api/v1/entities")
        .json(&serde_json::json!({
            "name": "Test Entity",
            "description": "Created via API"
        }))
        .await;
    response.assert_status_ok();
}

/// Test configuration integration
#[tokio::test]
async fn test_configuration_integration() {
    let mut config = UserServiceConfig::default();
    config.service_name = "integration-test-service".to_string();
    config.api.port = 3001; // Use different port to avoid conflicts
    
    let app = create_app(config).await.unwrap();
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/health").await;
    response.assert_status_ok();
    response.assert_json_contains(&serde_json::json!({
        "service": "integration-test-service"
    }));
}

/// Test domain service integration
#[tokio::test]
async fn test_domain_service_integration() {
    let service = MockDomainService::new();
    
    // Test process operation
    let request = RequestType {
        name: "Integration Test".to_string(),
        description: Some("Testing domain service".to_string()),
    };
    
    let result = service.process(request).await;
    assert!(result.is_ok());
    
    // Test CRUD operations
    let create_request = CreateRequest {
        name: "New Entity".to_string(),
        description: Some("Integration test entity".to_string()),
    };
    
    let entity = service.create(create_request).await.unwrap();
    assert_eq!(entity.name, "New Entity");
    
    // Test retrieval
    let retrieved = service.get_by_id("test-id").await.unwrap();
    assert!(retrieved.is_some());
    
    // Test update
    let update_request = UpdateRequest {
        name: Some("Updated Entity".to_string()),
        description: None,
        status: None,
    };
    
    let updated = service.update("test-id", update_request).await.unwrap();
    assert_eq!(updated.name, "Updated Entity");
    
    // Test deletion
    let result = service.delete("test-id").await;
    assert!(result.is_ok());
}

/// Test repository integration
#[tokio::test]
async fn test_repository_integration() {
    let repo = InMemoryDomainRepository::new();
    
    // Test finding existing entity
    let result = repo.find_by_id("test-id").await.unwrap();
    assert!(result.is_some());
    
    // Test finding non-existent entity
    let result = repo.find_by_id("non-existent").await.unwrap();
    assert!(result.is_none());
    
    // Test entity operations
    let entity = tyl_{microservice_name}::domain::DomainModel::new("Integration Test Entity");
    
    // Test save
    let save_result = repo.save(&entity).await;
    assert!(save_result.is_ok());
    
    // Test update
    let update_result = repo.update(&entity).await;
    assert!(update_result.is_ok());
    
    // Test deletion
    let delete_result = repo.delete(&entity.id).await;
    assert!(delete_result.is_ok());
}

/// Test error handling across components
#[tokio::test]
async fn test_error_handling_integration() {
    let config = UserServiceConfig::default();
    let app = create_app(config).await.unwrap();
    let server = TestServer::new(app).unwrap();
    
    // Test invalid request handling
    let response = server
        .post("/api/v1/process")
        .json(&serde_json::json!({
            "invalid_field": "value"
        }))
        .await;
    response.assert_status_unprocessable_entity();
    
    // Test non-existent entity
    let response = server.get("/api/v1/entities/non-existent-id").await;
    response.assert_status_not_found();
    
    // Test invalid endpoint
    let response = server.get("/api/v1/invalid").await;
    response.assert_status_not_found();
}

/// Test health checks integration
#[tokio::test]
async fn test_health_checks_integration() {
    let config = UserServiceConfig::default();
    let app = create_app(config).await.unwrap();
    let server = TestServer::new(app).unwrap();
    
    // Test basic health check
    let response = server.get("/health").await;
    response.assert_status_ok();
    response.assert_json_contains(&serde_json::json!({
        "status": "healthy"
    }));
    
    // Test readiness check
    let response = server.get("/health/ready").await;
    response.assert_status_ok();
    response.assert_json_contains(&serde_json::json!({
        "status": "ready"
    }));
    
    // Test liveness check
    let response = server.get("/health/live").await;
    response.assert_status_ok();
    response.assert_json_contains(&serde_json::json!({
        "status": "alive"
    }));
    
    // Test detailed health check
    let response = server.get("/health/detail").await;
    response.assert_status_ok();
    
    let json: serde_json::Value = response.json();
    assert!(json["dependencies"].is_object());
    assert!(json["dependencies"]["database"].is_string());
    assert!(json["dependencies"]["external_services"].is_string());
}

/// Test concurrent requests handling
#[tokio::test]
async fn test_concurrent_requests() {
    let config = UserServiceConfig::default();
    let app = create_app(config).await.unwrap();
    let server = TestServer::new(app).unwrap();
    
    // Create multiple concurrent requests
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let server_clone = server.clone();
        let handle = tokio::spawn(async move {
            let response = server_clone
                .post("/api/v1/process")
                .json(&serde_json::json!({
                    "name": format!("Concurrent Request {}", i),
                    "description": "Testing concurrency"
                }))
                .await;
            response.assert_status_ok();
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

/// Test serialization/deserialization integration
#[tokio::test] 
async fn test_serialization_integration() {
    use tyl_{microservice_name}::domain::{DomainModel, EntityStatus};
    
    let entity = DomainModel::new("Serialization Test");
    
    // Test JSON serialization
    let json = serde_json::to_string(&entity).unwrap();
    let deserialized: DomainModel = serde_json::from_str(&json).unwrap();
    
    assert_eq!(entity.id, deserialized.id);
    assert_eq!(entity.name, deserialized.name);
    assert_eq!(entity.status, deserialized.status);
    
    // Test with different status
    let mut entity_inactive = entity.clone();
    entity_inactive.status = EntityStatus::Inactive;
    
    let json = serde_json::to_string(&entity_inactive).unwrap();
    let deserialized: DomainModel = serde_json::from_str(&json).unwrap();
    
    assert_eq!(entity_inactive.status, deserialized.status);
    assert!(!deserialized.is_active());
}