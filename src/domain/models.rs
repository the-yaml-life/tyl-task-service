//! Domain models for the task management system
//!
//! These models represent the core entities in our graph-based task management system.
//! Following the comprehensive schema provided, these types map directly to graph nodes
//! and relationships in FalkorDB through tyl-graph-port.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tyl_errors::{TylError, TylResult};

/// Task context categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskContext {
    Work,
    Personal,
    Learning,
    Maintenance,
    Research,
}

/// Task status following state machine pattern
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Backlog,
    Ready,
    InProgress,
    Blocked,
    Review,
    Done,
    Cancelled,
}

impl TaskStatus {
    /// Valid transitions based on state machine constraints
    pub fn can_transition_to(&self, new_status: &TaskStatus) -> bool {
        match (self, new_status) {
            (TaskStatus::Backlog, TaskStatus::Ready) => true,
            (TaskStatus::Backlog, TaskStatus::Cancelled) => true,
            (TaskStatus::Ready, TaskStatus::InProgress) => true,
            (TaskStatus::Ready, TaskStatus::Backlog) => true,
            (TaskStatus::Ready, TaskStatus::Cancelled) => true,
            (TaskStatus::InProgress, TaskStatus::Blocked) => true,
            (TaskStatus::InProgress, TaskStatus::Review) => true,
            (TaskStatus::InProgress, TaskStatus::Done) => true,
            (TaskStatus::InProgress, TaskStatus::Cancelled) => true,
            (TaskStatus::Blocked, TaskStatus::InProgress) => true,
            (TaskStatus::Blocked, TaskStatus::Cancelled) => true,
            (TaskStatus::Review, TaskStatus::InProgress) => true,
            (TaskStatus::Review, TaskStatus::Done) => true,
            (TaskStatus::Review, TaskStatus::Cancelled) => true,
            _ => false,
        }
    }
    
    /// Check if the status indicates the task is actively being worked on
    pub fn is_active_work(&self) -> bool {
        matches!(self, TaskStatus::InProgress | TaskStatus::Review)
    }
    
    /// Check if the status indicates the task is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskStatus::Done | TaskStatus::Cancelled)
    }
    
    /// Check if the status indicates the task is blocked or waiting
    pub fn is_blocked(&self) -> bool {
        matches!(self, TaskStatus::Blocked)
    }
    
    /// Check if the status indicates the task is ready for work
    pub fn is_ready_for_work(&self) -> bool {
        matches!(self, TaskStatus::Ready)
    }
    
    /// Get the next logical status in the workflow
    pub fn next_status(&self) -> Option<TaskStatus> {
        match self {
            TaskStatus::Backlog => Some(TaskStatus::Ready),
            TaskStatus::Ready => Some(TaskStatus::InProgress),
            TaskStatus::InProgress => Some(TaskStatus::Review),
            TaskStatus::Review => Some(TaskStatus::Done),
            TaskStatus::Blocked => Some(TaskStatus::InProgress),
            TaskStatus::Done | TaskStatus::Cancelled => None,
        }
    }
    
    /// Get all valid next statuses from current status
    pub fn valid_next_statuses(&self) -> Vec<TaskStatus> {
        match self {
            TaskStatus::Backlog => vec![TaskStatus::Ready, TaskStatus::Cancelled],
            TaskStatus::Ready => vec![TaskStatus::InProgress, TaskStatus::Backlog, TaskStatus::Cancelled],
            TaskStatus::InProgress => vec![TaskStatus::Blocked, TaskStatus::Review, TaskStatus::Done, TaskStatus::Cancelled],
            TaskStatus::Blocked => vec![TaskStatus::InProgress, TaskStatus::Cancelled],
            TaskStatus::Review => vec![TaskStatus::InProgress, TaskStatus::Done, TaskStatus::Cancelled],
            TaskStatus::Done => vec![TaskStatus::Cancelled],
            TaskStatus::Cancelled => vec![],
        }
    }
    
    /// Get the priority order for status display (lower number = higher priority)
    pub fn display_priority(&self) -> u8 {
        match self {
            TaskStatus::Blocked => 0,      // Highest priority - needs attention
            TaskStatus::InProgress => 1,   // Active work
            TaskStatus::Review => 2,       // Needs review
            TaskStatus::Ready => 3,        // Ready to start
            TaskStatus::Backlog => 4,      // Planned work
            TaskStatus::Done => 5,         // Completed
            TaskStatus::Cancelled => 6,    // Lowest priority
        }
    }
}

