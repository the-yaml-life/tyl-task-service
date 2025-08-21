//! Task domain events for event-driven communication
//!
//! This module defines all events related to task management operations.
//! These events are published to enable loose coupling between services
//! and to support features like notifications, audit logging, and analytics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::{TaskStatus, TaskPriority, TaskContext, DependencyType};

/// Event published when a new task is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreated {
    pub task_id: String,
    pub name: String,
    pub context: TaskContext,
    pub priority: TaskPriority,
    pub assigned_user_id: Option<String>,
    pub project_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Event published when a task is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdated {
    pub task_id: String,
    pub previous_status: TaskStatus,
    pub current_status: TaskStatus,
    pub updated_fields: Vec<String>, // List of field names that were updated
    pub updated_at: DateTime<Utc>,
}

/// Event published when task status changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusChanged {
    pub task_id: String,
    pub previous_status: TaskStatus,
    pub new_status: TaskStatus,
    pub changed_by: Option<String>, // User ID who made the change
    pub comment: Option<String>,    // Optional comment about the status change
    pub changed_at: DateTime<Utc>,
}

/// Event published when a task is assigned to a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssigned {
    pub task_id: String,
    pub user_id: String,
    pub role: String, // "owner", "contributor", "reviewer", etc.
    pub assigned_by: Option<String>, // User ID who made the assignment
    pub assigned_at: DateTime<Utc>,
}

/// Event published when a task assignment is removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUnassigned {
    pub task_id: String,
    pub user_id: String,
    pub unassigned_by: Option<String>,
    pub unassigned_at: DateTime<Utc>,
}

/// Event published when a task is deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDeleted {
    pub task_id: String,
    pub name: String,
    pub deleted_by: Option<String>,
    pub deleted_at: DateTime<Utc>,
}

/// Event published when a dependency is added between tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependencyAdded {
    pub dependency_id: String,
    pub from_task_id: String,
    pub to_task_id: String,
    pub dependency_type: DependencyType,
    pub is_hard_dependency: bool,
    pub delay_days: u32,
    pub added_by: Option<String>,
    pub added_at: DateTime<Utc>,
}

/// Event published when a dependency is removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependencyRemoved {
    pub dependency_id: String,
    pub from_task_id: String,
    pub to_task_id: String,
    pub removed_by: Option<String>,
    pub removed_at: DateTime<Utc>,
}

/// Event published when a subtask relationship is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskAdded {
    pub parent_task_id: String,
    pub child_task_id: String,
    pub added_by: Option<String>,
    pub added_at: DateTime<Utc>,
}

/// Event published when a subtask relationship is removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskRemoved {
    pub parent_task_id: String,
    pub child_task_id: String,
    pub removed_by: Option<String>,
    pub removed_at: DateTime<Utc>,
}

/// Event published when a task becomes overdue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOverdue {
    pub task_id: String,
    pub name: String,
    pub due_date: DateTime<Utc>,
    pub assigned_user_ids: Vec<String>,
    pub days_overdue: i32,
    pub detected_at: DateTime<Utc>,
}

/// Event published when a task is marked as blocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBlocked {
    pub task_id: String,
    pub name: String,
    pub blocked_by_tasks: Vec<String>, // Task IDs that are blocking this task
    pub blocked_reason: Option<String>,
    pub blocked_at: DateTime<Utc>,
}

/// Event published when a previously blocked task becomes unblocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUnblocked {
    pub task_id: String,
    pub name: String,
    pub unblocked_reason: Option<String>,
    pub unblocked_at: DateTime<Utc>,
}

/// Event published when a task is started (status changes to InProgress)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStarted {
    pub task_id: String,
    pub name: String,
    pub started_by: Option<String>,
    pub estimated_completion_date: Option<DateTime<Utc>>,
    pub started_at: DateTime<Utc>,
}

/// Event published when a task is completed (status changes to Done)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompleted {
    pub task_id: String,
    pub name: String,
    pub completed_by: Option<String>,
    pub completion_time_days: Option<i32>, // Days from creation to completion
    pub success_criteria_met: Vec<String>, // List of success criteria that were verified
    pub completed_at: DateTime<Utc>,
}

/// Event published when a task priority is changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPriorityChanged {
    pub task_id: String,
    pub previous_priority: TaskPriority,
    pub new_priority: TaskPriority,
    pub changed_by: Option<String>,
    pub reason: Option<String>,
    pub changed_at: DateTime<Utc>,
}

/// Event published when a comment is added to a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCommentAdded {
    pub task_id: String,
    pub comment_id: String,
    pub content: String,
    pub author_id: String,
    pub is_internal: bool, // Whether the comment is internal to the team
    pub added_at: DateTime<Utc>,
}

/// Event published when an attachment is added to a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAttachmentAdded {
    pub task_id: String,
    pub attachment_id: String,
    pub name: String,
    pub url: String,
    pub file_type: String,
    pub size_bytes: u64,
    pub uploaded_by: String,
    pub uploaded_at: DateTime<Utc>,
}

/// Event published when a task is tagged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTagged {
    pub task_id: String,
    pub tag_id: String,
    pub tag_name: String,
    pub tag_category: String,
    pub tagged_by: Option<String>,
    pub tagged_at: DateTime<Utc>,
}

/// Event published when a tag is removed from a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUntagged {
    pub task_id: String,
    pub tag_id: String,
    pub tag_name: String,
    pub untagged_by: Option<String>,
    pub untagged_at: DateTime<Utc>,
}

