//! Domain services for task management
//!
//! This module contains the business logic services that orchestrate domain operations.
//! Services handle complex business rules, coordinating between repositories and enforcing
//! domain constraints.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tyl_errors::{TylError, TylResult};

use super::models::*;
use super::queries::{DependencyCycle, CycleSeverity, DependencyBreakSuggestion};

/// Main task service trait - defines the core business operations
#[async_trait]
pub trait TaskService {
    // Task CRUD operations
    async fn create_task(&self, request: CreateTaskRequest) -> TylResult<Task>;
    async fn get_task_by_id(&self, id: &str) -> TylResult<Option<Task>>;
    async fn update_task(&self, id: &str, request: UpdateTaskRequest) -> TylResult<Task>;
    async fn delete_task(&self, id: &str) -> TylResult<()>;
    async fn list_tasks(&self, filter: TaskFilter) -> TylResult<Vec<Task>>;
    
    // Task relationships
    async fn add_task_dependency(
        &self,
        from_task_id: &str,
        to_task_id: &str,
        dependency_type: DependencyType,
    ) -> TylResult<TaskDependency>;
    async fn remove_task_dependency(&self, dependency_id: &str) -> TylResult<()>;
    async fn get_task_dependencies(&self, task_id: &str) -> TylResult<Vec<TaskDependency>>;
    async fn get_blocked_tasks(&self, task_id: &str) -> TylResult<Vec<Task>>;
    
    // Task hierarchy
    async fn add_subtask(&self, parent_id: &str, child_id: &str) -> TylResult<()>;
    async fn remove_subtask(&self, parent_id: &str, child_id: &str) -> TylResult<()>;
    async fn get_subtasks(&self, parent_id: &str) -> TylResult<Vec<Task>>;
    async fn get_parent_task(&self, child_id: &str) -> TylResult<Option<Task>>;
    
    // Task status management
    async fn transition_task_status(&self, task_id: &str, new_status: TaskStatus) -> TylResult<Task>;
    
    // Task assignment
    async fn assign_task(&self, task_id: &str, user_id: &str, role: &str) -> TylResult<()>;
    async fn unassign_task(&self, task_id: &str, user_id: &str) -> TylResult<()>;
    async fn get_assigned_tasks(&self, user_id: &str) -> TylResult<Vec<Task>>;
    
    // Project management
    async fn create_project(&self, request: CreateProjectRequest) -> TylResult<Project>;
    async fn add_task_to_project(&self, task_id: &str, project_id: &str) -> TylResult<()>;
    async fn get_project_tasks(&self, project_id: &str) -> TylResult<Vec<Task>>;
    
    // Analytics and queries
    async fn get_task_analytics(&self, task_id: &str) -> TylResult<TaskAnalytics>;
    async fn get_critical_path(&self, project_id: &str) -> TylResult<Vec<Task>>;
    async fn detect_circular_dependencies(&self) -> TylResult<Vec<Vec<String>>>;
    async fn get_detailed_circular_dependencies(&self) -> TylResult<Vec<DependencyCycle>>;
    async fn get_actionable_tasks(&self, user_id: &str) -> TylResult<Vec<Task>>;
    async fn get_overdue_tasks(&self) -> TylResult<Vec<Task>>;
}


/// Analytics data for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAnalytics {
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

/// Repository trait for task persistence
#[async_trait]
pub trait TaskRepository {
    async fn save_task(&self, task: &Task) -> TylResult<()>;
    async fn find_task_by_id(&self, id: &str) -> TylResult<Option<Task>>;
    async fn find_tasks_by_filter(&self, filter: &TaskFilter) -> TylResult<Vec<Task>>;
    async fn delete_task(&self, id: &str) -> TylResult<()>;
    
    // Relationship operations
    async fn save_dependency(&self, dependency: &TaskDependency) -> TylResult<()>;
    async fn delete_dependency(&self, dependency_id: &str) -> TylResult<()>;
    async fn find_dependencies_by_task(&self, task_id: &str) -> TylResult<Vec<TaskDependency>>;
    async fn find_blocking_tasks(&self, task_id: &str) -> TylResult<Vec<Task>>;
    
    // Hierarchy operations
    async fn add_parent_child_relationship(&self, parent_id: &str, child_id: &str) -> TylResult<()>;
    async fn remove_parent_child_relationship(&self, parent_id: &str, child_id: &str) -> TylResult<()>;
    async fn find_children(&self, parent_id: &str) -> TylResult<Vec<Task>>;
    async fn find_parent(&self, child_id: &str) -> TylResult<Option<Task>>;
    
    // Assignment operations
    async fn assign_user_to_task(&self, task_id: &str, user_id: &str, role: &str) -> TylResult<()>;
    async fn unassign_user_from_task(&self, task_id: &str, user_id: &str) -> TylResult<()>;
    async fn find_assigned_tasks(&self, user_id: &str) -> TylResult<Vec<Task>>;
    
    // Project operations
    async fn save_project(&self, project: &Project) -> TylResult<()>;
    async fn add_task_to_project(&self, task_id: &str, project_id: &str) -> TylResult<()>;
    async fn find_project_tasks(&self, project_id: &str) -> TylResult<Vec<Task>>;
    
    // Analytics operations
    async fn calculate_completion_percentage(&self, task_id: &str) -> TylResult<f64>;
    async fn find_critical_path(&self, project_id: &str) -> TylResult<Vec<Task>>;
    async fn detect_circular_dependencies(&self) -> TylResult<Vec<Vec<String>>>;
}

/// Domain service implementation coordinating business logic
pub struct TaskDomainService<R: TaskRepository> {
    repository: R,
}

