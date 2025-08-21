//! Task management API routes
//!
//! This module defines all HTTP routes for task management operations,
//! following REST conventions and providing comprehensive task management capabilities.

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

use crate::{handlers::tasks::*, AppState};

/// Create task management routes
pub fn create_task_routes() -> Router<AppState> {
    Router::new()
        // Task CRUD operations
        .route("/tasks", post(create_task))
        .route("/tasks", get(list_tasks))
        .route("/tasks/:id", get(get_task))
        .route("/tasks/:id", put(update_task))
        .route("/tasks/:id", delete(delete_task))
        
        // Task status management
        .route("/tasks/:id/status", patch(transition_task_status))
        
        // Task dependencies
        .route("/tasks/:id/dependencies", post(add_task_dependency))
        .route("/tasks/:id/dependencies", get(get_task_dependencies))
        
        // Task hierarchy (subtasks)
        .route("/tasks/:parent_id/subtasks/:child_id", post(add_subtask))
        .route("/tasks/:parent_id/subtasks", get(get_subtasks))
        
        // Task assignment
        .route("/tasks/:id/assign", post(assign_task))
        .route("/tasks/:id/unassign", delete(unassign_task))
        
        // User-specific task queries
        .route("/users/:user_id/tasks", get(get_assigned_tasks))
        .route("/users/:user_id/tasks/actionable", get(get_actionable_tasks))
        
        // Task analytics and insights
        .route("/tasks/:id/analytics", get(get_task_analytics))
        .route("/tasks/overdue", get(get_overdue_tasks))
        
        // Project task management (if projects are implemented)
        // .route("/projects/:id/tasks", get(get_project_tasks))
        // .route("/projects/:id/critical-path", get(get_project_critical_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Method, StatusCode};
    use axum_test::TestServer;
    use tower::ServiceExt;
    
    // Mock AppState for testing
    fn create_test_app_state() -> AppState {
        // In a real test, you would create a proper mock AppState
        // For now, this is a placeholder
        todo!("Create proper test AppState with mocked dependencies")
    }
    
    #[tokio::test]
    async fn test_task_routes_structure() {
        let app = create_task_routes().with_state(create_test_app_state());
        
        // Test that the router is created successfully
        // In a real test environment, you would:
        // 1. Create a test server with the routes
        // 2. Make HTTP requests to verify route handling
        // 3. Mock the dependencies (task service, event service)
        // 4. Verify proper responses and error handling
        
        // Example test structure (commented out as it needs proper mocking):
        /*
        let server = TestServer::new(app).unwrap();
        
        // Test GET /tasks
        let response = server
            .get("/tasks")
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);
        
        // Test POST /tasks
        let task_json = r#"{
            "name": "Test Task",
            "context": "work",
            "priority": "medium"
        }"#;
        let response = server
            .post("/tasks")
            .json(&serde_json::from_str::<serde_json::Value>(task_json).unwrap())
            .await;
        assert_eq!(response.status_code(), StatusCode::CREATED);
        */
    }
    
    #[test]
    fn test_route_paths() {
        // Test that we can create the router without panicking
        let _router = create_task_routes();
        
        // In a real implementation, you would test:
        // - All expected routes are registered
        // - Routes accept correct HTTP methods
        // - Route parameters are properly extracted
        // - Middleware is applied correctly
    }
}