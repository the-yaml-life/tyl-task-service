//! Domain layer - Core business logic and models
//!
//! This module contains the core business logic, domain models, and service interfaces.
//! It is independent of external concerns like databases, HTTP, or other infrastructure.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::TaskServiceResult;

/// Main task service trait - Task management business logic interface
#[async_trait]
pub trait TaskService {
    /// Process a task operation
    async fn process(&self, request: CreateTaskRequest) -> TaskServiceResult<TaskResponse>;
    
    /// Get task by ID
    async fn get_by_id(&self, id: &str) -> TaskServiceResult<Option<Task>>;
    
    /// Create new task
    async fn create(&self, data: CreateTaskRequest) -> TaskServiceResult<Task>;
    
    /// Update existing task
    async fn update(&self, id: &str, data: UpdateTaskRequest) -> TaskServiceResult<Task>;
    
    /// Delete task
    async fn delete(&self, id: &str) -> TaskServiceResult<()>;
}

/// Task domain model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: String,
    pub email: String,
    pub username: String,
    pub full_name: Option<String>,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Task status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Active,
    Inactive,
    Pending,
    Suspended,
}

/// Request for creating a new task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub email: String,
    pub username: String,
    pub full_name: Option<String>,
    pub password: String,
}

/// Response type for task operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Request for updating existing task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub status: Option<TaskStatus>,
}

impl Task {
    /// Create a new task instance
    pub fn new(email: impl Into<String>, username: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            email: email.into(),
            username: username.into(),
            full_name: None,
            status: TaskStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Update the task with new data
    pub fn update(&mut self, request: UpdateTaskRequest) {
        if let Some(email) = request.email {
            self.email = email;
        }
        
        if let Some(username) = request.username {
            self.username = username;
        }
        
        if let Some(full_name) = request.full_name {
            self.full_name = Some(full_name);
        }
        
        if let Some(status) = request.status {
            self.status = status;
        }
        
        self.updated_at = chrono::Utc::now();
    }
    
    /// Check if the task is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, TaskStatus::Active)
    }
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Active
    }
}

/// Mock implementation for template purposes
/// Replace this with your actual task service implementation
pub struct MockTaskService {
    // In a real implementation, this would contain repositories, external service clients, etc.
}

impl MockTaskService {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl TaskService for MockTaskService {
    async fn process(&self, request: CreateTaskRequest) -> TaskServiceResult<TaskResponse> {
        // Mock business logic - replace with your actual implementation
        Ok(TaskResponse {
            id: Uuid::new_v4().to_string(),
            message: format!("Processed task: {}", request.username),
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn get_by_id(&self, id: &str) -> TaskServiceResult<Option<Task>> {
        // Mock implementation - replace with actual data access
        if id == "test-id" {
            Ok(Some(Task::new("test@example.com", "testuser")))
        } else {
            Ok(None)
        }
    }
    
    async fn create(&self, data: CreateTaskRequest) -> TaskServiceResult<Task> {
        // Mock implementation - replace with actual creation logic
        let mut task = Task::new(data.email, data.username);
        task.full_name = data.full_name;
        Ok(task)
    }
    
    async fn update(&self, id: &str, data: UpdateTaskRequest) -> TaskServiceResult<Task> {
        // Mock implementation - replace with actual update logic
        let mut task = Task::new("updated@example.com", "updateduser");
        task.id = id.to_string();
        task.update(data);
        Ok(task)
    }
    
    async fn delete(&self, _id: &str) -> TaskServiceResult<()> {
        // Mock implementation - replace with actual deletion logic
        Ok(())
    }
}

impl Default for MockTaskService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("test@example.com", "testuser");
        assert_eq!(task.email, "test@example.com");
        assert_eq!(task.username, "testuser");
        assert_eq!(task.status, TaskStatus::Active);
        assert!(task.is_active());
    }

    #[test]
    fn test_task_update() {
        let mut task = Task::new("original@example.com", "original");
        let original_updated_at = task.updated_at;
        
        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        task.update(UpdateTaskRequest {
            email: Some("updated@example.com".to_string()),
            username: Some("updated".to_string()),
            full_name: Some("Updated Task".to_string()),
            status: Some(TaskStatus::Inactive),
        });
        
        assert_eq!(task.email, "updated@example.com");
        assert_eq!(task.username, "updated");
        assert_eq!(task.full_name, Some("Updated Task".to_string()));
        assert_eq!(task.status, TaskStatus::Inactive);
        assert!(!task.is_active());
        assert!(task.updated_at > original_updated_at);
    }

    #[tokio::test]
    async fn test_mock_task_service() {
        let service = MockTaskService::new();
        
        // Test process method
        let request = CreateTaskRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            full_name: Some("Test Task".to_string()),
            password: "password123".to_string(),
        };
        
        let response = service.process(request).await.unwrap();
        assert!(response.message.contains("Processed task: testuser"));
        
        // Test get_by_id method
        let result = service.get_by_id("test-id").await.unwrap();
        assert!(result.is_some());
        
        let result = service.get_by_id("non-existent").await.unwrap();
        assert!(result.is_none());
        
        // Test create method
        let create_request = CreateTaskRequest {
            email: "new@example.com".to_string(),
            username: "newuser".to_string(),
            full_name: Some("New Task".to_string()),
            password: "password123".to_string(),
        };
        
        let created = service.create(create_request).await.unwrap();
        assert_eq!(created.email, "new@example.com");
        assert_eq!(created.username, "newuser");
        
        // Test update method
        let update_request = UpdateTaskRequest {
            email: Some("updated@example.com".to_string()),
            username: Some("updateduser".to_string()),
            full_name: None,
            status: Some(TaskStatus::Inactive),
        };
        
        let updated = service.update("test-id", update_request).await.unwrap();
        assert_eq!(updated.email, "updated@example.com");
        
        // Test delete method
        let result = service.delete("test-id").await;
        assert!(result.is_ok());
    }
}