/// Task priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Critical,
    High,
    Medium,
    Low,
    Wish,
}

/// Task complexity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskComplexity {
    Trivial,
    Simple,
    Medium,
    Complex,
    VeryComplex,
}

/// Task source origin
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskSource {
    Self_,
    Email,
    Meeting,
    AiSuggested,
    System,
    OtherPerson,
}

/// Task visibility levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskVisibility {
    Private,
    Shared,
    Public,
}

/// Success criterion for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    pub criterion: String,
    pub measurable: bool,
    pub verification_method: String,
}

/// Recurrence pattern for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecurrence {
    pub pattern: String, // "daily", "weekly", "monthly", "custom"
    pub interval: u32,
    pub end_date: Option<DateTime<Utc>>,
}

/// File attachment for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAttachment {
    pub name: String,
    pub url: String,
    pub attachment_type: String,
    pub size: u64,
    pub uploaded_at: DateTime<Utc>,
}

/// Core Task domain model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Human-readable ID based on project/context (e.g., "PROJ1-T042")
    pub id: String,
    
    /// System UUID for internal references
    pub uuid: String,
    
    /// Core properties
    pub name: String,
    pub description: Option<String>,
    pub context: TaskContext,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    
    /// Detailed information
    pub implementation_details: Option<String>,
    pub success_criteria: Vec<SuccessCriterion>,
    pub test_strategy: Option<String>,
    
    /// Time management
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_date: Option<DateTime<Utc>>,
    
    /// Additional metadata
    pub complexity: TaskComplexity,
    pub recurrence: Option<TaskRecurrence>,
    pub source: TaskSource,
    pub visibility: TaskVisibility,
    pub attachments: Vec<TaskAttachment>,
    
    /// Custom properties for extensibility
    pub custom_properties: HashMap<String, serde_json::Value>,
}

impl Task {
    /// Create a new task with minimal required fields
    pub fn new(id: String, name: String, context: TaskContext) -> Self {
        let now = Utc::now();
        Self {
            id,
            uuid: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            context,
            status: TaskStatus::Backlog,
            priority: TaskPriority::Medium,
            implementation_details: None,
            success_criteria: Vec::new(),
            test_strategy: None,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            due_date: None,
            estimated_date: None,
            complexity: TaskComplexity::Medium,
            recurrence: None,
            source: TaskSource::Self_,
            visibility: TaskVisibility::Private,
            attachments: Vec::new(),
            custom_properties: HashMap::new(),
        }
    }
    
    /// Builder pattern for complex task creation
    pub fn builder(id: String, name: String, context: TaskContext) -> TaskBuilder {
        TaskBuilder::new(id, name, context)
    }
    
    /// Update task status with validation
    pub fn update_status(&mut self, new_status: TaskStatus) -> TylResult<()> {
        if !self.status.can_transition_to(&new_status) {
            return Err(TylError::validation(
                "status",
                format!("Cannot transition from {:?} to {:?}", self.status, new_status)
            ));
        }
        
        self.status = new_status;
        self.updated_at = Utc::now();
        
        // Set timestamps based on status
        match self.status {
            TaskStatus::InProgress if self.started_at.is_none() => {
                self.started_at = Some(Utc::now());
            }
            TaskStatus::Done => {
                self.completed_at = Some(Utc::now());
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Check if task is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            return due_date < Utc::now() && !matches!(self.status, TaskStatus::Done | TaskStatus::Cancelled);
        }
        false
    }
    
    /// Check if task is actionable (ready to work on)
    pub fn is_actionable(&self) -> bool {
        matches!(self.status, TaskStatus::Ready | TaskStatus::InProgress)
    }
}

/// Builder for Task creation
pub struct TaskBuilder {
    task: Task,
}

impl TaskBuilder {
    fn new(id: String, name: String, context: TaskContext) -> Self {
        Self {
            task: Task::new(id, name, context),
        }
    }
    
    pub fn description(mut self, description: String) -> Self {
        self.task.description = Some(description);
        self
    }
    
    pub fn priority(mut self, priority: TaskPriority) -> Self {
        self.task.priority = priority;
        self
    }
    
    pub fn complexity(mut self, complexity: TaskComplexity) -> Self {
        self.task.complexity = complexity;
        self
    }
    
    pub fn due_date(mut self, due_date: DateTime<Utc>) -> Self {
        self.task.due_date = Some(due_date);
        self
    }
    
    pub fn implementation_details(mut self, details: String) -> Self {
        self.task.implementation_details = Some(details);
        self
    }
    
