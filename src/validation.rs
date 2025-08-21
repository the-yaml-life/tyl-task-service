//! Input validation for API requests
//!
//! This module provides comprehensive validation for all API request types
//! to ensure data integrity and proper error handling.

use crate::{TaskServiceError, TaskServiceResult};
use crate::domain::{CreateTaskRequest, UpdateTaskRequest, TaskStatus, TaskPriority, TaskComplexity};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Maximum allowed length for task names
const MAX_TASK_NAME_LENGTH: usize = 200;

/// Maximum allowed length for task descriptions
const MAX_DESCRIPTION_LENGTH: usize = 5000;

/// Maximum allowed length for task IDs
const MAX_TASK_ID_LENGTH: usize = 100;

/// Maximum allowed number of success criteria
const MAX_SUCCESS_CRITERIA: usize = 20;

/// Maximum allowed number of custom properties
const MAX_CUSTOM_PROPERTIES: usize = 50;

/// Trait for request validation
pub trait Validate {
    fn validate(&self) -> TaskServiceResult<()>;
}

impl Validate for CreateTaskRequest {
    fn validate(&self) -> TaskServiceResult<()> {
        // Validate task ID
        validate_task_id(&self.id)?;
        
        // Validate task name
        validate_task_name(&self.name)?;
        
        // Validate description
        if let Some(ref description) = self.description {
            validate_description(description)?;
        }
        
        // Validate dates
        validate_dates(self.due_date, self.estimated_date)?;
        
        // Validate success criteria
        validate_success_criteria(&self.success_criteria)?;
        
        // Validate implementation details
        if let Some(ref details) = self.implementation_details {
            validate_implementation_details(details)?;
        }
        
        // Validate custom properties
        validate_custom_properties(&self.custom_properties)?;
        
        // Validate user and project IDs
        if let Some(ref user_id) = self.assigned_user_id {
            validate_user_id(user_id)?;
        }
        
        if let Some(ref project_id) = self.project_id {
            validate_project_id(project_id)?;
        }
        
        Ok(())
    }
}

impl Validate for UpdateTaskRequest {
    fn validate(&self) -> TaskServiceResult<()> {
        // Validate name if provided
        if let Some(ref name) = self.name {
            validate_task_name(name)?;
        }
        
        // Validate description if provided
        if let Some(ref description) = self.description {
            validate_description(description)?;
        }
        
        // Validate dates if provided
        validate_dates(self.due_date, self.estimated_date)?;
        
        // Validate success criteria if provided
        if let Some(ref criteria) = self.success_criteria {
            validate_success_criteria(criteria)?;
        }
        
        // Validate implementation details if provided
        if let Some(ref details) = self.implementation_details {
            validate_implementation_details(details)?;
        }
        
        // Validate custom properties if provided
        if let Some(ref properties) = self.custom_properties {
            validate_custom_properties(properties)?;
        }
        
        Ok(())
    }
}

/// Validate task ID format and length
fn validate_task_id(id: &str) -> TaskServiceResult<()> {
    if id.is_empty() {
        return Err(TaskServiceError::InvalidInput {
            field: "id".to_string(),
            message: "Task ID cannot be empty".to_string(),
        });
    }
    
    if id.len() > MAX_TASK_ID_LENGTH {
        return Err(TaskServiceError::InvalidInput {
            field: "id".to_string(),
            message: format!("Task ID cannot exceed {} characters", MAX_TASK_ID_LENGTH),
        });
    }
    
    // Check for valid characters (alphanumeric, hyphens, underscores)
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(TaskServiceError::InvalidInput {
            field: "id".to_string(),
            message: "Task ID can only contain alphanumeric characters, hyphens, and underscores".to_string(),
        });
    }
    
    Ok(())
}