impl<R: TaskRepository> TaskDomainService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    
    /// Validate that a task status transition is allowed
    fn validate_status_transition(&self, current: &TaskStatus, new: &TaskStatus) -> TylResult<()> {
        // Basic state machine validation
        if !current.can_transition_to(new) {
            return Err(TylError::validation(
                "status",
                format!("Cannot transition from {:?} to {:?}", current, new)
            ));
        }
        
        // Additional business rule validations
        self.validate_status_transition_business_rules(current, new)
    }
    
    /// Validate advanced business rules for status transitions
    fn validate_status_transition_business_rules(&self, current: &TaskStatus, new: &TaskStatus) -> TylResult<()> {
        match (current, new) {
            // Cannot go directly from Backlog to InProgress without being Ready first
            (TaskStatus::Backlog, TaskStatus::InProgress) => {
                Err(TylError::validation(
                    "status",
                    "Tasks must transition through Ready state before starting work".to_string()
                ))
            },
            
            // Cannot go directly from Backlog to Review
            (TaskStatus::Backlog, TaskStatus::Review) => {
                Err(TylError::validation(
                    "status",
                    "Tasks must be worked on before review".to_string()
                ))
            },
            
            // Cannot go directly from Ready to Done without work
            (TaskStatus::Ready, TaskStatus::Done) => {
                Err(TylError::validation(
                    "status",
                    "Tasks must show progress before completion".to_string()
                ))
            },
            
            // Cannot go from Done back to any state except for special cases
            (TaskStatus::Done, status) if !matches!(status, TaskStatus::Cancelled) => {
                Err(TylError::validation(
                    "status",
                    "Completed tasks can only be cancelled, not reopened".to_string()
                ))
            },
            
            // Cannot go from Cancelled to any other state
            (TaskStatus::Cancelled, _) => {
                Err(TylError::validation(
                    "status",
                    "Cancelled tasks cannot be reopened. Create a new task instead".to_string()
                ))
            },
            
            // All other transitions are valid if they passed the basic state machine check
            _ => Ok(())
        }
    }
    
    /// Generate the next task ID for a project
    async fn generate_task_id(&self, project_code: &str) -> TylResult<String> {
        // This would typically query the database for the next sequence number
        // For now, we'll use a timestamp-based approach
        let timestamp = Utc::now().timestamp_millis();
        Ok(format!("{}-T{}", project_code, timestamp % 100000))
    }
    
    /// Check for circular dependencies before adding a new dependency
    async fn check_circular_dependency(&self, from_task_id: &str, to_task_id: &str) -> TylResult<()> {
        // Check for self-dependency
        if from_task_id == to_task_id {
            return Err(TylError::validation(
                "dependency",
                "A task cannot depend on itself"
            ));
        }
        
        // Check if adding this dependency would create a cycle
        if self.would_create_cycle(from_task_id, to_task_id).await? {
            return Err(TylError::validation(
                "dependency",
                format!("Adding dependency from {} to {} would create a circular dependency", 
                        from_task_id, to_task_id)
            ));
        }
        
        Ok(())
    }
    
    /// Check if adding a dependency would create a circular dependency
    async fn would_create_cycle(&self, from_task_id: &str, to_task_id: &str) -> TylResult<bool> {
        // Use depth-first search to check if there's already a path from to_task_id to from_task_id
        // If such a path exists, adding from_task_id -> to_task_id would create a cycle
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![to_task_id.to_string()];
        
        while let Some(current_task) = stack.pop() {
            if current_task == from_task_id {
                return Ok(true); // Found a path back to the starting task - would create cycle
            }
            
            if visited.contains(&current_task) {
                continue; // Already visited this task
            }
            
            visited.insert(current_task.clone());
            
            // Get all tasks that the current task depends on
            let dependencies = self.repository.find_dependencies_by_task(&current_task).await?;
            for dep in dependencies {
                if !visited.contains(&dep.to_task_id) {
                    stack.push(dep.to_task_id);
                }
            }
        }
        
        Ok(false) // No cycle would be created
    }
    
    /// Advanced circular dependency detection with detailed path information
    async fn detect_all_circular_dependencies(&self) -> TylResult<Vec<DependencyCycle>> {
        let mut cycles = Vec::new();
        let mut global_visited = std::collections::HashSet::new();
        
        // Get all tasks to check for cycles
        let all_tasks = self.repository.find_tasks_by_filter(&TaskFilter::default()).await?;
        
        for task in all_tasks {
            if global_visited.contains(&task.id) {
                continue;
            }
            
            // Perform DFS from this task to detect cycles
            let mut visited = std::collections::HashSet::new();
            let mut rec_stack = std::collections::HashSet::new();
            let mut path = Vec::new();
            
            if self.dfs_detect_cycle(&task.id, &mut visited, &mut rec_stack, &mut path, &mut cycles).await? {
                // Mark all tasks in found cycles as globally visited
                for cycle in &cycles {
                    for task_id in &cycle.tasks_in_cycle {
                        global_visited.insert(task_id.clone());
                    }
                }
            }
        }
        
        Ok(cycles)
    }
    
    /// Depth-first search to detect cycles in the dependency graph (iterative version)
    async fn dfs_detect_cycle(
        &self,
        start_task_id: &str,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<DependencyCycle>
    ) -> TylResult<bool> {
        use std::collections::VecDeque;
        
        // Stack for iterative DFS: (task_id, is_backtrack)
        let mut stack: VecDeque<(String, bool)> = VecDeque::new();
        stack.push_back((start_task_id.to_string(), false));
        
        while let Some((task_id, is_backtrack)) = stack.pop_back() {
            if is_backtrack {
                // Backtracking phase
                rec_stack.remove(&task_id);
                if let Some(last) = path.last() {
                    if last == &task_id {
                        path.pop();
                    }
                }
                continue;
            }
            
            // Forward phase
            if visited.contains(&task_id) && !rec_stack.contains(&task_id) {
                continue; // Already processed this node
            }
            
            if rec_stack.contains(&task_id) {
                // Found a cycle! Extract the cycle path
                if let Some(cycle_start_idx) = path.iter().position(|id| id == &task_id) {
                    let mut cycle_path: Vec<String> = path[cycle_start_idx..].to_vec();
                    cycle_path.push(task_id.clone()); // Complete the cycle
                    
                    let cycle_info = DependencyCycle {
                        cycle_id: uuid::Uuid::new_v4().to_string(),
                        tasks_in_cycle: cycle_path.clone(),
                        cycle_length: cycle_path.len() as u32,
                        severity: self.calculate_cycle_severity(&cycle_path).await?,
                        suggested_breaks: self.suggest_dependency_breaks(&cycle_path).await?,
                    };
                    
                    cycles.push(cycle_info);
                    return Ok(true);
                }
                continue;
            }
            
            visited.insert(task_id.clone());
            rec_stack.insert(task_id.clone());
            path.push(task_id.clone());
            
            // Add backtrack marker
            stack.push_back((task_id.clone(), true));
            
            // Get dependencies for this task and add them to stack
            let dependencies = self.repository.find_dependencies_by_task(&task_id).await?;
            for dep in dependencies {
                let dep_task_id = dep.to_task_id;
                if !visited.contains(&dep_task_id) || rec_stack.contains(&dep_task_id) {
                    stack.push_back((dep_task_id, false));
                }
            }
        }
        
        Ok(false)
    }
    
    /// Calculate the severity of a circular dependency
    async fn calculate_cycle_severity(&self, cycle_path: &[String]) -> TylResult<CycleSeverity> {
        let mut max_priority = 0;
        let mut has_critical_path_tasks = false;
        
        for task_id in cycle_path {
            if let Some(task) = self.repository.find_task_by_id(task_id).await? {
                // Check priority impact
                let priority_score = match task.priority {
                    crate::domain::TaskPriority::Critical => 4,
                    crate::domain::TaskPriority::High => 3,
                    crate::domain::TaskPriority::Medium => 2,
                    crate::domain::TaskPriority::Low => 1,
                    crate::domain::TaskPriority::Wish => 0,
                };
                max_priority = max_priority.max(priority_score);
                
                // Check if task is on critical path (simplified check)
                if matches!(task.priority, crate::domain::TaskPriority::Critical | crate::domain::TaskPriority::High) {
                    has_critical_path_tasks = true;
                }
            }
        }
        
        let severity = match (cycle_path.len(), max_priority, has_critical_path_tasks) {
            (len, 4, _) if len > 5 => CycleSeverity::Critical,
            (len, p, true) if len > 3 && p >= 3 => CycleSeverity::High,
            (len, p, _) if len > 2 && p >= 2 => CycleSeverity::Medium,
            _ => CycleSeverity::Low,
        };
        
        Ok(severity)
    }
    
    /// Find all tasks affected by a circular dependency
    async fn find_affected_tasks(&self, cycle_path: &[String]) -> TylResult<Vec<String>> {
        let mut affected = std::collections::HashSet::new();
        
        for task_id in cycle_path {
            affected.insert(task_id.clone());
            
            // Find tasks that depend on cycle tasks
            let blocking_tasks = self.repository.find_blocking_tasks(task_id).await?;
            for blocked_task in blocking_tasks {
                affected.insert(blocked_task.id);
            }
        }
        
        Ok(affected.into_iter().collect())
    }
    
    /// Suggest which dependencies to break to resolve circular dependency
    async fn suggest_dependency_breaks(&self, cycle_path: &[String]) -> TylResult<Vec<DependencyBreakSuggestion>> {
        let mut suggestions = Vec::new();
        
        for i in 0..cycle_path.len() {
            let from_task = &cycle_path[i];
            let to_task = &cycle_path[(i + 1) % cycle_path.len()];
            
            // Calculate impact of breaking this dependency
            let impact_score = self.calculate_break_impact(from_task, to_task).await?;
            
            let suggestion = DependencyBreakSuggestion {
                from_task: from_task.clone(),
                to_task: to_task.clone(),
                reason: self.generate_break_reason(from_task, to_task).await?,
                impact_score,
            };
            
            suggestions.push(suggestion);
        }
        
        // Sort by impact score (lower = better to break)
        suggestions.sort_by(|a, b| a.impact_score.partial_cmp(&b.impact_score).unwrap());
        
        Ok(suggestions)
    }
    
    /// Calculate the impact of breaking a specific dependency
    async fn calculate_break_impact(&self, from_task_id: &str, to_task_id: &str) -> TylResult<f64> {
        let mut impact = 0.0;
        
        // Get task information
        if let (Some(from_task), Some(to_task)) = (
            self.repository.find_task_by_id(from_task_id).await?,
            self.repository.find_task_by_id(to_task_id).await?
        ) {
            // Higher priority tasks have higher break cost
            impact += match from_task.priority {
                crate::domain::TaskPriority::Critical => 10.0,
                crate::domain::TaskPriority::High => 7.0,
                crate::domain::TaskPriority::Medium => 5.0,
                crate::domain::TaskPriority::Low => 3.0,
                crate::domain::TaskPriority::Wish => 1.0,
            };
            
            // Tasks with more dependencies have higher break cost
            let from_deps = self.repository.find_dependencies_by_task(from_task_id).await?;
            impact += from_deps.len() as f64 * 0.5;
            
            // Active tasks have higher break cost
            if from_task.status.is_active_work() {
                impact += 5.0;
            }
        }
        
        Ok(impact)
    }
    
    /// Generate a human-readable reason for breaking a dependency
    async fn generate_break_reason(&self, from_task_id: &str, to_task_id: &str) -> TylResult<String> {
        if let (Some(from_task), Some(to_task)) = (
            self.repository.find_task_by_id(from_task_id).await?,
            self.repository.find_task_by_id(to_task_id).await?
        ) {
            let reason = match (from_task.priority, to_task.priority) {
                (crate::domain::TaskPriority::Low, crate::domain::TaskPriority::High) => {
                    format!("Low priority task '{}' blocking high priority task '{}'", from_task.name, to_task.name)
                },
                _ if from_task.status.is_terminal() => {
                    format!("Completed task '{}' has unnecessary dependency", from_task.name)
                },
                _ => {
                    format!("Break dependency from '{}' to '{}' (lowest impact)", from_task.name, to_task.name)
                }
            };
            Ok(reason)
        } else {
            Ok(format!("Break dependency from {} to {}", from_task_id, to_task_id))
        }
    }
    
    /// Calculate task analytics
    async fn calculate_task_analytics(&self, task_id: &str) -> TylResult<TaskAnalytics> {
        let completion_percentage = self.repository.calculate_completion_percentage(task_id).await?;
        let dependencies = self.repository.find_dependencies_by_task(task_id).await?;
        let blocking_tasks = self.repository.find_blocking_tasks(task_id).await?;
        let subtasks = self.repository.find_children(task_id).await?;
        
        let completed_subtasks = subtasks.iter()
            .filter(|t| t.status == TaskStatus::Done)
            .count() as u32;
        
        let blocking_count = blocking_tasks.len() as u32;
        let blocked_by_count = dependencies.iter()
            .filter(|d| d.dependency_type == DependencyType::Blocks)
            .count() as u32;
        
        Ok(TaskAnalytics {
            task_id: task_id.to_string(),
            completion_percentage,
            blocking_count,
            blocked_by_count,
            subtask_count: subtasks.len() as u32,
            completed_subtasks,
            is_on_critical_path: false, // Would be calculated via graph algorithms
            estimated_completion_date: None, // Would be calculated based on dependencies
            time_to_completion_days: None,
            dependency_chain_length: dependencies.len() as u32,
            priority_score: 0.0, // Would be calculated based on priority algorithm
        })
    }
}

