//! Task management HTTP handlers
//!
//! This module provides REST API endpoints for task management operations,
//! integrating with the graph-based task service and event system.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Json, IntoResponse},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tyl_errors::TylError;
use uuid::Uuid;

use crate::{
    domain::{
        TaskService, CreateTaskRequest, UpdateTaskRequest, TaskFilter, CreateProjectRequest,
        Task, Project, TaskDependency, TaskStatus, TaskPriority, TaskContext, TaskComplexity,
        TaskSource, TaskVisibility, DependencyType, TaskAnalytics,
    },
    events::{EventService, TaskCreated, TaskUpdated, TaskStatusChanged, TaskAssigned},
    handlers::ApiError,
    AppState, TaskServiceError, LogLevel, LogRecord,
};
use tokio::time::{sleep, Duration};

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateTaskApiRequest {
    pub name: String,
    pub description: Option<String>,
    pub context: TaskContext,
    pub priority: Option<TaskPriority>,
    pub complexity: Option<TaskComplexity>,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_date: Option<DateTime<Utc>>,
    pub implementation_details: Option<String>,
    pub success_criteria: Option<Vec<SuccessCriterionDto>>,
    pub test_strategy: Option<String>,
    pub source: Option<TaskSource>,
    pub visibility: Option<TaskVisibility>,
    pub recurrence: Option<TaskRecurrenceDto>,
    pub custom_properties: Option<HashMap<String, serde_json::Value>>,
    pub assigned_user_id: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskApiRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,
    pub complexity: Option<TaskComplexity>,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_date: Option<DateTime<Utc>>,
    pub implementation_details: Option<String>,
    pub success_criteria: Option<Vec<SuccessCriterionDto>>,
    pub test_strategy: Option<String>,
    pub visibility: Option<TaskVisibility>,
    pub custom_properties: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessCriterionDto {
    pub criterion: String,
    pub measurable: bool,
    pub verification_method: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskRecurrenceDto {
    pub pattern: String,
    pub interval: u32,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct TaskStatusTransitionRequest {
    pub new_status: TaskStatus,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddDependencyRequest {
    pub to_task_id: String,
    pub dependency_type: DependencyType,
    pub is_hard_dependency: Option<bool>,
    pub delay_days: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct AssignTaskRequest {
    pub user_id: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TaskQueryParams {
    pub status: Option<String>, // Comma-separated statuses
    pub priority: Option<String>, // Comma-separated priorities
    pub context: Option<String>, // Comma-separated contexts
    pub assigned_user_id: Option<String>,
    pub project_id: Option<String>,
    pub due_before: Option<DateTime<Utc>>,
    pub due_after: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
    pub is_overdue: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: String,
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
    pub context: TaskContext,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub complexity: TaskComplexity,
    pub implementation_details: Option<String>,
    pub success_criteria: Vec<SuccessCriterionDto>,
    pub test_strategy: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_date: Option<DateTime<Utc>>,
    pub source: TaskSource,
    pub visibility: TaskVisibility,
    pub recurrence: Option<TaskRecurrenceDto>,
    pub attachments: Vec<TaskAttachmentDto>,
    pub custom_properties: HashMap<String, serde_json::Value>,
    pub is_overdue: bool,
    pub is_actionable: bool,
}

#[derive(Debug, Serialize)]
pub struct TaskAttachmentDto {
    pub name: String,
    pub url: String,
    pub attachment_type: String,
    pub size: u64,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub tasks: Vec<TaskResponse>,
    pub total_count: Option<usize>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct TaskDependencyResponse {
    pub id: String,
    pub from_task_id: String,
    pub to_task_id: String,
    pub dependency_type: DependencyType,
    pub is_hard_dependency: bool,
    pub delay_days: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TaskAnalyticsResponse {
    pub task_id: String,
    pub completion_percentage: f64,
    pub blocking_count: u32,
    pub blocked_by_count: u32,
    pub subtask_count: u32,
    pub completed_subtasks: u32,
    pub is_on_critical_path: bool,
    pub estimated_completion_date: Option<DateTime<Utc>>,
    pub time_to_completion_days: Option<i32>,
    pub dependency_chain_length: u32,
    pub priority_score: f64,
}

// ============================================================================
// Conversion functions
// ============================================================================

impl From<&Task> for TaskResponse {
    fn from(task: &Task) -> Self {
        Self {
            id: task.id.clone(),
            uuid: task.uuid.clone(),
            name: task.name.clone(),
            description: task.description.clone(),
            context: task.context.clone(),
            status: task.status.clone(),
            priority: task.priority.clone(),
            complexity: task.complexity.clone(),
            implementation_details: task.implementation_details.clone(),
            success_criteria: task.success_criteria.iter()
                .map(|sc| SuccessCriterionDto {
                    criterion: sc.criterion.clone(),
                    measurable: sc.measurable,
                    verification_method: sc.verification_method.clone(),
                })
                .collect(),
            test_strategy: task.test_strategy.clone(),
            created_at: task.created_at,
            updated_at: task.updated_at,
            started_at: task.started_at,
            completed_at: task.completed_at,
            due_date: task.due_date,
            estimated_date: task.estimated_date,
            source: task.source.clone(),
            visibility: task.visibility.clone(),
            recurrence: task.recurrence.as_ref().map(|r| TaskRecurrenceDto {
                pattern: r.pattern.clone(),
                interval: r.interval,
                end_date: r.end_date,
            }),
            attachments: task.attachments.iter()
                .map(|a| TaskAttachmentDto {
                    name: a.name.clone(),
                    url: a.url.clone(),
                    attachment_type: a.attachment_type.clone(),
                    size: a.size,
                    uploaded_at: a.uploaded_at,
                })
                .collect(),
            custom_properties: task.custom_properties.clone(),
            is_overdue: task.is_overdue(),
            is_actionable: task.is_actionable(),
        }
    }
}

impl From<&TaskDependency> for TaskDependencyResponse {
    fn from(dep: &TaskDependency) -> Self {
        Self {
            id: dep.id.clone(),
            from_task_id: dep.from_task_id.clone(),
            to_task_id: dep.to_task_id.clone(),
            dependency_type: dep.dependency_type.clone(),
            is_hard_dependency: dep.is_hard_dependency,
            delay_days: dep.delay_days,
            created_at: dep.created_at,
        }
    }
}

impl From<&TaskAnalytics> for TaskAnalyticsResponse {
    fn from(analytics: &TaskAnalytics) -> Self {
        Self {
            task_id: analytics.task_id.clone(),
            completion_percentage: analytics.completion_percentage,
            blocking_count: analytics.blocking_count,
            blocked_by_count: analytics.blocked_by_count,
            subtask_count: analytics.subtask_count,
            completed_subtasks: analytics.completed_subtasks,
            is_on_critical_path: analytics.is_on_critical_path,
            estimated_completion_date: analytics.estimated_completion_date,
            time_to_completion_days: analytics.time_to_completion_days,
            dependency_chain_length: analytics.dependency_chain_length,
            priority_score: analytics.priority_score,
        }
    }
}

fn parse_query_list<T>(value: Option<&str>) -> Vec<T> 
where 
    T: std::str::FromStr,
{
    value
        .map(|s| s.split(',').filter_map(|item| item.trim().parse().ok()).collect())
        .unwrap_or_default()
}

/// Helper function to publish events with retry logic
async fn publish_event_with_retry<T>(
    event_service: &EventService,
    topic: &str,
    event: T,
    max_retries: u32,
) -> Result<(), TaskServiceError>
where
    T: Serialize + Clone + Send + Sync,
{
    let mut retries = 0;
    loop {
        match event_service.publish(topic, event.clone()).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                retries += 1;
                if retries >= max_retries {
                    return Err(TaskServiceError::EventPublishing {
                        event_type: topic.to_string(),
                        message: e.to_string(),
                    });
                }
                
                // Exponential backoff: 100ms, 200ms, 400ms, etc.
                let delay = Duration::from_millis(100 * (1 << retries));
                tracing::warn!(
                    "Event publishing failed (attempt {}/{}): {}. Retrying in {:?}",
                    retries, max_retries, e, delay
                );
                sleep(delay).await;
            }
        }
    }
}

fn create_task_filter(params: TaskQueryParams) -> TaskFilter {
    TaskFilter {
        status: if let Some(status_str) = params.status {
            let statuses: Vec<TaskStatus> = status_str.split(',')
                .filter_map(|s| serde_json::from_str(&format!("\"{}\"", s.trim())).ok())
                .collect();
            if statuses.is_empty() { None } else { Some(statuses) }
        } else { None },
        priority: if let Some(priority_str) = params.priority {
            let priorities: Vec<TaskPriority> = priority_str.split(',')
                .filter_map(|s| serde_json::from_str(&format!("\"{}\"", s.trim())).ok())
                .collect();
            if priorities.is_empty() { None } else { Some(priorities) }
        } else { None },
        context: if let Some(context_str) = params.context {
            let contexts: Vec<TaskContext> = context_str.split(',')
                .filter_map(|s| serde_json::from_str(&format!("\"{}\"", s.trim())).ok())
                .collect();
            if contexts.is_empty() { None } else { Some(contexts) }
        } else { None },
        assigned_user_id: params.assigned_user_id,
        project_id: params.project_id,
        due_before: params.due_before,
        due_after: params.due_after,
        due_date_from: None,
        due_date_to: None,
        created_after: params.created_after,
        created_before: None,
        search_text: None,
        tags: None,
        complexity: None,
        has_dependencies: None,
        is_overdue: params.is_overdue,
        limit: params.limit.or(Some(100)),
        offset: params.offset.or(Some(0)),
    }
}

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Create a new task
pub async fn create_task(
    State(state): State<AppState>,
    Json(request): Json<CreateTaskApiRequest>,
) -> Result<Json<TaskResponse>, ApiError> {
    // Start tracing span for this request
    let span_id = state.tracer.start_span("create_task", None)
        .map_err(|e| ApiError::internal_server_error(format!("Tracing error: {}", e)))?;

    // Log request received
    state.logger.log(&LogRecord::new(LogLevel::Info, 
        &format!("Creating new task: {}", request.name)));

    // Generate task ID (in a real implementation, this would be more sophisticated)
    let task_id = if let Some(ref project_id) = request.project_id {
        format!("{}-T{}", project_id, Uuid::new_v4().simple().to_string()[..8].to_uppercase())
    } else {
        format!("TASK-{}", Uuid::new_v4().simple().to_string()[..8].to_uppercase())
    };

    // Add task ID to trace span
    state.tracer.add_span_attribute(&span_id, "task_id", serde_json::json!(task_id.clone()))
        .map_err(|e| ApiError::internal_server_error(format!("Tracing error: {}", e)))?;

    // Convert API request to domain request
    let domain_request = CreateTaskRequest {
        id: task_id,
        name: request.name,
        description: request.description,
        context: request.context,
        priority: request.priority.unwrap_or(TaskPriority::Medium),
        complexity: request.complexity.unwrap_or(TaskComplexity::Medium),
        due_date: request.due_date,
        estimated_date: request.estimated_date,
        implementation_details: request.implementation_details,
        success_criteria: request.success_criteria.unwrap_or_default().into_iter()
            .map(|sc| crate::domain::SuccessCriterion {
                criterion: sc.criterion,
                measurable: sc.measurable,
                verification_method: sc.verification_method,
            })
            .collect(),
        test_strategy: request.test_strategy,
        source: request.source.unwrap_or(TaskSource::Self_),
        visibility: request.visibility.unwrap_or(TaskVisibility::Private),
        recurrence: request.recurrence.map(|r| crate::domain::TaskRecurrence {
            pattern: r.pattern,
            interval: r.interval,
            end_date: r.end_date,
        }),
        custom_properties: request.custom_properties.unwrap_or_default(),
        assigned_user_id: request.assigned_user_id.clone(),
        project_id: request.project_id.clone(),
    };

    // Create the task
    let task = match state.domain_service.create_task(domain_request).await {
        Ok(task) => task,
        Err(e) => {
            state.logger.log(&LogRecord::new(LogLevel::Error, 
                &format!("Failed to create task: {}", e)));
            let _ = state.tracer.end_span(span_id);
            return Err(ApiError::from(e));
        }
    };

    // Publish task created event
    let event = TaskCreated {
        task_id: task.id.clone(),
        name: task.name.clone(),
        context: task.context.clone(),
        priority: task.priority.clone(),
        assigned_user_id: request.assigned_user_id,
        project_id: request.project_id,
        created_at: task.created_at,
    };
    
    // Publish task created event with retry logic
    if let Err(e) = publish_event_with_retry(&state.event_service, "task.created", event, 3).await {
        state.logger.log(&LogRecord::new(LogLevel::Error, 
            &format!("Failed to publish task.created event after retries: {}", e)));
        // We don't fail the request if event publishing fails, but we log it as an error
    }

    // Log successful task creation
    state.logger.log(&LogRecord::new(LogLevel::Info, 
        &format!("Task created successfully: {}", task.id)));

    // End tracing span
    let _ = state.tracer.end_span(span_id);

    Ok(Json(TaskResponse::from(&task)))
}

/// Get a task by ID
pub async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, ApiError> {
    match state.domain_service.get_task_by_id(&task_id).await
        .map_err(ApiError::from)?
    {
        Some(task) => Ok(Json(TaskResponse::from(&task))),
        None => Err(ApiError::not_found("Task", task_id)),
    }
}

/// Update an existing task
pub async fn update_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(request): Json<UpdateTaskApiRequest>,
) -> Result<Json<TaskResponse>, ApiError> {
    // Get original task for comparison
    let original_task = state.domain_service.get_task_by_id(&task_id).await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("Task", &task_id))?;

    // Convert API request to domain request
    let domain_request = UpdateTaskRequest {
        name: request.name,
        description: request.description,
        priority: request.priority,
        complexity: request.complexity,
        due_date: request.due_date,
        estimated_date: request.estimated_date,
        implementation_details: request.implementation_details,
        success_criteria: request.success_criteria.map(|criteria| 
            criteria.into_iter().map(|sc| crate::domain::SuccessCriterion {
                criterion: sc.criterion,
                measurable: sc.measurable,
                verification_method: sc.verification_method,
            }).collect()
        ),
        test_strategy: request.test_strategy,
        visibility: request.visibility,
        custom_properties: request.custom_properties,
    };

    // Update the task
    let updated_task = state.domain_service.update_task(&task_id, domain_request).await
        .map_err(ApiError::from)?;

    // Publish task updated event
    let event = TaskUpdated {
        task_id: updated_task.id.clone(),
        previous_status: original_task.status,
        current_status: updated_task.status.clone(),
        updated_fields: vec![], // In a real implementation, track which fields changed
        updated_at: updated_task.updated_at,
    };
    
    if let Err(e) = state.event_service.publish("task.updated", event).await {
        tracing::warn!("Failed to publish task.updated event: {}", e);
    }

    Ok(Json(TaskResponse::from(&updated_task)))
}

/// Delete a task
pub async fn delete_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.domain_service.delete_task(&task_id).await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

/// List tasks with filtering
pub async fn list_tasks(
    State(state): State<AppState>,
    Query(params): Query<TaskQueryParams>,
) -> Result<Json<TaskListResponse>, ApiError> {
    let filter = create_task_filter(params);
    let tasks = state.domain_service.list_tasks(filter).await
        .map_err(ApiError::from)?;

    let task_responses: Vec<TaskResponse> = tasks.iter()
        .map(TaskResponse::from)
        .collect();

    let response = TaskListResponse {
        has_more: false, // In a real implementation, check if there are more results
        total_count: Some(task_responses.len()),
        tasks: task_responses,
    };

    Ok(Json(response))
}

/// Transition task status
pub async fn transition_task_status(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(request): Json<TaskStatusTransitionRequest>,
) -> Result<Json<TaskResponse>, ApiError> {
    let original_status = state.domain_service.get_task_by_id(&task_id).await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("Task", &task_id))?
        .status;

    let updated_task = state.domain_service.transition_task_status(&task_id, request.new_status.clone()).await
        .map_err(ApiError::from)?;

    // Publish status change event
    let event = TaskStatusChanged {
        task_id: updated_task.id.clone(),
        previous_status: original_status,
        new_status: request.new_status,
        changed_by: None, // In a real implementation, get from auth context
        comment: request.comment,
        changed_at: updated_task.updated_at,
    };
    
    if let Err(e) = publish_event_with_retry(&state.event_service, "task.status_changed", event, 3).await {
        tracing::error!("Failed to publish task.status_changed event after retries: {}", e);
    }

    Ok(Json(TaskResponse::from(&updated_task)))
}

/// Add task dependency
pub async fn add_task_dependency(
    State(state): State<AppState>,
    Path(from_task_id): Path<String>,
    Json(request): Json<AddDependencyRequest>,
) -> Result<Json<TaskDependencyResponse>, ApiError> {
    let dependency = state.domain_service.add_task_dependency(
        &from_task_id,
        &request.to_task_id,
        request.dependency_type,
    ).await.map_err(ApiError::from)?;

    Ok(Json(TaskDependencyResponse::from(&dependency)))
}

/// Get task dependencies
pub async fn get_task_dependencies(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Vec<TaskDependencyResponse>>, ApiError> {
    let dependencies = state.domain_service.get_task_dependencies(&task_id).await
        .map_err(ApiError::from)?;

    let responses: Vec<TaskDependencyResponse> = dependencies.iter()
        .map(TaskDependencyResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Assign task to user
pub async fn assign_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(request): Json<AssignTaskRequest>,
) -> Result<StatusCode, ApiError> {
    let role = request.role.as_deref().unwrap_or("owner");
    
    state.domain_service.assign_task(&task_id, &request.user_id, role).await
        .map_err(ApiError::from)?;

    // Publish task assigned event
    let event = TaskAssigned {
        task_id: task_id.clone(),
        user_id: request.user_id,
        role: role.to_string(),
        assigned_by: None, // In a real implementation, get from auth context
        assigned_at: Utc::now(),
    };
    
    if let Err(e) = state.event_service.publish("task.assigned", event).await {
        tracing::warn!("Failed to publish task.assigned event: {}", e);
    }

    Ok(StatusCode::OK)
}

/// Get assigned tasks for a user
pub async fn get_assigned_tasks(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<TaskListResponse>, ApiError> {
    let tasks = state.domain_service.get_assigned_tasks(&user_id).await
        .map_err(ApiError::from)?;

    let task_responses: Vec<TaskResponse> = tasks.iter()
        .map(TaskResponse::from)
        .collect();

    let response = TaskListResponse {
        has_more: false,
        total_count: Some(task_responses.len()),
        tasks: task_responses,
    };

    Ok(Json(response))
}

/// Get actionable tasks for a user
pub async fn get_actionable_tasks(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<TaskListResponse>, ApiError> {
    let tasks = state.domain_service.get_actionable_tasks(&user_id).await
        .map_err(ApiError::from)?;

    let task_responses: Vec<TaskResponse> = tasks.iter()
        .map(TaskResponse::from)
        .collect();

    let response = TaskListResponse {
        has_more: false,
        total_count: Some(task_responses.len()),
        tasks: task_responses,
    };

    Ok(Json(response))
}

/// Get overdue tasks
pub async fn get_overdue_tasks(
    State(state): State<AppState>,
) -> Result<Json<TaskListResponse>, ApiError> {
    let tasks = state.domain_service.get_overdue_tasks().await
        .map_err(ApiError::from)?;

    let task_responses: Vec<TaskResponse> = tasks.iter()
        .map(TaskResponse::from)
        .collect();

    let response = TaskListResponse {
        has_more: false,
        total_count: Some(task_responses.len()),
        tasks: task_responses,
    };

    Ok(Json(response))
}

/// Get task analytics
pub async fn get_task_analytics(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskAnalyticsResponse>, ApiError> {
    let analytics = state.domain_service.get_task_analytics(&task_id).await
        .map_err(ApiError::from)?;

    Ok(Json(TaskAnalyticsResponse::from(&analytics)))
}

/// Add subtask
pub async fn add_subtask(
    State(state): State<AppState>,
    Path((parent_id, child_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    state.domain_service.add_subtask(&parent_id, &child_id).await
        .map_err(ApiError::from)?;

    Ok(StatusCode::OK)
}

/// Get subtasks
pub async fn get_subtasks(
    State(state): State<AppState>,
    Path(parent_id): Path<String>,
) -> Result<Json<TaskListResponse>, ApiError> {
    let tasks = state.domain_service.get_subtasks(&parent_id).await
        .map_err(ApiError::from)?;

    let task_responses: Vec<TaskResponse> = tasks.iter()
        .map(TaskResponse::from)
        .collect();

    let response = TaskListResponse {
        has_more: false,
        total_count: Some(task_responses.len()),
        tasks: task_responses,
    };

    Ok(Json(response))
}

/// Get circular dependency analysis
pub async fn get_circular_dependencies(
    State(state): State<AppState>,
) -> Result<Json<CircularDependenciesResponse>, ApiError> {
    let cycles = state.domain_service.get_detailed_circular_dependencies().await
        .map_err(ApiError::from)?;

    let response = CircularDependenciesResponse {
        total_cycles: cycles.len(),
        has_critical_cycles: cycles.iter().any(|c| matches!(c.severity, crate::domain::queries::CycleSeverity::Critical)),
        cycles,
    };

    Ok(Json(response))
}

/// Response for circular dependency analysis
#[derive(Debug, Serialize)]
pub struct CircularDependenciesResponse {
    pub total_cycles: usize,
    pub cycles: Vec<crate::domain::queries::DependencyCycle>,
    pub has_critical_cycles: bool,
}