    pub fn add_success_criterion(mut self, criterion: SuccessCriterion) -> Self {
        self.task.success_criteria.push(criterion);
        self
    }
    
    pub fn source(mut self, source: TaskSource) -> Self {
        self.task.source = source;
        self
    }
    
    pub fn visibility(mut self, visibility: TaskVisibility) -> Self {
        self.task.visibility = visibility;
        self
    }
    
    pub fn recurrence(mut self, recurrence: TaskRecurrence) -> Self {
        self.task.recurrence = Some(recurrence);
        self
    }
    
    pub fn add_custom_property(mut self, key: String, value: serde_json::Value) -> Self {
        self.task.custom_properties.insert(key, value);
        self
    }
    
    pub fn build(self) -> Task {
        self.task
    }
}

/// Dependency relationship types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    Blocks,
    Requires,
    RelatedTo,
    Duplicates,
}

/// Task dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependency {
    pub id: String,
    pub from_task_id: String,
    pub to_task_id: String,
    pub dependency_type: DependencyType,
    pub is_hard_dependency: bool,
    pub delay_days: u32,
    pub created_at: DateTime<Utc>,
    pub properties: HashMap<String, serde_json::Value>,
}

impl TaskDependency {
    pub fn new(
        from_task_id: String,
        to_task_id: String,
        dependency_type: DependencyType,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from_task_id,
            to_task_id,
            dependency_type,
            is_hard_dependency: true,
            delay_days: 0,
            created_at: Utc::now(),
            properties: HashMap::new(),
        }
    }
}

/// Project entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub code: String, // Used for task ID generation
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    pub fn new(id: String, code: String, name: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            code,
            name,
            description: None,
            status: "active".to_string(),
            start_date: None,
            end_date: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(id: String, username: String, email: String, full_name: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            username,
            email,
            full_name,
            role: "user".to_string(),
            avatar_url: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Tag entity for categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub category: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Tag {
    pub fn new(id: String, name: String, category: String) -> Self {
        Self {
            id,
            name,
            color: "#007acc".to_string(), // Default blue
            category,
            description: None,
            created_at: Utc::now(),
        }
    }
}

/// Comment on tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author_id: String,
}

impl Comment {
    pub fn new(id: String, content: String, author_id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            content,
            created_at: now,
            updated_at: now,
            author_id,
        }
    }
}

/// Milestone entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Milestone {
    pub fn new(id: String, name: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description: None,
            due_date: None,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Context entity for task organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub icon: Option<String>,
    pub is_active: bool,
    pub purpose: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Context {
    pub fn new(id: String, name: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description: None,
            color: "#28a745".to_string(), // Default green
            icon: None,
            is_active: true,
            purpose: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Request DTO for creating a new task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub context: TaskContext,
    pub priority: TaskPriority,
    pub complexity: TaskComplexity,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_date: Option<DateTime<Utc>>,
    pub implementation_details: Option<String>,
    pub success_criteria: Vec<SuccessCriterion>,
    pub test_strategy: Option<String>,
    pub source: TaskSource,
    pub visibility: TaskVisibility,
    pub recurrence: Option<TaskRecurrence>,
    pub custom_properties: HashMap<String, serde_json::Value>,
    pub assigned_user_id: Option<String>,
    pub project_id: Option<String>,
}

/// Request DTO for updating an existing task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,
    pub complexity: Option<TaskComplexity>,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_date: Option<DateTime<Utc>>,
    pub implementation_details: Option<String>,
    pub success_criteria: Option<Vec<SuccessCriterion>>,
    pub test_strategy: Option<String>,
    pub visibility: Option<TaskVisibility>,
    pub custom_properties: Option<HashMap<String, serde_json::Value>>,
}

/// Detailed response DTO for task operations with relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDetailResponse {
    pub task: Task,
    pub dependencies: Option<Vec<TaskDependency>>,
    pub subtasks: Option<Vec<Task>>,
    pub comments_count: Option<u32>,
    pub computed_properties: Option<HashMap<String, serde_json::Value>>,
}

impl TaskDetailResponse {
    pub fn new(task: Task) -> Self {
        Self {
            task,
            dependencies: None,
            subtasks: None,
            comments_count: None,
            computed_properties: None,
        }
    }
    
    pub fn with_dependencies(mut self, dependencies: Vec<TaskDependency>) -> Self {
        self.dependencies = Some(dependencies);
        self
    }
    