#[async_trait]
impl<R: TaskRepository + Send + Sync> TaskService for TaskDomainService<R> {
    async fn create_task(&self, request: CreateTaskRequest) -> TylResult<Task> {
        // Validate the request
        if request.name.trim().is_empty() {
            return Err(TylError::validation("name", "Task name cannot be empty"));
        }
        
        // Create the task using the builder pattern
        let mut task_builder = Task::builder(request.id, request.name, request.context)
            .priority(request.priority)
            .complexity(request.complexity)
            .source(request.source)
            .visibility(request.visibility);
        
        if let Some(description) = request.description {
            task_builder = task_builder.description(description);
        }
        
        if let Some(due_date) = request.due_date {
            task_builder = task_builder.due_date(due_date);
        }
        
        if let Some(details) = request.implementation_details {
            task_builder = task_builder.implementation_details(details);
        }
        
        for criterion in request.success_criteria {
            task_builder = task_builder.add_success_criterion(criterion);
        }
        
        if let Some(recurrence) = request.recurrence {
            task_builder = task_builder.recurrence(recurrence);
        }
        
        for (key, value) in request.custom_properties {
            task_builder = task_builder.add_custom_property(key, value);
        }
        
        let task = task_builder.build();
        
        // Save the task
        self.repository.save_task(&task).await?;
        
        // Handle assignment if specified
        if let Some(user_id) = request.assigned_user_id {
            self.repository.assign_user_to_task(&task.id, &user_id, "owner").await?;
        }
        
        // Handle project assignment if specified
        if let Some(project_id) = request.project_id {
            self.repository.add_task_to_project(&task.id, &project_id).await?;
        }
        
        Ok(task)
    }
    