/// Event published when a milestone is reached
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneReached {
    pub milestone_id: String,
    pub milestone_name: String,
    pub project_id: Option<String>,
    pub completed_tasks: Vec<String>, // Task IDs that contributed to this milestone
    pub completion_percentage: f64,
    pub reached_at: DateTime<Utc>,
}

/// Event published when circular dependencies are detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependencyDetected {
    pub cycle_id: String,
    pub task_ids_in_cycle: Vec<String>,
    pub cycle_length: u32,
    pub detected_at: DateTime<Utc>,
}

/// Event published when task analytics are calculated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAnalyticsCalculated {
    pub task_id: String,
    pub completion_percentage: f64,
    pub blocking_count: u32,
    pub blocked_by_count: u32,
    pub is_on_critical_path: bool,
    pub priority_score: f64,
    pub calculated_at: DateTime<Utc>,
}

/// Event published when project analytics are updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalyticsUpdated {
    pub project_id: String,
    pub total_tasks: u32,
    pub completed_tasks: u32,
    pub in_progress_tasks: u32,
    pub blocked_tasks: u32,
    pub overdue_tasks: u32,
    pub completion_percentage: f64,
    pub estimated_completion_date: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// Event published for system notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNotification {
    pub notification_id: String,
    pub task_id: String,
    pub recipient_user_ids: Vec<String>,
    pub notification_type: TaskNotificationType,
    pub title: String,
    pub message: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskNotificationType {
    TaskAssigned,
    TaskOverdue,
    TaskCompleted,
    TaskStatusChanged,
    TaskCommentAdded,
    DependencyBlocking,
    MilestoneReached,
    DeadlineApproaching,
    Priority_Changed,
}

/// Batch event for multiple task operations (useful for bulk operations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBatchOperation {
    pub operation_id: String,
    pub operation_type: BatchOperationType,
    pub task_ids: Vec<String>,
    pub performed_by: Option<String>,
    pub operation_data: HashMap<String, serde_json::Value>,
    pub performed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchOperationType {
    BulkStatusUpdate,
    BulkAssignment,
    BulkPriorityChange,
    BulkDelete,
    BulkMove, // Move tasks to different project
}

/// Integration event for external systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalIntegrationEvent {
    pub integration_id: String,
    pub task_id: String,
    pub external_system: String, // "jira", "github", "slack", etc.
    pub external_id: Option<String>, // ID in the external system
    pub event_type: String, // "sync", "export", "import", etc.
    pub payload: HashMap<String, serde_json::Value>,
    pub processed_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_task_created_serialization() {
        let event = TaskCreated {
            task_id: "PROJ1-T001".to_string(),
            name: "Test Task".to_string(),
            context: TaskContext::Work,
            priority: TaskPriority::High,
            assigned_user_id: Some("user123".to_string()),
            project_id: Some("PROJ1".to_string()),
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: TaskCreated = serde_json::from_str(&json).unwrap();
        
        assert_eq!(event.task_id, deserialized.task_id);
        assert_eq!(event.name, deserialized.name);
        assert_eq!(event.context, deserialized.context);
        assert_eq!(event.priority, deserialized.priority);
    }

    #[test]
    fn test_task_status_changed_serialization() {
        let event = TaskStatusChanged {
            task_id: "PROJ1-T001".to_string(),
            previous_status: TaskStatus::Ready,
            new_status: TaskStatus::InProgress,
            changed_by: Some("user123".to_string()),
            comment: Some("Starting work on this task".to_string()),
            changed_at: Utc::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: TaskStatusChanged = serde_json::from_str(&json).unwrap();
        
        assert_eq!(event.task_id, deserialized.task_id);
        assert_eq!(event.previous_status, deserialized.previous_status);
        assert_eq!(event.new_status, deserialized.new_status);
    }

    #[test]
    fn test_task_notification_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("priority".to_string(), serde_json::json!("high"));
        metadata.insert("due_date".to_string(), serde_json::json!("2024-12-31"));

        let event = TaskNotification {
            notification_id: "notif_001".to_string(),
            task_id: "PROJ1-T001".to_string(),
            recipient_user_ids: vec!["user123".to_string(), "user456".to_string()],
            notification_type: TaskNotificationType::TaskOverdue,
            title: "Task Overdue".to_string(),
            message: "Your task 'Test Task' is overdue".to_string(),
            metadata,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: TaskNotification = serde_json::from_str(&json).unwrap();
        
        assert_eq!(event.task_id, deserialized.task_id);
        assert_eq!(event.recipient_user_ids.len(), deserialized.recipient_user_ids.len());
        assert!(deserialized.metadata.contains_key("priority"));
        assert!(deserialized.metadata.contains_key("due_date"));
    }

    #[test]
    fn test_circular_dependency_event() {
        let event = CircularDependencyDetected {
            cycle_id: "cycle_001".to_string(),
            task_ids_in_cycle: vec![
                "PROJ1-T001".to_string(),
                "PROJ1-T002".to_string(),
                "PROJ1-T003".to_string(),
            ],
            cycle_length: 3,
            detected_at: Utc::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: CircularDependencyDetected = serde_json::from_str(&json).unwrap();
        
        assert_eq!(event.cycle_id, deserialized.cycle_id);
        assert_eq!(event.task_ids_in_cycle.len(), 3);
        assert_eq!(event.cycle_length, 3);
    }
}