/// Validate task name
fn validate_task_name(name: &str) -> TaskServiceResult<()> {
    if name.trim().is_empty() {
        return Err(TaskServiceError::InvalidInput {
            field: "name".to_string(),
            message: "Task name cannot be empty".to_string(),
        });
    }
    
    if name.len() > MAX_TASK_NAME_LENGTH {
        return Err(TaskServiceError::InvalidInput {
            field: "name".to_string(),
            message: format!("Task name cannot exceed {} characters", MAX_TASK_NAME_LENGTH),
        });
    }
    
    // Check for potentially dangerous characters
    if name.contains('\0') || name.contains('\x1f') {
        return Err(TaskServiceError::InvalidInput {
            field: "name".to_string(),
            message: "Task name contains invalid characters".to_string(),
        });
    }
    
    Ok(())
}

/// Validate task description
fn validate_description(description: &str) -> TaskServiceResult<()> {
    if description.len() > MAX_DESCRIPTION_LENGTH {
        return Err(TaskServiceError::InvalidInput {
            field: "description".to_string(),
            message: format!("Description cannot exceed {} characters", MAX_DESCRIPTION_LENGTH),
        });
    }
    
    Ok(())
}

/// Validate dates are logical
fn validate_dates(due_date: Option<DateTime<Utc>>, estimated_date: Option<DateTime<Utc>>) -> TaskServiceResult<()> {
    if let (Some(due), Some(estimated)) = (due_date, estimated_date) {
        if estimated > due {
            return Err(TaskServiceError::InvalidInput {
                field: "estimated_date".to_string(),
                message: "Estimated completion date cannot be after due date".to_string(),
            });
        }
    }
    
    // Check that dates are not too far in the past or future
    let now = Utc::now();
    let max_future = now + chrono::Duration::days(365 * 10); // 10 years
    let min_past = now - chrono::Duration::days(365 * 5); // 5 years
    
    if let Some(due) = due_date {
        if due > max_future {
            return Err(TaskServiceError::InvalidInput {
                field: "due_date".to_string(),
                message: "Due date cannot be more than 10 years in the future".to_string(),
            });
        }
        if due < min_past {
            return Err(TaskServiceError::InvalidInput {
                field: "due_date".to_string(),
                message: "Due date cannot be more than 5 years in the past".to_string(),
            });
        }
    }
    
    if let Some(estimated) = estimated_date {
        if estimated > max_future {
            return Err(TaskServiceError::InvalidInput {
                field: "estimated_date".to_string(),
                message: "Estimated date cannot be more than 10 years in the future".to_string(),
            });
        }
        if estimated < min_past {
            return Err(TaskServiceError::InvalidInput {
                field: "estimated_date".to_string(),
                message: "Estimated date cannot be more than 5 years in the past".to_string(),
            });
        }
    }
    
    Ok(())
}

/// Validate success criteria
fn validate_success_criteria(criteria: &[crate::domain::SuccessCriterion]) -> TaskServiceResult<()> {
    if criteria.len() > MAX_SUCCESS_CRITERIA {
        return Err(TaskServiceError::InvalidInput {
            field: "success_criteria".to_string(),
            message: format!("Cannot have more than {} success criteria", MAX_SUCCESS_CRITERIA),
        });
    }
    
    for (i, criterion) in criteria.iter().enumerate() {
        if criterion.criterion.trim().is_empty() {
            return Err(TaskServiceError::InvalidInput {
                field: format!("success_criteria[{}].criterion", i),
                message: "Success criterion cannot be empty".to_string(),
            });
        }
        
        if criterion.criterion.len() > 500 {
            return Err(TaskServiceError::InvalidInput {
                field: format!("success_criteria[{}].criterion", i),
                message: "Success criterion cannot exceed 500 characters".to_string(),
            });
        }
        
        if criterion.verification_method.trim().is_empty() {
            return Err(TaskServiceError::InvalidInput {
                field: format!("success_criteria[{}].verification_method", i),
                message: "Verification method cannot be empty".to_string(),
            });
        }
    }
    
    Ok(())
}