    async fn get_task_by_id(&self, id: &str) -> TylResult<Option<Task>> {
        self.repository.find_task_by_id(id).await
    }
    
    async fn update_task(&self, id: &str, request: UpdateTaskRequest) -> TylResult<Task> {
        let mut task = self.repository.find_task_by_id(id).await?
            .ok_or_else(|| TylError::not_found("task", id))?;
        
        // Apply updates
        if let Some(name) = request.name {
            if name.trim().is_empty() {
                return Err(TylError::validation("name", "Task name cannot be empty"));
            }
            task.name = name;
        }
        
        if let Some(description) = request.description {
            task.description = Some(description);
        }
        
        if let Some(priority) = request.priority {
            task.priority = priority;
        }
        
        if let Some(complexity) = request.complexity {
            task.complexity = complexity;
        }
        
        if let Some(due_date) = request.due_date {
            task.due_date = Some(due_date);
        }
        
        if let Some(estimated_date) = request.estimated_date {
            task.estimated_date = Some(estimated_date);
        }
        
        if let Some(details) = request.implementation_details {
            task.implementation_details = Some(details);
        }
        
        if let Some(criteria) = request.success_criteria {
            task.success_criteria = criteria;
        }
        
        if let Some(test_strategy) = request.test_strategy {
            task.test_strategy = Some(test_strategy);
        }
        
        if let Some(visibility) = request.visibility {
            task.visibility = visibility;
        }
        
        if let Some(properties) = request.custom_properties {
            task.custom_properties = properties;
        }
        
        task.updated_at = Utc::now();
        
        // Save the updated task
        self.repository.save_task(&task).await?;
        
        Ok(task)
    }
    