    pub fn with_subtasks(mut self, subtasks: Vec<Task>) -> Self {
        self.subtasks = Some(subtasks);
        self
    }
    
    pub fn with_comments_count(mut self, count: u32) -> Self {
        self.comments_count = Some(count);
        self
    }
    
    pub fn with_computed_properties(mut self, properties: HashMap<String, serde_json::Value>) -> Self {
        self.computed_properties = Some(properties);
        self
    }
}

/// Filter options for listing tasks
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskFilter {
    pub context: Option<Vec<TaskContext>>,
    pub status: Option<Vec<TaskStatus>>,
    pub priority: Option<Vec<TaskPriority>>,
    pub complexity: Option<Vec<TaskComplexity>>,
    pub assigned_user_id: Option<String>,
    pub project_id: Option<String>,
    pub due_date_from: Option<DateTime<Utc>>,
    pub due_date_to: Option<DateTime<Utc>>,
    pub due_before: Option<DateTime<Utc>>,
    pub due_after: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub search_text: Option<String>,
    pub tags: Option<Vec<String>>,
    pub has_dependencies: Option<bool>,
    pub is_overdue: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Request DTO for creating a new project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "PROJ1-T001".to_string(),
            "Implement user authentication".to_string(),
            TaskContext::Work,
        );
        
        assert_eq!(task.id, "PROJ1-T001");
        assert_eq!(task.name, "Implement user authentication");
        assert_eq!(task.context, TaskContext::Work);
        assert_eq!(task.status, TaskStatus::Backlog);
        assert_eq!(task.priority, TaskPriority::Medium);
    }
    
    #[test]
    fn test_task_status_transitions() {
        let mut task = Task::new(
            "PROJ1-T001".to_string(),
            "Test task".to_string(),
            TaskContext::Work,
        );
        
        // Valid transition: Backlog -> Ready
        assert!(task.update_status(TaskStatus::Ready).is_ok());
        assert_eq!(task.status, TaskStatus::Ready);
        
        // Valid transition: Ready -> InProgress
        assert!(task.update_status(TaskStatus::InProgress).is_ok());
        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(task.started_at.is_some());
        
        // Invalid transition: InProgress -> Backlog
        assert!(task.update_status(TaskStatus::Backlog).is_err());
        
        // Valid transition: InProgress -> Done
        assert!(task.update_status(TaskStatus::Done).is_ok());
        assert_eq!(task.status, TaskStatus::Done);
        assert!(task.completed_at.is_some());
    }
    
    #[test]
    fn test_task_builder() {
        let criterion = SuccessCriterion {
            criterion: "All tests pass".to_string(),
            measurable: true,
            verification_method: "Automated test suite".to_string(),
        };
        
        let task = Task::builder(
            "PROJ1-T002".to_string(),
            "Complex feature".to_string(),
            TaskContext::Work,
        )
        .description("A complex feature implementation".to_string())
        .priority(TaskPriority::High)
        .complexity(TaskComplexity::Complex)
        .add_success_criterion(criterion)
        .source(TaskSource::Meeting)
        .visibility(TaskVisibility::Shared)
        .add_custom_property("epic".to_string(), serde_json::json!("USER_AUTH"))
        .build();
        
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.complexity, TaskComplexity::Complex);
        assert_eq!(task.success_criteria.len(), 1);
        assert_eq!(task.source, TaskSource::Meeting);
        assert_eq!(task.visibility, TaskVisibility::Shared);
        assert!(task.custom_properties.contains_key("epic"));
    }
    
    #[test]
    fn test_task_overdue() {
        let mut task = Task::new(
            "PROJ1-T003".to_string(),
            "Overdue task".to_string(),
            TaskContext::Work,
        );
        
        // Set due date in the past
        task.due_date = Some(Utc::now() - chrono::Duration::days(1));
        assert!(task.is_overdue());
        
        // Complete the task - should not be overdue
        task.status = TaskStatus::Done;
        task.completed_at = Some(Utc::now());
        assert!(!task.is_overdue());
    }
    
    #[test]
    fn test_task_dependency() {
        let dep = TaskDependency::new(
            "PROJ1-T001".to_string(),
            "PROJ1-T002".to_string(),
            DependencyType::Blocks,
        );
        
        assert_eq!(dep.from_task_id, "PROJ1-T001");
        assert_eq!(dep.to_task_id, "PROJ1-T002");
        assert_eq!(dep.dependency_type, DependencyType::Blocks);
        assert!(dep.is_hard_dependency);
    }
}