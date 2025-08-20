//! API handlers for business logic endpoints
//!
//! Contains HTTP handlers for the main business operations of the microservice.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::{
    AppState, 
    domain::{CreateTaskRequest, TaskResponse, UpdateTaskRequest, Task},
    utils::generate_correlation_id,
};

/// API error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub correlation_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// API success response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub correlation_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            correlation_id: generate_correlation_id(),
            timestamp: chrono::Utc::now(),
        }
    }
}

impl ApiError {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            correlation_id: generate_correlation_id(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Main business operation endpoint
/// 
/// POST /api/v1/process
pub async fn process_request(
    State(state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
) -> Result<Json<ApiResponse<TaskResponse>>, (StatusCode, Json<ApiError>)> {
    let correlation_id = generate_correlation_id();
    
    info!(
        correlation_id = %correlation_id,
        request_email = %request.email,
        "Processing business request"
    );

    match state.domain_service.process(request).await {
        Ok(response) => {
            info!(
                correlation_id = %correlation_id,
                response_id = %response.id,
                "Request processed successfully"
            );
            
            Ok(Json(ApiResponse::new(response)))
        }
        Err(e) => {
            error!(
                correlation_id = %correlation_id,
                error = %e,
                "Failed to process request"
            );
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("PROCESSING_ERROR", e.to_string())),
            ))
        }
    }
}

/// Get entity by ID
/// 
/// GET /api/v1/entities/{id}
pub async fn get_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Task>>, (StatusCode, Json<ApiError>)> {
    let correlation_id = generate_correlation_id();
    
    info!(
        correlation_id = %correlation_id,
        entity_id = %id,
        "Getting entity by ID"
    );

    match state.domain_service.get_by_id(&id).await {
        Ok(Some(entity)) => {
            info!(
                correlation_id = %correlation_id,
                entity_id = %id,
                "Entity found"
            );
            
            Ok(Json(ApiResponse::new(entity)))
        }
        Ok(None) => {
            info!(
                correlation_id = %correlation_id,
                entity_id = %id,
                "Entity not found"
            );
            
            Err((
                StatusCode::NOT_FOUND,
                Json(ApiError::new("ENTITY_NOT_FOUND", format!("Entity with ID {} not found", id))),
            ))
        }
        Err(e) => {
            error!(
                correlation_id = %correlation_id,
                entity_id = %id,
                error = %e,
                "Failed to get entity"
            );
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// Create new entity
/// 
/// POST /api/v1/entities
pub async fn create_entity(
    State(state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
) -> Result<Json<ApiResponse<Task>>, (StatusCode, Json<ApiError>)> {
    let correlation_id = generate_correlation_id();
    
    info!(
        correlation_id = %correlation_id,
        entity_email = %request.email,
        "Creating new entity"
    );

    match state.domain_service.create(request).await {
        Ok(entity) => {
            info!(
                correlation_id = %correlation_id,
                entity_id = %entity.id,
                "Entity created successfully"
            );
            
            Ok(Json(ApiResponse::new(entity)))
        }
        Err(e) => {
            error!(
                correlation_id = %correlation_id,
                error = %e,
                "Failed to create entity"
            );
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("CREATION_ERROR", e.to_string())),
            ))
        }
    }
}

/// Update existing entity
/// 
/// PUT /api/v1/entities/{id}
pub async fn update_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateTaskRequest>,
) -> Result<Json<ApiResponse<Task>>, (StatusCode, Json<ApiError>)> {
    let correlation_id = generate_correlation_id();
    
    info!(
        correlation_id = %correlation_id,
        entity_id = %id,
        "Updating entity"
    );

    match state.domain_service.update(&id, request).await {
        Ok(entity) => {
            info!(
                correlation_id = %correlation_id,
                entity_id = %id,
                "Entity updated successfully"
            );
            
            Ok(Json(ApiResponse::new(entity)))
        }
        Err(e) => {
            error!(
                correlation_id = %correlation_id,
                entity_id = %id,
                error = %e,
                "Failed to update entity"
            );
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("UPDATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// Delete entity
/// 
/// DELETE /api/v1/entities/{id}
pub async fn delete_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let correlation_id = generate_correlation_id();
    
    info!(
        correlation_id = %correlation_id,
        entity_id = %id,
        "Deleting entity"
    );

    match state.domain_service.delete(&id).await {
        Ok(()) => {
            info!(
                correlation_id = %correlation_id,
                entity_id = %id,
                "Entity deleted successfully"
            );
            
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!(
                correlation_id = %correlation_id,
                entity_id = %id,
                error = %e,
                "Failed to delete entity"
            );
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("DELETION_ERROR", e.to_string())),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TaskServiceConfig, domain::MockTaskService, events::EventService};
    use std::sync::Arc;

    async fn create_test_state() -> AppState {
        AppState {
            config: Arc::new(TaskServiceConfig::default()),
            domain_service: Arc::new(MockTaskService::new()),
            event_service: Arc::new(EventService::new().await.unwrap()),
            logger: Arc::new(tyl_logging::loggers::console::ConsoleLogger::new()),
        }
    }

    #[tokio::test]
    async fn test_process_request() {
        let state = create_test_state().await;
        let request = CreateTaskRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            full_name: Some("Test Task".to_string()),
            password: "password123".to_string(),
        };
        
        let result = process_request(State(state), Json(request)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.data.message.contains("Processed"));
    }

    #[tokio::test]
    async fn test_get_entity_found() {
        let state = create_test_state().await;
        let result = get_entity(State(state), Path("test-id".to_string())).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.username, "testuser");
    }

    #[tokio::test]
    async fn test_get_entity_not_found() {
        let state = create_test_state().await;
        let result = get_entity(State(state), Path("non-existent".to_string())).await;
        
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_entity() {
        let state = create_test_state().await;
        let request = CreateTaskRequest {
            email: "new@example.com".to_string(),
            username: "newuser".to_string(),
            full_name: Some("New Task".to_string()),
            password: "password123".to_string(),
        };
        
        let result = create_entity(State(state), Json(request)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.data.username, "newuser");
    }

    #[tokio::test]
    async fn test_update_entity() {
        let state = create_test_state().await;
        let request = UpdateTaskRequest {
            email: Some("updated@example.com".to_string()),
            username: Some("updateduser".to_string()),
            full_name: Some("Updated Task".to_string()),
            status: None,
        };
        
        let result = update_entity(State(state), Path("test-id".to_string()), Json(request)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_entity() {
        let state = create_test_state().await;
        let result = delete_entity(State(state), Path("test-id".to_string())).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    }
}