    async fn delete_task(&self, id: &str) -> TylResult<()> {
        // Check if task exists
        if self.repository.find_task_by_id(id).await?.is_none() {
            return Err(TylError::not_found("task", id));
        }
        
        // Check for dependencies that would be broken
        let dependencies = self.repository.find_dependencies_by_task(id).await?;
        if !dependencies.is_empty() {
            return Err(TylError::validation(
                "dependencies",
                format!("Task {} has {} dependencies. Remove dependencies first.", id, dependencies.len())
            ));
        }
        
        self.repository.delete_task(id).await
    }
    
    async fn list_tasks(&self, filter: TaskFilter) -> TylResult<Vec<Task>> {
        self.repository.find_tasks_by_filter(&filter).await
    }
    
    async fn add_task_dependency(
        &self,
        from_task_id: &str,
        to_task_id: &str,
        dependency_type: DependencyType,
    ) -> TylResult<TaskDependency> {
        // Validate that both tasks exist
        if self.repository.find_task_by_id(from_task_id).await?.is_none() {
            return Err(TylError::not_found("task", from_task_id));
        }
        if self.repository.find_task_by_id(to_task_id).await?.is_none() {
            return Err(TylError::not_found("task", to_task_id));
        }
        
        // Check for circular dependencies
        self.check_circular_dependency(from_task_id, to_task_id).await?;
        
        let dependency = TaskDependency::new(
            from_task_id.to_string(),
            to_task_id.to_string(),
            dependency_type,
        );
        
        self.repository.save_dependency(&dependency).await?;
        Ok(dependency)
    }
    
    async fn remove_task_dependency(&self, dependency_id: &str) -> TylResult<()> {
        self.repository.delete_dependency(dependency_id).await
    }
    
    async fn get_task_dependencies(&self, task_id: &str) -> TylResult<Vec<TaskDependency>> {
        self.repository.find_dependencies_by_task(task_id).await
    }
    
    async fn get_blocked_tasks(&self, task_id: &str) -> TylResult<Vec<Task>> {
        self.repository.find_blocking_tasks(task_id).await
    }
    
    async fn add_subtask(&self, parent_id: &str, child_id: &str) -> TylResult<()> {
        // Validate that both tasks exist
        if self.repository.find_task_by_id(parent_id).await?.is_none() {
            return Err(TylError::not_found("task", parent_id));
        }
        if self.repository.find_task_by_id(child_id).await?.is_none() {
            return Err(TylError::not_found("task", child_id));
        }
        
        // Check for circular hierarchies
        if parent_id == child_id {
            return Err(TylError::validation("hierarchy", "A task cannot be a subtask of itself"));
        }
        
        self.repository.add_parent_child_relationship(parent_id, child_id).await
    }
    
    async fn remove_subtask(&self, parent_id: &str, child_id: &str) -> TylResult<()> {
        self.repository.remove_parent_child_relationship(parent_id, child_id).await
    }
    
    async fn get_subtasks(&self, parent_id: &str) -> TylResult<Vec<Task>> {
        self.repository.find_children(parent_id).await
    }
    
    async fn get_parent_task(&self, child_id: &str) -> TylResult<Option<Task>> {
        self.repository.find_parent(child_id).await
    }
    
    async fn transition_task_status(&self, task_id: &str, new_status: TaskStatus) -> TylResult<Task> {
        let mut task = self.repository.find_task_by_id(task_id).await?
            .ok_or_else(|| TylError::not_found("task", task_id))?;
        
        // Validate the transition
        self.validate_status_transition(&task.status, &new_status)?;
        
        // Additional context-specific validations
        self.validate_transition_prerequisites(&task, &new_status).await?;
        
        // Apply the status change
        task.update_status(new_status)?;
        
        // Save the updated task
        self.repository.save_task(&task).await?;
        
        Ok(task)
    }
    
    async fn assign_task(&self, task_id: &str, user_id: &str, role: &str) -> TylResult<()> {
        // Validate that task exists
        if self.repository.find_task_by_id(task_id).await?.is_none() {
            return Err(TylError::not_found("task", task_id));
        }
        
        self.repository.assign_user_to_task(task_id, user_id, role).await
    }
    
    async fn unassign_task(&self, task_id: &str, user_id: &str) -> TylResult<()> {
        self.repository.unassign_user_from_task(task_id, user_id).await
    }
    
    async fn get_assigned_tasks(&self, user_id: &str) -> TylResult<Vec<Task>> {
        self.repository.find_assigned_tasks(user_id).await
    }
    