/// Validate implementation details
fn validate_implementation_details(details: &str) -> TaskServiceResult<()> {
    if details.len() > MAX_DESCRIPTION_LENGTH {
        return Err(TaskServiceError::InvalidInput {
            field: "implementation_details".to_string(),
            message: format!("Implementation details cannot exceed {} characters", MAX_DESCRIPTION_LENGTH),
        });
    }
    
    Ok(())
}

/// Validate custom properties
fn validate_custom_properties(properties: &HashMap<String, serde_json::Value>) -> TaskServiceResult<()> {
    if properties.len() > MAX_CUSTOM_PROPERTIES {
        return Err(TaskServiceError::InvalidInput {
            field: "custom_properties".to_string(),
            message: format!("Cannot have more than {} custom properties", MAX_CUSTOM_PROPERTIES),
        });
    }
    
    for (key, value) in properties {
        // Validate key
        if key.trim().is_empty() {
            return Err(TaskServiceError::InvalidInput {
                field: "custom_properties".to_string(),
                message: "Custom property keys cannot be empty".to_string(),
            });
        }
        
        if key.len() > 100 {
            return Err(TaskServiceError::InvalidInput {
                field: "custom_properties".to_string(),
                message: "Custom property keys cannot exceed 100 characters".to_string(),
            });
        }
        
        // Validate value size (prevent excessively large JSON values)
        let value_str = serde_json::to_string(value).map_err(|_| {
            TaskServiceError::InvalidInput {
                field: "custom_properties".to_string(),
                message: "Invalid JSON value in custom properties".to_string(),
            }
        })?;
        
        if value_str.len() > 10000 {
            return Err(TaskServiceError::InvalidInput {
                field: "custom_properties".to_string(),
                message: "Custom property values cannot exceed 10KB when serialized".to_string(),
            });
        }
    }
    
    Ok(())
}

/// Validate user ID format
fn validate_user_id(user_id: &str) -> TaskServiceResult<()> {
    if user_id.trim().is_empty() {
        return Err(TaskServiceError::InvalidInput {
            field: "assigned_user_id".to_string(),
            message: "User ID cannot be empty".to_string(),
        });
    }
    
    if user_id.len() > 100 {
        return Err(TaskServiceError::InvalidInput {
            field: "assigned_user_id".to_string(),
            message: "User ID cannot exceed 100 characters".to_string(),
        });
    }
    
    Ok(())
}

/// Validate project ID format
fn validate_project_id(project_id: &str) -> TaskServiceResult<()> {
    if project_id.trim().is_empty() {
        return Err(TaskServiceError::InvalidInput {
            field: "project_id".to_string(),
            message: "Project ID cannot be empty".to_string(),
        });
    }
    
    if project_id.len() > 100 {
        return Err(TaskServiceError::InvalidInput {
            field: "project_id".to_string(),
            message: "Project ID cannot exceed 100 characters".to_string(),
        });
    }
    
    Ok(())
}

/// Validate status transition request
pub fn validate_status_transition(current: &TaskStatus, new: &TaskStatus) -> TaskServiceResult<()> {
    if !current.can_transition_to(new) {
        return Err(TaskServiceError::InvalidStatusTransition {
            from: format!("{:?}", current),
            to: format!("{:?}", new),
        });
    }
    
    Ok(())
}

