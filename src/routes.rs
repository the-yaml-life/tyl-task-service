//! Route definitions for the microservice
//!
//! This module organizes all HTTP routes and their corresponding handlers.

use axum::{
    http::StatusCode,
    routing::{delete, get, post, put},
    Router,
};

use crate::{
    handlers::{
        health::{health_check, readiness_check, liveness_check, health_detail},
        api::{process_request, get_entity, create_entity, update_entity, delete_entity},
    },
    AppState,
};

/// Create health check routes
/// 
/// These routes are typically used by load balancers, orchestrators, and monitoring systems.
pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
        .route("/health/detail", get(health_detail))
}

/// Create API routes for business logic
/// 
/// These routes implement the main business functionality of the microservice.
pub fn api_routes() -> Router<AppState> {
    Router::new()
        // Main business operation
        .route("/api/v1/process", post(process_request))
        
        // CRUD operations for entities
        .route("/api/v1/entities", post(create_entity))
        .route("/api/v1/entities/:id", get(get_entity))
        .route("/api/v1/entities/:id", put(update_entity))
        .route("/api/v1/entities/:id", delete(delete_entity))
}

/// Create the complete router with all routes
/// 
/// This function combines all route modules into a single router.
pub fn create_router() -> Router<AppState> {
    Router::new()
        .merge(health_routes())
        .merge(api_routes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use crate::{TaskServiceConfig, domain::MockTaskService, events::EventService};
    use std::sync::Arc;

    async fn create_test_app() -> Router {
        let state = AppState {
            config: Arc::new(TaskServiceConfig::default()),
            domain_service: Arc::new(MockTaskService::new()),
            event_service: Arc::new(EventService::new().await.unwrap()),
            logger: Arc::new(tyl_logging::loggers::console::ConsoleLogger::new()),
        };

        create_router().with_state(state)
    }

    #[tokio::test]
    async fn test_health_routes() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Test basic health check
        let response = server.get("/health").await;
        response.assert_status_ok();
        // Just check that the response is valid JSON with a status field
        let json: serde_json::Value = response.json();
        assert_eq!(json["status"], "healthy");

        // Test readiness check
        let response = server.get("/health/ready").await;
        response.assert_status_ok();

        // Test liveness check
        let response = server.get("/health/live").await;
        response.assert_status_ok();

        // Test detailed health check
        let response = server.get("/health/detail").await;
        response.assert_status_ok();
    }

    #[tokio::test]
    async fn test_api_routes() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Test process endpoint
        let response = server
            .post("/api/v1/process")
            .json(&serde_json::json!({
                "email": "test@example.com",
                "username": "testuser",
                "full_name": "Test Task",
                "password": "password123"
            }))
            .await;
        response.assert_status_ok();

        // Test create entity
        let response = server
            .post("/api/v1/entities")
            .json(&serde_json::json!({
                "email": "test@example.com",
                "username": "testuser",
                "full_name": "Test Task",
                "password": "password123"
            }))
            .await;
        response.assert_status_ok();

        // Test get entity (existing)
        let response = server.get("/api/v1/entities/test-id").await;
        response.assert_status_ok();

        // Test get entity (non-existent)
        let response = server.get("/api/v1/entities/non-existent").await;
        response.assert_status_not_found();

        // Test update entity
        let response = server
            .put("/api/v1/entities/test-id")
            .json(&serde_json::json!({
                "email": "updated@example.com",
                "username": "updateduser",
                "full_name": "Updated Task"
            }))
            .await;
        response.assert_status_ok();

        // Test delete entity
        let response = server.delete("/api/v1/entities/test-id").await;
        response.assert_status(StatusCode::NO_CONTENT);
    }
}