    async fn create_project(&self, request: CreateProjectRequest) -> TylResult<Project> {
        let project = Project {
            id: request.id,
            code: request.code,
            name: request.name,
            description: request.description,
            status: "active".to_string(),
            start_date: request.start_date,
            end_date: request.end_date,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.repository.save_project(&project).await?;
        Ok(project)
    }
    
    async fn add_task_to_project(&self, task_id: &str, project_id: &str) -> TylResult<()> {
        // Validate that both task and project exist
        if self.repository.find_task_by_id(task_id).await?.is_none() {
            return Err(TylError::not_found("task", task_id));
        }
        
        self.repository.add_task_to_project(task_id, project_id).await
    }
    
    async fn get_project_tasks(&self, project_id: &str) -> TylResult<Vec<Task>> {
        self.repository.find_project_tasks(project_id).await
    }
    
    async fn get_task_analytics(&self, task_id: &str) -> TylResult<TaskAnalytics> {
        // Validate that task exists
        if self.repository.find_task_by_id(task_id).await?.is_none() {
            return Err(TylError::not_found("task", task_id));
        }
        
        self.calculate_task_analytics(task_id).await
    }
    
    async fn get_critical_path(&self, project_id: &str) -> TylResult<Vec<Task>> {
        self.repository.find_critical_path(project_id).await
    }
    
    async fn detect_circular_dependencies(&self) -> TylResult<Vec<Vec<String>>> {
        // Use the advanced detection and convert to simple format for compatibility
        let detailed_cycles = self.detect_all_circular_dependencies().await?;
        let simple_cycles = detailed_cycles
            .into_iter()
            .map(|cycle| cycle.tasks_in_cycle)
            .collect();
        Ok(simple_cycles)
    }
    
    async fn get_detailed_circular_dependencies(&self) -> TylResult<Vec<DependencyCycle>> {
        self.detect_all_circular_dependencies().await
    }
    
    async fn get_actionable_tasks(&self, user_id: &str) -> TylResult<Vec<Task>> {
        let filter = TaskFilter {
            assigned_user_id: Some(user_id.to_string()),
            status: Some(vec![TaskStatus::Ready, TaskStatus::InProgress]),
            ..Default::default()
        };
        
        let tasks = self.repository.find_tasks_by_filter(&filter).await?;
        
        // Filter to only actionable tasks (no blocking dependencies)
        let mut actionable = Vec::new();
        for task in tasks {
            let dependencies = self.repository.find_dependencies_by_task(&task.id).await?;
            let has_blocking_deps = dependencies.iter().any(|dep| {
                matches!(dep.dependency_type, DependencyType::Blocks | DependencyType::Requires)
            });
            
            if !has_blocking_deps && task.is_actionable() {
                actionable.push(task);
            }
        }
        
        Ok(actionable)
    }
    
    async fn get_overdue_tasks(&self) -> TylResult<Vec<Task>> {
        let filter = TaskFilter {
            status: Some(vec![TaskStatus::Ready, TaskStatus::InProgress, TaskStatus::Blocked]),
            is_overdue: Some(true),
            ..Default::default()
        };
        
        let tasks = self.repository.find_tasks_by_filter(&filter).await?;
        Ok(tasks.into_iter().filter(|t| t.is_overdue()).collect())
    }
}

/// Private helper methods for TaskDomainService
impl<R: TaskRepository + Send + Sync> TaskDomainService<R> {
    /// Validate prerequisites for specific status transitions (private helper)
    async fn validate_transition_prerequisites(&self, task: &Task, new_status: &TaskStatus) -> TylResult<()> {
        match new_status {
            TaskStatus::InProgress => {
                // Validate that task has an assignee before starting work
                // Check if task has no assignment through relationships
                // TODO: Query for task assignments through graph relationships
                if true { // Simplified for now
                    return Err(TylError::validation(
                        "status",
                        "Task must be assigned to a user before starting work".to_string()
                    ));
                }
            },
            
            TaskStatus::Review => {
                // Validate that task has implementation details before review
                if task.implementation_details.is_none() || task.implementation_details.as_ref().unwrap().trim().is_empty() {
                    return Err(TylError::validation(
                        "status",
                        "Task must have implementation details before review".to_string()
                    ));
                }
            },
            
            TaskStatus::Done => {
                // Validate that all success criteria are met
                if !task.success_criteria.is_empty() {
                    for criterion in &task.success_criteria {
                        // TODO: Add completion tracking to SuccessCriterion model
                        // For now, assume all criteria need completion
                        if true {
                            return Err(TylError::validation(
                                "status",
                                format!("Success criterion not met: {}", criterion.criterion)
                            ));
                        }
                    }
                }
                
                // Validate that all dependencies are completed
                self.validate_dependencies_completed(task).await?;
            },
            
            TaskStatus::Blocked => {
                // When blocking a task, ensure there's a reason documented
                if task.custom_properties.get("blocking_reason").is_none() {
                    return Err(TylError::validation(
                        "status",
                        "Blocked status requires a blocking reason in custom properties".to_string()
                    ));
                }
            },
            
            _ => {} // No special prerequisites for other statuses
        }
        
        Ok(())
    }
    