/// Validate status transition with context-aware business rules
pub fn validate_status_transition_with_context(
    current: &TaskStatus, 
    new: &TaskStatus,
    task_context: Option<&str>,
    has_assignee: bool,
    has_dependencies: bool
) -> TaskServiceResult<()> {
    // First check basic transition validity
    validate_status_transition(current, new)?;
    
    // Context-specific validations
    match new {
        TaskStatus::InProgress => {
            if !has_assignee {
                return Err(TaskServiceError::InvalidInput {
                    field: "status".to_string(),
                    message: "Cannot start work on unassigned task".to_string(),
                });
            }
            
            // Critical tasks should not skip Ready state
            if matches!(current, TaskStatus::Backlog) && task_context == Some("critical") {
                return Err(TaskServiceError::InvalidInput {
                    field: "status".to_string(),
                    message: "Critical tasks must be reviewed in Ready state before starting".to_string(),
                });
            }
        },
        
        TaskStatus::Done => {
            // Tasks with dependencies need special validation
            if has_dependencies {
                return Err(TaskServiceError::InvalidInput {
                    field: "status".to_string(),
                    message: "Tasks with dependencies require dependency validation before completion".to_string(),
                });
            }
        },
        
        TaskStatus::Review => {
            // Ensure task has progressed beyond initial states
            if matches!(current, TaskStatus::Backlog | TaskStatus::Ready) {
                return Err(TaskServiceError::InvalidInput {
                    field: "status".to_string(),
                    message: "Tasks must show work progress before review".to_string(),
                });
            }
        },
        
        _ => {} // No additional context validations needed
    }
    
    Ok(())
}

/// Validate bulk status transition for multiple tasks
pub fn validate_bulk_status_transition(
    transitions: &[(TaskStatus, TaskStatus)]
) -> TaskServiceResult<()> {
    for (i, (current, new)) in transitions.iter().enumerate() {
        validate_status_transition(current, new).map_err(|e| {
            TaskServiceError::InvalidInput {
                field: format!("transitions[{}]", i),
                message: format!("Invalid transition: {}", e),
            }
        })?;
    }
    
    Ok(())
}

/// Get suggested next statuses for a given current status
pub fn get_suggested_next_statuses(current: &TaskStatus) -> Vec<TaskStatus> {
    current.valid_next_statuses()
}

/// Validate that a status transition is not too aggressive (skipping important steps)
pub fn validate_transition_pace(current: &TaskStatus, new: &TaskStatus) -> TaskServiceResult<()> {
    let aggressive_transitions = [
        (TaskStatus::Backlog, TaskStatus::Review),
        (TaskStatus::Backlog, TaskStatus::Done),
        (TaskStatus::Ready, TaskStatus::Done),
        (TaskStatus::Blocked, TaskStatus::Done),
    ];
    
    if aggressive_transitions.contains(&(*current, *new)) {
        return Err(TaskServiceError::InvalidInput {
            field: "status".to_string(),
            message: format!(
                "Transition from {:?} to {:?} skips important workflow steps. Consider intermediate statuses.",
                current, new
            ),
        });
    }
    
    Ok(())
}

/// Validate task dependency request
pub fn validate_dependency_request(from_task_id: &str, to_task_id: &str) -> TaskServiceResult<()> {
    validate_task_id(from_task_id)?;
    validate_task_id(to_task_id)?;
    
    if from_task_id == to_task_id {
        return Err(TaskServiceError::DependencyViolation {
            message: "A task cannot depend on itself".to_string(),
        });
    }
    
    Ok(())
}

