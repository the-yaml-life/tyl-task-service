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
        tasks::{
            create_task, get_task, update_task, delete_task, list_tasks,
            transition_task_status, add_task_dependency, get_task_dependencies,
            assign_task, get_assigned_tasks, get_actionable_tasks, get_overdue_tasks,
            get_task_analytics, add_subtask, get_subtasks, get_circular_dependencies
        },
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

/// Create API routes for task management
/// 
/// These routes implement the complete task management functionality.
pub fn api_routes() -> Router<AppState> {
    Router::new()
        // Core task CRUD operations
        .route("/api/v1/tasks", post(create_task))
        .route("/api/v1/tasks", get(list_tasks))
        .route("/api/v1/tasks/:id", get(get_task))
        .route("/api/v1/tasks/:id", put(update_task))
        .route("/api/v1/tasks/:id", delete(delete_task))
        
        // Task status management
        .route("/api/v1/tasks/:id/status", post(transition_task_status))
        
        // Task dependencies
        .route("/api/v1/tasks/:id/dependencies", post(add_task_dependency))
        .route("/api/v1/tasks/:id/dependencies", get(get_task_dependencies))
        
        // Task hierarchy (subtasks)
        .route("/api/v1/tasks/:parent_id/subtasks/:child_id", post(add_subtask))
        .route("/api/v1/tasks/:parent_id/subtasks", get(get_subtasks))
        
        // Task assignment
        .route("/api/v1/tasks/:id/assign", post(assign_task))
        
        // Task queries and analytics
        .route("/api/v1/tasks/:id/analytics", get(get_task_analytics))
        .route("/api/v1/users/:user_id/tasks", get(get_assigned_tasks))
        .route("/api/v1/users/:user_id/tasks/actionable", get(get_actionable_tasks))
        .route("/api/v1/tasks/overdue", get(get_overdue_tasks))
        .route("/api/v1/tasks/circular-dependencies", get(get_circular_dependencies))
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
            tracer: Arc::new(tyl_tracing::SimpleTracer::new(tyl_tracing::TraceConfig::new("test-service"))),
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

        // Test create task
        let create_response = server
            .post("/api/v1/tasks")
            .json(&serde_json::json!({
                "name": "Test Task",
                "description": "A test task",
                "context": "work",
                "priority": "medium",
                "complexity": "simple"
            }))
            .await;
        create_response.assert_status_ok();
        
        // Extract the created task ID from the response
        let create_json: serde_json::Value = create_response.json();
        
        // Try different possible JSON paths for the task ID
        let task_id = create_json["id"].as_str()
            .or_else(|| create_json["data"]["id"].as_str())
            .expect(&format!("Could not find task ID in response: {}", create_json));

        // Test list tasks
        let response = server.get("/api/v1/tasks").await;
        response.assert_status_ok();

        // Test get task (existing)
        let response = server.get(&format!("/api/v1/tasks/{}", task_id)).await;
        response.assert_status_ok();

        // Test get task (non-existent)
        let response = server.get("/api/v1/tasks/non-existent").await;
        response.assert_status_not_found();

        // Test update task
        let response = server
            .put(&format!("/api/v1/tasks/{}", task_id))
            .json(&serde_json::json!({
                "name": "Updated Test Task",
                "description": "Updated description"
            }))
            .await;
        response.assert_status_ok();

        // Test delete task
        let response = server.delete(&format!("/api/v1/tasks/{}", task_id)).await;
        response.assert_status(StatusCode::NO_CONTENT);
    }
}