    /// Validate that all task dependencies are completed before marking as done (private helper)
    async fn validate_dependencies_completed(&self, task: &Task) -> TylResult<()> {
        let dependencies = self.repository.find_dependencies_by_task(&task.id).await?;
        
        for dependency in dependencies {
            if let Some(blocking_task) = self.repository.find_task_by_id(&dependency.to_task_id).await? {
                if !matches!(blocking_task.status, TaskStatus::Done) {
                    return Err(TylError::validation(
                        "status",
                        format!("Cannot complete task while dependency '{}' is not done", blocking_task.name)
                    ));
                }
            }
        }
        
        Ok(())
    }
}

/// Mock implementation of TaskService for development and testing
pub struct MockTaskService {
    tasks: std::sync::Arc<std::sync::Mutex<HashMap<String, Task>>>,
    dependencies: std::sync::Arc<std::sync::Mutex<HashMap<String, TaskDependency>>>,
    projects: std::sync::Arc<std::sync::Mutex<HashMap<String, Project>>>,
}

impl MockTaskService {
    pub fn new() -> Self {
        let mut tasks = HashMap::new();
        
        // Pre-populate with a test task for testing
        let test_task = Task::builder(
            "test-id".to_string(),
            "Test Task".to_string(),
            TaskContext::Work,
        )
        .priority(TaskPriority::Medium)
        .complexity(TaskComplexity::Medium)
        .source(TaskSource::Self_)
        .visibility(TaskVisibility::Private)
        .build();
        
        tasks.insert("test-id".to_string(), test_task);
        
        Self {
            tasks: std::sync::Arc::new(std::sync::Mutex::new(tasks)),
            dependencies: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            projects: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl TaskService for MockTaskService {
    async fn create_task(&self, request: CreateTaskRequest) -> TylResult<Task> {
        let task = Task::builder(request.id.clone(), request.name, request.context)
            .priority(request.priority)
            .complexity(request.complexity)
            .source(request.source)
            .visibility(request.visibility)
            .build();
        
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(request.id, task.clone());
        Ok(task)
    }
    
    async fn get_task_by_id(&self, id: &str) -> TylResult<Option<Task>> {
        let tasks = self.tasks.lock().unwrap();
        Ok(tasks.get(id).cloned())
    }
    
    async fn update_task(&self, id: &str, request: UpdateTaskRequest) -> TylResult<Task> {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(id) {
            if let Some(name) = request.name {
                task.name = name;
            }
            if let Some(priority) = request.priority {
                task.priority = priority;
            }
            task.updated_at = Utc::now();
            Ok(task.clone())
        } else {
            Err(TylError::not_found("task", id))
        }
    }
    
    async fn delete_task(&self, id: &str) -> TylResult<()> {
        let mut tasks = self.tasks.lock().unwrap();
        if tasks.remove(id).is_some() {
            Ok(())
        } else {
            Err(TylError::not_found("task", id))
        }
    }
    
    async fn list_tasks(&self, _filter: TaskFilter) -> TylResult<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        Ok(tasks.values().cloned().collect())
    }
    
    async fn add_task_dependency(
        &self,
        from_task_id: &str,
        to_task_id: &str,
        dependency_type: DependencyType,
    ) -> TylResult<TaskDependency> {
        let dependency = TaskDependency::new(
            from_task_id.to_string(),
            to_task_id.to_string(),
            dependency_type,
        );
        
        let mut dependencies = self.dependencies.lock().unwrap();
        dependencies.insert(dependency.id.clone(), dependency.clone());
        Ok(dependency)
    }
    
    async fn remove_task_dependency(&self, dependency_id: &str) -> TylResult<()> {
        let mut dependencies = self.dependencies.lock().unwrap();
        if dependencies.remove(dependency_id).is_some() {
            Ok(())
        } else {
            Err(TylError::not_found("dependency", dependency_id))
        }
    }
    
    async fn get_task_dependencies(&self, task_id: &str) -> TylResult<Vec<TaskDependency>> {
        let dependencies = self.dependencies.lock().unwrap();
        Ok(dependencies
            .values()
            .filter(|dep| dep.from_task_id == task_id)
            .cloned()
            .collect())
    }
    
    async fn get_blocked_tasks(&self, task_id: &str) -> TylResult<Vec<Task>> {
        let dependencies = self.dependencies.lock().unwrap();
        let tasks = self.tasks.lock().unwrap();
        
        let blocked_task_ids: Vec<String> = dependencies
            .values()
            .filter(|dep| dep.to_task_id == task_id && matches!(dep.dependency_type, DependencyType::Blocks))
            .map(|dep| dep.from_task_id.clone())
            .collect();
        
        Ok(blocked_task_ids
            .into_iter()
            .filter_map(|id| tasks.get(&id).cloned())
            .collect())
    }
    
    async fn add_subtask(&self, _parent_id: &str, _child_id: &str) -> TylResult<()> {
        Ok(()) // Mock implementation
    }
    
    async fn remove_subtask(&self, _parent_id: &str, _child_id: &str) -> TylResult<()> {
        Ok(()) // Mock implementation
    }
    
    async fn get_subtasks(&self, _parent_id: &str) -> TylResult<Vec<Task>> {
        Ok(vec![]) // Mock implementation
    }
    
    async fn get_parent_task(&self, _child_id: &str) -> TylResult<Option<Task>> {
        Ok(None) // Mock implementation
    }
    
    async fn transition_task_status(&self, task_id: &str, new_status: TaskStatus) -> TylResult<Task> {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(task_id) {
            task.update_status(new_status)?;
            Ok(task.clone())
        } else {
            Err(TylError::not_found("task", task_id))
        }
    }
    
    async fn assign_task(&self, _task_id: &str, _user_id: &str, _role: &str) -> TylResult<()> {
        Ok(()) // Mock implementation
    }
    
    async fn unassign_task(&self, _task_id: &str, _user_id: &str) -> TylResult<()> {
        Ok(()) // Mock implementation
    }
    
    async fn get_assigned_tasks(&self, _user_id: &str) -> TylResult<Vec<Task>> {
        Ok(vec![]) // Mock implementation
    }
    
    async fn create_project(&self, request: CreateProjectRequest) -> TylResult<Project> {
        let project = Project::new(request.id.clone(), request.code, request.name);
        let mut projects = self.projects.lock().unwrap();
        projects.insert(request.id, project.clone());
        Ok(project)
    }
    
    async fn add_task_to_project(&self, _task_id: &str, _project_id: &str) -> TylResult<()> {
        Ok(()) // Mock implementation
    }
    
    async fn get_project_tasks(&self, _project_id: &str) -> TylResult<Vec<Task>> {
        Ok(vec![]) // Mock implementation
    }
    
    async fn get_task_analytics(&self, _task_id: &str) -> TylResult<TaskAnalytics> {
        Ok(TaskAnalytics {
            task_id: _task_id.to_string(),
            completion_percentage: 0.0,
            blocking_count: 0,
            blocked_by_count: 0,
            subtask_count: 0,
            completed_subtasks: 0,
            is_on_critical_path: false,
            estimated_completion_date: None,
            time_to_completion_days: None,
            dependency_chain_length: 0,
            priority_score: 0.0,
        })
    }
    
    async fn get_critical_path(&self, _project_id: &str) -> TylResult<Vec<Task>> {
        Ok(vec![]) // Mock implementation
    }
    
    async fn detect_circular_dependencies(&self) -> TylResult<Vec<Vec<String>>> {
        Ok(vec![]) // Mock implementation
    }
    
    async fn get_detailed_circular_dependencies(&self) -> TylResult<Vec<DependencyCycle>> {
        Ok(vec![]) // Mock implementation
    }
    
    async fn get_actionable_tasks(&self, _user_id: &str) -> TylResult<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        Ok(tasks
            .values()
            .filter(|task| task.is_actionable())
            .cloned()
            .collect())
    }
    
    async fn get_overdue_tasks(&self) -> TylResult<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        Ok(tasks
            .values()
            .filter(|task| task.is_overdue())
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock repository for testing
    struct MockTaskRepository;
    
    #[async_trait]
    impl TaskRepository for MockTaskRepository {
        async fn save_task(&self, _task: &Task) -> TylResult<()> {
            Ok(())
        }
        
        async fn find_task_by_id(&self, id: &str) -> TylResult<Option<Task>> {
            if id == "test-task-1" {
                Ok(Some(Task::new("test-task-1".to_string(), "Test Task".to_string(), TaskContext::Work)))
            } else {
                Ok(None)
            }
        }
        
        async fn find_tasks_by_filter(&self, _filter: &TaskFilter) -> TylResult<Vec<Task>> {
            Ok(vec![])
        }
        
        async fn delete_task(&self, _id: &str) -> TylResult<()> {
            Ok(())
        }
        
        // Implement other methods with mock behavior...
        async fn save_dependency(&self, _dependency: &TaskDependency) -> TylResult<()> {
            Ok(())
        }
        
        async fn delete_dependency(&self, _dependency_id: &str) -> TylResult<()> {
            Ok(())
        }
        
        async fn find_dependencies_by_task(&self, _task_id: &str) -> TylResult<Vec<TaskDependency>> {
            Ok(vec![])
        }
        
        async fn find_blocking_tasks(&self, _task_id: &str) -> TylResult<Vec<Task>> {
            Ok(vec![])
        }
        
        async fn add_parent_child_relationship(&self, _parent_id: &str, _child_id: &str) -> TylResult<()> {
            Ok(())
        }
        
        async fn remove_parent_child_relationship(&self, _parent_id: &str, _child_id: &str) -> TylResult<()> {
            Ok(())
        }
        
        async fn find_children(&self, _parent_id: &str) -> TylResult<Vec<Task>> {
            Ok(vec![])
        }
        
        async fn find_parent(&self, _child_id: &str) -> TylResult<Option<Task>> {
            Ok(None)
        }
        
        async fn assign_user_to_task(&self, _task_id: &str, _user_id: &str, _role: &str) -> TylResult<()> {
            Ok(())
        }
        
        async fn unassign_user_from_task(&self, _task_id: &str, _user_id: &str) -> TylResult<()> {
            Ok(())
        }
        
        async fn find_assigned_tasks(&self, _user_id: &str) -> TylResult<Vec<Task>> {
            Ok(vec![])
        }
        
        async fn save_project(&self, _project: &Project) -> TylResult<()> {
            Ok(())
        }
        
        async fn add_task_to_project(&self, _task_id: &str, _project_id: &str) -> TylResult<()> {
            Ok(())
        }
        
        async fn find_project_tasks(&self, _project_id: &str) -> TylResult<Vec<Task>> {
            Ok(vec![])
        }
        
        async fn calculate_completion_percentage(&self, _task_id: &str) -> TylResult<f64> {
            Ok(0.0)
        }
        
        async fn find_critical_path(&self, _project_id: &str) -> TylResult<Vec<Task>> {
            Ok(vec![])
        }
        
        async fn detect_circular_dependencies(&self) -> TylResult<Vec<Vec<String>>> {
            Ok(vec![])
        }
    }
    
    #[tokio::test]
    async fn test_create_task() {
        let service = TaskDomainService::new(MockTaskRepository);
        
        let request = CreateTaskRequest {
            id: "PROJ1-T001".to_string(),
            name: "Test Task".to_string(),
            description: Some("A test task".to_string()),
            context: TaskContext::Work,
            priority: TaskPriority::High,
            complexity: TaskComplexity::Medium,
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
        
        let result = service.create_task(request).await;
        assert!(result.is_ok());
        
        let task = result.unwrap();
        assert_eq!(task.id, "PROJ1-T001");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.priority, TaskPriority::High);
    }
    
    #[tokio::test]
    async fn test_create_task_invalid_name() {
        let service = TaskDomainService::new(MockTaskRepository);
        
        let request = CreateTaskRequest {
            id: "PROJ1-T001".to_string(),
            name: "".to_string(), // Empty name
            description: None,
            context: TaskContext::Work,
            priority: TaskPriority::Medium,
            complexity: TaskComplexity::Medium,
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
        
        let result = service.create_task(request).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_get_task_by_id() {
        let service = TaskDomainService::new(MockTaskRepository);
        
        // Test existing task
        let result = service.get_task_by_id("test-task-1").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
        
        // Test non-existing task
        let result = service.get_task_by_id("non-existent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
    
    #[tokio::test]
    async fn test_status_transition_validation() {
        let service = TaskDomainService::new(MockTaskRepository);
        
        // Valid transition
        assert!(service.validate_status_transition(&TaskStatus::Backlog, &TaskStatus::Ready).is_ok());
        
        // Invalid transition
        assert!(service.validate_status_transition(&TaskStatus::Done, &TaskStatus::InProgress).is_err());
    }
}