/// Validate assignment request
pub fn validate_assignment_request(task_id: &str, user_id: &str, role: &str) -> TaskServiceResult<()> {
    validate_task_id(task_id)?;
    validate_user_id(user_id)?;
    
    if role.trim().is_empty() {
        return Err(TaskServiceError::InvalidInput {
            field: "role".to_string(),
            message: "Role cannot be empty".to_string(),
        });
    }
    
    if role.len() > 50 {
        return Err(TaskServiceError::InvalidInput {
            field: "role".to_string(),
            message: "Role cannot exceed 50 characters".to_string(),
        });
    }
    
    // Validate role is one of the allowed values
    let allowed_roles = ["assignee", "reviewer", "observer", "collaborator"];
    if !allowed_roles.contains(&role) {
        return Err(TaskServiceError::InvalidInput {
            field: "role".to_string(),
            message: format!("Role must be one of: {}", allowed_roles.join(", ")),
        });
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{TaskContext, TaskSource, TaskVisibility};
    
    #[test]
    fn test_valid_create_task_request() {
        let request = CreateTaskRequest {
            id: "TEST-001".to_string(),
            name: "Valid task name".to_string(),
            description: Some("Valid description".to_string()),
            context: TaskContext::Work,
            priority: TaskPriority::Medium,
            complexity: TaskComplexity::Simple,
            due_date: Some(Utc::now() + chrono::Duration::days(7)),
            estimated_date: Some(Utc::now() + chrono::Duration::days(5)),
            implementation_details: None,
            success_criteria: vec![],
            test_strategy: None,
            source: TaskSource::Self_,
            visibility: TaskVisibility::Private,
            recurrence: None,
            custom_properties: HashMap::new(),
            assigned_user_id: None,
            project_id: None,
        };
        
        assert!(request.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_task_name() {
        let mut request = CreateTaskRequest {
            id: "TEST-001".to_string(),
            name: "".to_string(), // Empty name
            description: None,
            context: TaskContext::Work,
            priority: TaskPriority::Medium,
            complexity: TaskComplexity::Simple,
            due_date: None,
            estimated_date: None,
            implementation_details: None,
            success_criteria: vec![],
            test_strategy: None,
            source: TaskSource::Self_,
            visibility: TaskVisibility::Private,
            recurrence: None,
            custom_properties: HashMap::new(),
            assigned_user_id: None,
            project_id: None,
        };
        
        assert!(request.validate().is_err());
        
        // Test name too long
        request.name = "a".repeat(MAX_TASK_NAME_LENGTH + 1);
        assert!(request.validate().is_err());
    }
    
    #[test]
    fn test_invalid_dates() {
        let request = CreateTaskRequest {
            id: "TEST-001".to_string(),
            name: "Valid name".to_string(),
            description: None,
            context: TaskContext::Work,
            priority: TaskPriority::Medium,
            complexity: TaskComplexity::Simple,
            due_date: Some(Utc::now() + chrono::Duration::days(5)),
            estimated_date: Some(Utc::now() + chrono::Duration::days(10)), // After due date
            implementation_details: None,
            success_criteria: vec![],
            test_strategy: None,
            source: TaskSource::Self_,
            visibility: TaskVisibility::Private,
            recurrence: None,
            custom_properties: HashMap::new(),
            assigned_user_id: None,
            project_id: None,
        };
        
        assert!(request.validate().is_err());
    }
    
    #[test]
    fn test_dependency_validation() {
        // Valid dependency
        assert!(validate_dependency_request("TASK-1", "TASK-2").is_ok());
        
        // Self-dependency should fail
        assert!(validate_dependency_request("TASK-1", "TASK-1").is_err());
        
        // Invalid task IDs should fail
        assert!(validate_dependency_request("", "TASK-2").is_err());
        assert!(validate_dependency_request("TASK-1", "").is_err());
    }
    
    #[test]
    fn test_assignment_validation() {
        // Valid assignment
        assert!(validate_assignment_request("TASK-1", "user123", "assignee").is_ok());
        
        // Invalid role should fail
        assert!(validate_assignment_request("TASK-1", "user123", "invalid_role").is_err());
        
        // Empty role should fail
        assert!(validate_assignment_request("TASK-1", "user123", "").is_err());
    }
    
    #[test]
    fn test_status_transition_validation() {
        use crate::domain::TaskStatus;
        
        // Valid transitions
        assert!(validate_status_transition(&TaskStatus::Backlog, &TaskStatus::Ready).is_ok());
        assert!(validate_status_transition(&TaskStatus::Ready, &TaskStatus::InProgress).is_ok());
        assert!(validate_status_transition(&TaskStatus::InProgress, &TaskStatus::Review).is_ok());
        assert!(validate_status_transition(&TaskStatus::Review, &TaskStatus::Done).is_ok());
        
        // Invalid transitions
        assert!(validate_status_transition(&TaskStatus::Backlog, &TaskStatus::InProgress).is_err());
        assert!(validate_status_transition(&TaskStatus::Done, &TaskStatus::InProgress).is_err());
        assert!(validate_status_transition(&TaskStatus::Cancelled, &TaskStatus::Ready).is_err());
    }
    
    #[test]
    fn test_status_transition_with_context() {
        use crate::domain::TaskStatus;
        
        // Valid transition with assignee
        assert!(validate_status_transition_with_context(
            &TaskStatus::Ready, 
            &TaskStatus::InProgress,
            None,
            true,  // has assignee
            false  // no dependencies
        ).is_ok());
        
        // Invalid transition without assignee
        assert!(validate_status_transition_with_context(
            &TaskStatus::Ready, 
            &TaskStatus::InProgress,
            None,
            false, // no assignee
            false
        ).is_err());
        
        // Critical task should not skip Ready state
        assert!(validate_status_transition_with_context(
            &TaskStatus::Backlog, 
            &TaskStatus::InProgress,
            Some("critical"),
            true,
            false
        ).is_err());
        
        // Task with dependencies completing
        assert!(validate_status_transition_with_context(
            &TaskStatus::Review, 
            &TaskStatus::Done,
            None,
            true,
            true  // has dependencies
        ).is_err());
    }
    
    #[test]
    fn test_transition_pace_validation() {
        use crate::domain::TaskStatus;
        
        // Normal paced transitions should be ok
        assert!(validate_transition_pace(&TaskStatus::Backlog, &TaskStatus::Ready).is_ok());
        assert!(validate_transition_pace(&TaskStatus::Ready, &TaskStatus::InProgress).is_ok());
        
        // Aggressive transitions should fail
        assert!(validate_transition_pace(&TaskStatus::Backlog, &TaskStatus::Review).is_err());
        assert!(validate_transition_pace(&TaskStatus::Backlog, &TaskStatus::Done).is_err());
        assert!(validate_transition_pace(&TaskStatus::Ready, &TaskStatus::Done).is_err());
        assert!(validate_transition_pace(&TaskStatus::Blocked, &TaskStatus::Done).is_err());
    }
    
    #[test]
    fn test_bulk_status_transitions() {
        use crate::domain::TaskStatus;
        
        let valid_transitions = vec![
            (TaskStatus::Backlog, TaskStatus::Ready),
            (TaskStatus::Ready, TaskStatus::InProgress),
            (TaskStatus::InProgress, TaskStatus::Review),
        ];
        
        assert!(validate_bulk_status_transition(&valid_transitions).is_ok());
        
        let invalid_transitions = vec![
            (TaskStatus::Backlog, TaskStatus::Ready),
            (TaskStatus::Done, TaskStatus::InProgress), // Invalid
            (TaskStatus::Ready, TaskStatus::InProgress),
        ];
        
        assert!(validate_bulk_status_transition(&invalid_transitions).is_err());
    }
    
    #[test]
    fn test_suggested_next_statuses() {
        use crate::domain::TaskStatus;
        
        let suggestions = get_suggested_next_statuses(&TaskStatus::Backlog);
        assert!(suggestions.contains(&TaskStatus::Ready));
        assert!(suggestions.contains(&TaskStatus::Cancelled));
        
        let suggestions = get_suggested_next_statuses(&TaskStatus::InProgress);
        assert!(suggestions.contains(&TaskStatus::Blocked));
        assert!(suggestions.contains(&TaskStatus::Review));
        assert!(suggestions.contains(&TaskStatus::Done));
        assert!(suggestions.contains(&TaskStatus::Cancelled));
        
        let suggestions = get_suggested_next_statuses(&TaskStatus::Cancelled);
        assert!(suggestions.is_empty());
    }
}