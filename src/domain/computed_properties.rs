//! Computed properties and business rules for task management
//!
//! This module implements computed properties that are calculated dynamically
//! based on graph relationships and task state, as well as business rules
//! that enforce domain constraints and logic.

use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tyl_errors::{TylError, TylResult};
use tyl_falkordb_adapter::FalkorDBAdapter;

use super::{Task, TaskStatus, TaskPriority, DependencyType, TaskContext};

/// Service for computing dynamic task properties based on graph relationships
#[async_trait]
pub trait ComputedPropertyService {
    // Task completion and progress
    async fn calculate_completion_percentage(&self, task_id: &str) -> TylResult<f64>;
    async fn is_task_actionable(&self, task_id: &str) -> TylResult<bool>;
    async fn is_task_blocked(&self, task_id: &str) -> TylResult<bool>;
    async fn is_task_on_critical_path(&self, task_id: &str) -> TylResult<bool>;
    
    // Task dependencies and relationships
    async fn get_blocking_tasks(&self, task_id: &str) -> TylResult<Vec<String>>;
    async fn get_blocked_tasks(&self, task_id: &str) -> TylResult<Vec<String>>;
    async fn calculate_dependency_chain_length(&self, task_id: &str) -> TylResult<u32>;
    
    // Task timing and scheduling
    async fn calculate_earliest_start_date(&self, task_id: &str) -> TylResult<Option<DateTime<Utc>>>;
    async fn calculate_estimated_completion_date(&self, task_id: &str) -> TylResult<Option<DateTime<Utc>>>;
    async fn is_task_overdue(&self, task_id: &str) -> TylResult<bool>;
    async fn is_task_at_risk(&self, task_id: &str) -> TylResult<bool>;
    
    // Task priority and scoring
    async fn calculate_priority_score(&self, task_id: &str) -> TylResult<f64>;
    async fn calculate_impact_score(&self, task_id: &str) -> TylResult<f64>;
    async fn calculate_urgency_score(&self, task_id: &str) -> TylResult<f64>;
    
    // Resource and workload
    async fn has_required_resources(&self, task_id: &str) -> TylResult<bool>;
    async fn calculate_resource_utilization(&self, task_id: &str) -> TylResult<f64>;
    
    // Quality and risk metrics
    async fn calculate_complexity_score(&self, task_id: &str) -> TylResult<f64>;
    async fn calculate_risk_score(&self, task_id: &str) -> TylResult<f64>;
    async fn is_task_stale(&self, task_id: &str) -> TylResult<bool>;
}

/// Implementation of computed property service using graph database
pub struct GraphComputedPropertyService {
    adapter: std::sync::Arc<FalkorDBAdapter>,
    graph_name: String,
}

impl GraphComputedPropertyService {
    pub fn new(adapter: std::sync::Arc<FalkorDBAdapter>, graph_name: String) -> Self {
        Self {
            adapter,
            graph_name,
        }
    }
    
    /// Get task details from the graph
    async fn get_task(&self, task_id: &str) -> TylResult<Option<Task>> {
        let query = format!(
            "MATCH (t:Task {{id: '{}'}}) RETURN t",
            task_id
        );
        
        let result = self.adapter.execute_cypher(&query).await?;
        
        // Parse the result and convert to Task
        // This is a simplified implementation - in reality, you'd parse the graph result
        Ok(None) // Placeholder
    }
    
    /// Execute a Cypher query and get numeric result
    async fn execute_numeric_query(&self, query: &str) -> TylResult<f64> {
        let result = self.adapter.execute_cypher(query).await?;
        // Parse numeric result from graph response
        // This is simplified - real implementation would parse the actual result
        Ok(0.0)
    }
    
    /// Execute a Cypher query and get count result
    async fn execute_count_query(&self, query: &str) -> TylResult<u32> {
        let result = self.adapter.execute_cypher(query).await?;
        // Parse count result from graph response
        Ok(0)
    }
    
    /// Execute a Cypher query and get boolean result
    async fn execute_boolean_query(&self, query: &str) -> TylResult<bool> {
        let result = self.adapter.execute_cypher(query).await?;
        // Parse boolean result from graph response
        Ok(false)
    }
    
    /// Execute a Cypher query and get string list result
    async fn execute_string_list_query(&self, query: &str) -> TylResult<Vec<String>> {
        let result = self.adapter.execute_cypher(query).await?;
        // Parse string list result from graph response
        Ok(vec![])
    }
}

#[async_trait]
impl ComputedPropertyService for GraphComputedPropertyService {
    async fn calculate_completion_percentage(&self, task_id: &str) -> TylResult<f64> {
        // Calculate completion based on subtasks
        let query = format!(
            r#"
            MATCH (parent:Task {{id: '{}'}})
            OPTIONAL MATCH (parent)-[:SUBTASK_OF]->(child:Task)
            WITH parent, count(child) as total_subtasks, 
                 count(CASE WHEN child.status = 'Done' THEN 1 END) as completed_subtasks
            RETURN CASE 
                WHEN total_subtasks = 0 THEN 
                    CASE parent.status 
                        WHEN 'Done' THEN 100.0 
                        ELSE 0.0 
                    END
                ELSE (completed_subtasks * 100.0) / total_subtasks
            END as completion_percentage
            "#,
            task_id
        );
        
        self.execute_numeric_query(&query).await
    }
    
    async fn is_task_actionable(&self, task_id: &str) -> TylResult<bool> {
        // A task is actionable if it's Ready/InProgress and all dependencies are Done
        let query = format!(
            r#"
            MATCH (t:Task {{id: '{}'}})
            WHERE t.status IN ['Ready', 'InProgress']
            OPTIONAL MATCH (t)-[:DEPENDS_ON]->(dep:Task)
            WITH t, count(dep) as total_deps, count(CASE WHEN dep.status = 'Done' THEN 1 END) as completed_deps
            RETURN total_deps = completed_deps as is_actionable
            "#,
            task_id
        );
        
        self.execute_boolean_query(&query).await
    }
    
    async fn is_task_blocked(&self, task_id: &str) -> TylResult<bool> {
        // A task is blocked if it has incomplete dependencies
        let query = format!(
            r#"
            MATCH (t:Task {{id: '{}'}})
            MATCH (t)-[:DEPENDS_ON]->(dep:Task)
            WHERE dep.status <> 'Done'
            RETURN count(dep) > 0 as is_blocked
            "#,
            task_id
        );
        
        self.execute_boolean_query(&query).await
    }
    
    async fn is_task_on_critical_path(&self, task_id: &str) -> TylResult<bool> {
        // Critical path calculation - simplified version
        let query = format!(
            r#"
            MATCH path = (start:Task)-[:DEPENDS_ON*]->(t:Task {{id: '{}'}})
            WHERE NOT EXISTS((start)-[:DEPENDS_ON]->())
            WITH path, length(path) as path_length
            ORDER BY path_length DESC
            LIMIT 1
            RETURN path_length > 3 as is_critical
            "#,
            task_id
        );
        
        self.execute_boolean_query(&query).await
    }
    
    async fn get_blocking_tasks(&self, task_id: &str) -> TylResult<Vec<String>> {
        let query = format!(
            r#"
            MATCH (t:Task {{id: '{}'}})
            MATCH (t)-[:DEPENDS_ON]->(blocking:Task)
            WHERE blocking.status <> 'Done'
            RETURN blocking.id as blocking_task_id
            "#,
            task_id
        );
        
        self.execute_string_list_query(&query).await
    }
    
    async fn get_blocked_tasks(&self, task_id: &str) -> TylResult<Vec<String>> {
        let query = format!(
            r#"
            MATCH (t:Task {{id: '{}'}})
            MATCH (blocked:Task)-[:DEPENDS_ON]->(t)
            RETURN blocked.id as blocked_task_id
            "#,
            task_id
        );
        
        self.execute_string_list_query(&query).await
    }
    
    async fn calculate_dependency_chain_length(&self, task_id: &str) -> TylResult<u32> {
        let query = format!(
            r#"
            MATCH (t:Task {{id: '{}'}})
            OPTIONAL MATCH path = (t)-[:DEPENDS_ON*]->(dep:Task)
            RETURN max(length(path)) as max_chain_length
            "#,
            task_id
        );
        
        self.execute_count_query(&query).await
    }
    
    async fn calculate_earliest_start_date(&self, task_id: &str) -> TylResult<Option<DateTime<Utc>>> {
        // Calculate based on dependency completion dates
        let query = format!(
            r#"
            MATCH (t:Task {{id: '{}'}})
            OPTIONAL MATCH (t)-[:DEPENDS_ON]->(dep:Task)
            WITH t, max(dep.estimated_completion_date) as latest_dependency_end
            RETURN COALESCE(latest_dependency_end, t.created_at) as earliest_start
            "#,
            task_id
        );
        
        // This would parse the date result from the graph
        // For now, return None as placeholder
        Ok(None)
    }
    
    async fn calculate_estimated_completion_date(&self, task_id: &str) -> TylResult<Option<DateTime<Utc>>> {
        // Calculate based on earliest start + estimated duration
        let earliest_start = self.calculate_earliest_start_date(task_id).await?;
        
        if let Some(start) = earliest_start {
            // Add estimated duration based on complexity
            let task = self.get_task(task_id).await?;
            if let Some(task) = task {
                let duration_days = match task.complexity {
                    super::TaskComplexity::Simple => 1,
                    super::TaskComplexity::Medium => 3,
                    super::TaskComplexity::Complex => 7,
                    super::TaskComplexity::Epic => 21,
                };
                
                Ok(Some(start + Duration::days(duration_days)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    async fn is_task_overdue(&self, task_id: &str) -> TylResult<bool> {
        let query = format!(
            r#"
            MATCH (t:Task {{id: '{}'}})
            WHERE t.due_date IS NOT NULL 
              AND datetime(t.due_date) < datetime()
              AND t.status <> 'Done'
            RETURN count(t) > 0 as is_overdue
            "#,
            task_id
        );
        
        self.execute_boolean_query(&query).await
    }
    
    async fn is_task_at_risk(&self, task_id: &str) -> TylResult<bool> {
        // A task is at risk if it's close to due date or has many blocking dependencies
        let query = format!(
            r#"
            MATCH (t:Task {{id: '{}'}})
            OPTIONAL MATCH (t)-[:DEPENDS_ON]->(dep:Task)
            WHERE dep.status <> 'Done'
            WITH t, count(dep) as blocking_count
            RETURN (
                (t.due_date IS NOT NULL AND 
                 duration.between(datetime(), datetime(t.due_date)).days < 3) OR
                blocking_count > 2
            ) as is_at_risk
            "#,
            task_id
        );
        
        self.execute_boolean_query(&query).await
    }
    
    async fn calculate_priority_score(&self, task_id: &str) -> TylResult<f64> {
        let task = self.get_task(task_id).await?;
        if let Some(task) = task {
            let base_score = match task.priority {
                TaskPriority::Critical => 100.0,
                TaskPriority::High => 75.0,
                TaskPriority::Medium => 50.0,
                TaskPriority::Low => 25.0,
            };
            
            // Adjust based on overdue status
            let is_overdue = self.is_task_overdue(task_id).await?;
            let overdue_multiplier = if is_overdue { 1.5 } else { 1.0 };
            
            // Adjust based on blocking other tasks
            let blocked_tasks = self.get_blocked_tasks(task_id).await?;
            let blocking_bonus = blocked_tasks.len() as f64 * 5.0;
            
            Ok(base_score * overdue_multiplier + blocking_bonus)
        } else {
            Ok(0.0)
        }
    }
    
    async fn calculate_impact_score(&self, task_id: &str) -> TylResult<f64> {
        // Impact based on how many tasks depend on this one
        let blocked_tasks = self.get_blocked_tasks(task_id).await?;
        let chain_length = self.calculate_dependency_chain_length(task_id).await?;
        
        Ok((blocked_tasks.len() as f64 * 10.0) + (chain_length as f64 * 2.0))
    }
    
    async fn calculate_urgency_score(&self, task_id: &str) -> TylResult<f64> {
        let task = self.get_task(task_id).await?;
        if let Some(task) = task {
            if let Some(due_date) = task.due_date {
                let now = Utc::now();
                let days_until_due = (due_date - now).num_days();
                
                if days_until_due < 0 {
                    100.0 // Overdue
                } else if days_until_due == 0 {
                    90.0 // Due today
                } else if days_until_due <= 3 {
                    70.0 // Due within 3 days
                } else if days_until_due <= 7 {
                    50.0 // Due within a week
                } else {
                    20.0 // More than a week
                }
            } else {
                30.0 // No due date
            }
        } else {
            0.0
        }
    }
    
    async fn has_required_resources(&self, task_id: &str) -> TylResult<bool> {
        // Check if task has assigned users and required skills/resources
        let query = format!(
            r#"
            MATCH (t:Task {{id: '{}'}})
            OPTIONAL MATCH (t)-[:ASSIGNED_TO]->(u:User)
            OPTIONAL MATCH (t)-[:REQUIRES]->(r:Resource)
            WITH t, count(u) as assigned_users, count(r) as required_resources
            RETURN assigned_users > 0 AND required_resources = 0 as has_resources
            "#,
            task_id
        );
        
        self.execute_boolean_query(&query).await
    }
    
    async fn calculate_resource_utilization(&self, task_id: &str) -> TylResult<f64> {
        // Calculate resource utilization for assigned users
        let query = format!(
            r#"
            MATCH (t:Task {{id: '{}'}})
            MATCH (t)-[:ASSIGNED_TO]->(u:User)
            OPTIONAL MATCH (u)<-[:ASSIGNED_TO]-(other:Task)
            WHERE other.status IN ['Ready', 'InProgress'] AND other.id <> t.id
            WITH u, count(other) as other_tasks
            RETURN avg(other_tasks) as avg_utilization
            "#,
            task_id
        );
        
        self.execute_numeric_query(&query).await
    }
    
    async fn calculate_complexity_score(&self, task_id: &str) -> TylResult<f64> {
        let task = self.get_task(task_id).await?;
        if let Some(task) = task {
            let base_score = match task.complexity {
                super::TaskComplexity::Simple => 1.0,
                super::TaskComplexity::Medium => 3.0,
                super::TaskComplexity::Complex => 8.0,
                super::TaskComplexity::Epic => 21.0,
            };
            
            // Adjust based on number of success criteria
            let criteria_count = task.success_criteria.len() as f64;
            let criteria_bonus = criteria_count * 0.5;
            
            Ok(base_score + criteria_bonus)
        } else {
            Ok(1.0)
        }
    }
    
    async fn calculate_risk_score(&self, task_id: &str) -> TylResult<f64> {
        let complexity = self.calculate_complexity_score(task_id).await?;
        let is_blocked = self.is_task_blocked(task_id).await?;
        let is_overdue = self.is_task_overdue(task_id).await?;
        let has_resources = self.has_required_resources(task_id).await?;
        
        let mut risk_score = complexity * 10.0;
        
        if is_blocked {
            risk_score += 20.0;
        }
        
        if is_overdue {
            risk_score += 30.0;
        }
        
        if !has_resources {
            risk_score += 15.0;
        }
        
        Ok(risk_score.min(100.0)) // Cap at 100
    }
    
    async fn is_task_stale(&self, task_id: &str) -> TylResult<bool> {
        // A task is stale if it hasn't been updated in a while
        let query = format!(
            r#"
            MATCH (t:Task {{id: '{}'}})
            RETURN duration.between(datetime(t.updated_at), datetime()).days > 30 as is_stale
            "#,
            task_id
        );
        
        self.execute_boolean_query(&query).await
    }
}

/// Business rules engine for task management
#[async_trait]
pub trait BusinessRulesEngine {
    // Status transition rules
    async fn validate_status_transition(&self, task_id: &str, new_status: TaskStatus) -> TylResult<ValidationResult>;
    async fn can_start_task(&self, task_id: &str) -> TylResult<bool>;
    async fn can_complete_task(&self, task_id: &str) -> TylResult<bool>;
    
    // Assignment rules
    async fn validate_task_assignment(&self, task_id: &str, user_id: &str) -> TylResult<ValidationResult>;
    async fn check_user_capacity(&self, user_id: &str) -> TylResult<CapacityCheck>;
    
    // Dependency rules
    async fn validate_dependency_creation(&self, from_task: &str, to_task: &str, dep_type: DependencyType) -> TylResult<ValidationResult>;
    async fn check_circular_dependencies(&self, from_task: &str, to_task: &str) -> TylResult<bool>;
    
    // Project rules
    async fn validate_project_assignment(&self, task_id: &str, project_id: &str) -> TylResult<ValidationResult>;
    async fn check_project_capacity(&self, project_id: &str) -> TylResult<ProjectCapacityCheck>;
    
    // Deadline and scheduling rules
    async fn validate_due_date(&self, task_id: &str, due_date: DateTime<Utc>) -> TylResult<ValidationResult>;
    async fn suggest_realistic_due_date(&self, task_id: &str) -> TylResult<Option<DateTime<Utc>>>;
    
    // Quality gates
    async fn check_completion_criteria(&self, task_id: &str) -> TylResult<CompletionCriteriaCheck>;
    async fn validate_task_complexity(&self, task_id: &str) -> TylResult<ValidationResult>;
}

// ============================================================================
// Result Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub violations: Vec<RuleViolation>,
    pub warnings: Vec<RuleWarning>,
    pub suggestions: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            violations: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        }
    }
    
    pub fn invalid(violation: RuleViolation) -> Self {
        Self {
            is_valid: false,
            violations: vec![violation],
            warnings: Vec::new(),
            suggestions: Vec::new(),
        }
    }
    
    pub fn with_warnings(mut self, warnings: Vec<RuleWarning>) -> Self {
        self.warnings = warnings;
        self
    }
    
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleViolation {
    pub rule_name: String,
    pub severity: ViolationSeverity,
    pub message: String,
    pub field: Option<String>,
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleWarning {
    pub rule_name: String,
    pub message: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Error,   // Blocks the operation
    Warning, // Allows operation but not recommended
    Info,    // Informational only
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityCheck {
    pub user_id: String,
    pub current_task_count: u32,
    pub max_concurrent_tasks: u32,
    pub capacity_utilization: f64, // 0.0 to 1.0+
    pub can_take_more_tasks: bool,
    pub recommended_max_additional: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCapacityCheck {
    pub project_id: String,
    pub current_active_tasks: u32,
    pub max_concurrent_tasks: Option<u32>,
    pub team_size: u32,
    pub average_tasks_per_person: f64,
    pub capacity_status: CapacityStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapacityStatus {
    UnderUtilized,
    Optimal,
    NearCapacity,
    OverCapacity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionCriteriaCheck {
    pub task_id: String,
    pub has_success_criteria: bool,
    pub criteria_met: u32,
    pub total_criteria: u32,
    pub completion_percentage: f64,
    pub blocking_criteria: Vec<String>, // Criteria that must be met to complete
    pub can_complete: bool,
}

// ============================================================================
// Implementation
// ============================================================================

pub struct GraphComputedPropertyService {
    adapter: std::sync::Arc<FalkorDBAdapter>,
}

impl GraphComputedPropertyService {
    pub fn new(adapter: std::sync::Arc<FalkorDBAdapter>) -> Self {
        Self { adapter }
    }
    
    /// Build Cypher query to calculate completion percentage based on subtasks
    fn build_completion_percentage_query(&self, task_id: &str) -> String {
        format!(
            r#"
            MATCH (parent:Task {{id: '{}'}})
            OPTIONAL MATCH (parent)<-[:SUBTASK_OF]-(child:Task)
            WITH parent, 
                 count(child) as total_subtasks,
                 count(CASE WHEN child.status = 'done' THEN 1 END) as completed_subtasks
            RETURN 
              CASE 
                WHEN total_subtasks = 0 THEN 
                  CASE WHEN parent.status = 'done' THEN 100.0 ELSE 0.0 END
                ELSE (completed_subtasks * 100.0 / total_subtasks)
              END as completion_percentage
            "#,
            task_id.replace('\'', "\\'")
        )
    }
    
    /// Build query to check if task is actionable (no blocking dependencies)
    fn build_actionable_check_query(&self, task_id: &str) -> String {
        format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            OPTIONAL MATCH (task)-[:DEPENDS_ON {{dependency_type: 'blocks'}}]->(blocking:Task)
            WHERE blocking.status != 'done'
            WITH task, count(blocking) as blocking_count
            RETURN 
              task.status IN ['ready', 'in_progress'] AND 
              blocking_count = 0 as is_actionable
            "#,
            task_id.replace('\'', "\\'")
        )
    }
    
    /// Build query to calculate priority score based on multiple factors
    fn build_priority_score_query(&self, task_id: &str) -> String {
        format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            
            // Base priority score
            WITH task,
              CASE task.priority
                WHEN 'critical' THEN 100
                WHEN 'high' THEN 80
                WHEN 'medium' THEN 60
                WHEN 'low' THEN 40
                WHEN 'wish' THEN 20
                ELSE 30
              END as base_priority
            
            // Urgency based on due date
            WITH task, base_priority,
              CASE 
                WHEN task.due_date IS NULL THEN 0
                WHEN task.due_date < datetime() THEN 50  // Overdue
                WHEN task.due_date < datetime() + duration('P1D') THEN 40  // Due today
                WHEN task.due_date < datetime() + duration('P3D') THEN 30  // Due in 3 days
                WHEN task.due_date < datetime() + duration('P7D') THEN 20  // Due this week
                ELSE 10
              END as urgency_score
            
            // Impact based on blocking relationships
            OPTIONAL MATCH (task)<-[:DEPENDS_ON {{dependency_type: 'blocks'}}]-(blocked:Task)
            WHERE blocked.status != 'done'
            WITH task, base_priority, urgency_score, count(blocked) as blocked_count
            
            WITH task, base_priority, urgency_score, blocked_count,
              CASE 
                WHEN blocked_count > 10 THEN 30
                WHEN blocked_count > 5 THEN 20
                WHEN blocked_count > 0 THEN 10
                ELSE 0
              END as impact_score
            
            RETURN (base_priority + urgency_score + impact_score) as priority_score
            "#,
            task_id.replace('\'', "\\'")
        )
    }
}

#[async_trait]
impl ComputedPropertyService for GraphComputedPropertyService {
    async fn calculate_completion_percentage(&self, task_id: &str) -> TylResult<f64> {
        let query = self.build_completion_percentage_query(task_id);
        let result = self.adapter.execute_cypher(&query).await?;
        
        // Parse result from Cypher query
        // In a real implementation, we would parse the JSON response
        // For now, return a default value
        Ok(0.0)
    }
    
    async fn is_task_actionable(&self, task_id: &str) -> TylResult<bool> {
        let query = self.build_actionable_check_query(task_id);
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse boolean result from Cypher
        // For now, return true as default
        Ok(true)
    }
    
    async fn is_task_blocked(&self, task_id: &str) -> TylResult<bool> {
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            OPTIONAL MATCH (task)-[:DEPENDS_ON {{dependency_type: 'blocks'}}]->(blocking:Task)
            WHERE blocking.status NOT IN ['done', 'cancelled']
            RETURN count(blocking) > 0 as is_blocked
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse result
        Ok(false) // Default implementation
    }
    
    async fn is_task_on_critical_path(&self, task_id: &str) -> TylResult<bool> {
        // This would require complex critical path algorithm implementation
        // For now, return false as default
        Ok(false)
    }
    
    async fn get_blocking_tasks(&self, task_id: &str) -> TylResult<Vec<String>> {
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            MATCH (task)-[:DEPENDS_ON]->(blocking:Task)
            WHERE blocking.status NOT IN ['done', 'cancelled']
            RETURN collect(blocking.id) as blocking_task_ids
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse and return task IDs
        Ok(vec![]) // Default implementation
    }
    
    async fn get_blocked_tasks(&self, task_id: &str) -> TylResult<Vec<String>> {
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            MATCH (blocked:Task)-[:DEPENDS_ON]->(task)
            WHERE blocked.status NOT IN ['done', 'cancelled']
            RETURN collect(blocked.id) as blocked_task_ids
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse and return task IDs
        Ok(vec![]) // Default implementation
    }
    
    async fn calculate_dependency_chain_length(&self, task_id: &str) -> TylResult<u32> {
        let query = format!(
            r#"
            MATCH path = (task:Task {{id: '{}'}})-[:DEPENDS_ON*]->(dependency:Task)
            RETURN max(length(path)) as max_chain_length
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        Ok(0) // Default implementation
    }
    
    async fn calculate_earliest_start_date(&self, task_id: &str) -> TylResult<Option<DateTime<Utc>>> {
        // Calculate based on dependency completion dates
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            OPTIONAL MATCH (task)-[:DEPENDS_ON]->(dependency:Task)
            WHERE dependency.status != 'done'
            
            WITH task, max(dependency.estimated_completion_date) as latest_dependency_date
            
            RETURN 
              CASE 
                WHEN latest_dependency_date IS NULL THEN datetime()
                ELSE latest_dependency_date
              END as earliest_start_date
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // For now, return current time as default
        Ok(Some(Utc::now()))
    }
    
    async fn calculate_estimated_completion_date(&self, task_id: &str) -> TylResult<Option<DateTime<Utc>>> {
        // Calculate based on estimated effort and earliest start date
        // This is a simplified implementation
        Ok(Some(Utc::now() + Duration::days(7))) // Default: 7 days from now
    }
    
    async fn is_task_overdue(&self, task_id: &str) -> TylResult<bool> {
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            RETURN 
              task.due_date IS NOT NULL AND 
              task.due_date < datetime() AND
              task.status NOT IN ['done', 'cancelled'] as is_overdue
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        Ok(false) // Default implementation
    }
    
    async fn is_task_at_risk(&self, task_id: &str) -> TylResult<bool> {
        // A task is at risk if:
        // 1. It's approaching due date with no progress
        // 2. Its dependencies are delayed
        // 3. Assigned user is overloaded
        
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            
            // Check if approaching due date
            WITH task,
              task.due_date IS NOT NULL AND 
              task.due_date < datetime() + duration('P3D') AND
              task.status = 'backlog' as approaching_due
            
            // Check dependency delays
            OPTIONAL MATCH (task)-[:DEPENDS_ON]->(dep:Task)
            WHERE dep.due_date IS NOT NULL AND dep.due_date < datetime() AND dep.status != 'done'
            
            WITH task, approaching_due, count(dep) as delayed_dependencies
            
            RETURN approaching_due OR delayed_dependencies > 0 as is_at_risk
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        Ok(false) // Default implementation
    }
    
    async fn calculate_priority_score(&self, task_id: &str) -> TylResult<f64> {
        let query = self.build_priority_score_query(task_id);
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse numerical result
        Ok(50.0) // Default medium priority score
    }
    
    async fn calculate_impact_score(&self, task_id: &str) -> TylResult<f64> {
        // Impact based on how many other tasks depend on this one
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            OPTIONAL MATCH (blocked:Task)-[:DEPENDS_ON]->(task)
            WITH task, count(blocked) as directly_blocked
            
            // Calculate indirect impact through dependency chains
            OPTIONAL MATCH (indirectly_blocked:Task)-[:DEPENDS_ON*2..5]->(task)
            WITH task, directly_blocked, count(DISTINCT indirectly_blocked) as indirectly_blocked
            
            // Weight direct impact more heavily than indirect
            RETURN (directly_blocked * 2.0 + indirectly_blocked * 1.0) as impact_score
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        Ok(0.0) // Default implementation
    }
    
    async fn calculate_urgency_score(&self, task_id: &str) -> TylResult<f64> {
        // Urgency based on due date proximity
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            WITH task,
              CASE 
                WHEN task.due_date IS NULL THEN 0
                WHEN task.due_date < datetime() THEN 100  // Overdue = max urgency
                WHEN task.due_date < datetime() + duration('P1D') THEN 90
                WHEN task.due_date < datetime() + duration('P3D') THEN 70
                WHEN task.due_date < datetime() + duration('P7D') THEN 50
                WHEN task.due_date < datetime() + duration('P14D') THEN 30
                WHEN task.due_date < datetime() + duration('P30D') THEN 10
                ELSE 0
              END as urgency_score
            
            RETURN urgency_score
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        Ok(0.0) // Default implementation
    }
    
    async fn has_required_resources(&self, _task_id: &str) -> TylResult<bool> {
        // Check if task has required assignees, tools, etc.
        // This would be implemented based on resource management requirements
        Ok(true) // Default: assume resources are available
    }
    
    async fn calculate_resource_utilization(&self, _task_id: &str) -> TylResult<f64> {
        // Calculate how much of available resources this task is using
        Ok(0.5) // Default: 50% utilization
    }
    
    async fn calculate_complexity_score(&self, task_id: &str) -> TylResult<f64> {
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            
            // Base complexity score
            WITH task,
              CASE task.complexity
                WHEN 'trivial' THEN 1
                WHEN 'simple' THEN 3
                WHEN 'medium' THEN 5
                WHEN 'complex' THEN 8
                WHEN 'very_complex' THEN 13
                ELSE 5
              END as base_complexity
            
            // Factor in number of subtasks
            OPTIONAL MATCH (task)<-[:SUBTASK_OF]-(subtask:Task)
            WITH task, base_complexity, count(subtask) as subtask_count
            
            // Factor in dependencies
            OPTIONAL MATCH (task)-[:DEPENDS_ON]->(dependency:Task)
            WITH task, base_complexity, subtask_count, count(dependency) as dependency_count
            
            // Calculate final complexity score
            RETURN base_complexity + (subtask_count * 0.5) + (dependency_count * 0.3) as complexity_score
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        Ok(5.0) // Default: medium complexity
    }
    
    async fn calculate_risk_score(&self, task_id: &str) -> TylResult<f64> {
        // Risk score based on various factors
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            
            // Risk factors
            WITH task,
              // Due date risk
              CASE 
                WHEN task.due_date IS NULL THEN 0
                WHEN task.due_date < datetime() THEN 10  // Overdue
                WHEN task.due_date < datetime() + duration('P3D') THEN 7
                WHEN task.due_date < datetime() + duration('P7D') THEN 5
                ELSE 0
              END as due_date_risk,
              
              // Complexity risk
              CASE task.complexity
                WHEN 'very_complex' THEN 8
                WHEN 'complex' THEN 5
                WHEN 'medium' THEN 2
                ELSE 0
              END as complexity_risk
            
            // Dependency risk
            OPTIONAL MATCH (task)-[:DEPENDS_ON]->(dep:Task)
            WHERE dep.status IN ['blocked', 'backlog']
            WITH task, due_date_risk, complexity_risk, count(dep) as risky_dependencies
            
            // Assignment risk (unassigned or overloaded assignee)
            OPTIONAL MATCH (task)-[:ASSIGNED_TO]->(user:User)
            OPTIONAL MATCH (user)<-[:ASSIGNED_TO]-(other_task:Task)
            WHERE other_task.status IN ['in_progress', 'ready']
            
            WITH task, due_date_risk, complexity_risk, risky_dependencies,
              CASE 
                WHEN user IS NULL THEN 5  // Unassigned
                WHEN count(other_task) > 5 THEN 3  // Overloaded
                ELSE 0
              END as assignment_risk
            
            RETURN due_date_risk + complexity_risk + (risky_dependencies * 2) + assignment_risk as risk_score
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        Ok(0.0) // Default implementation
    }
    
    async fn is_task_stale(&self, task_id: &str) -> TylResult<bool> {
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            WITH task,
              task.status IN ['ready', 'backlog'] AND 
              task.updated_at < datetime() - duration('P30D') as is_stale
            RETURN is_stale
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        Ok(false) // Default implementation
    }
}

// Business Rules Engine Implementation
pub struct TaskBusinessRulesEngine {
    adapter: std::sync::Arc<FalkorDBAdapter>,
}

impl TaskBusinessRulesEngine {
    pub fn new(adapter: std::sync::Arc<FalkorDBAdapter>) -> Self {
        Self { adapter }
    }
}

#[async_trait]
impl BusinessRulesEngine for TaskBusinessRulesEngine {
    async fn validate_status_transition(&self, task_id: &str, new_status: TaskStatus) -> TylResult<ValidationResult> {
        // Get current task status
        let query = format!(
            "MATCH (task:Task {{id: '{}'}}) RETURN task.status as current_status",
            task_id.replace('\'', "\\'")
        );
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // In a real implementation, we would parse the current status and validate the transition
        // For now, return a valid result
        Ok(ValidationResult::valid())
    }
    
    async fn can_start_task(&self, task_id: &str) -> TylResult<bool> {
        // Check if all dependencies are met and task is assigned
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            
            // Check if task is assigned
            OPTIONAL MATCH (task)-[:ASSIGNED_TO]->(user:User)
            
            // Check blocking dependencies
            OPTIONAL MATCH (task)-[:DEPENDS_ON {{dependency_type: 'blocks'}}]->(blocking:Task)
            WHERE blocking.status NOT IN ['done', 'cancelled']
            
            WITH task, user, count(blocking) as blocking_count
            
            RETURN 
              user IS NOT NULL AND 
              blocking_count = 0 AND
              task.status IN ['ready', 'backlog'] as can_start
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        Ok(true) // Default implementation
    }
    
    async fn can_complete_task(&self, task_id: &str) -> TylResult<bool> {
        // Check if all subtasks are complete and success criteria are met
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            
            // Check subtasks
            OPTIONAL MATCH (task)<-[:SUBTASK_OF]-(subtask:Task)
            WHERE subtask.status NOT IN ['done', 'cancelled']
            
            WITH task, count(subtask) as incomplete_subtasks
            
            RETURN 
              task.status IN ['in_progress', 'review'] AND
              incomplete_subtasks = 0 as can_complete
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        Ok(true) // Default implementation
    }
    
    async fn validate_task_assignment(&self, task_id: &str, user_id: &str) -> TylResult<ValidationResult> {
        // Check user capacity and skills
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}}), (user:User {{id: '{}'}})
            
            // Count user's current tasks
            OPTIONAL MATCH (user)<-[:ASSIGNED_TO]-(current_task:Task)
            WHERE current_task.status IN ['in_progress', 'ready']
            
            WITH task, user, count(current_task) as current_task_count
            
            RETURN 
              current_task_count < 5 as within_capacity,  // Assume max 5 concurrent tasks
              current_task_count
            "#,
            task_id.replace('\'', "\\'"),
            user_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // For now, return valid assignment
        Ok(ValidationResult::valid())
    }
    
    async fn check_user_capacity(&self, user_id: &str) -> TylResult<CapacityCheck> {
        let query = format!(
            r#"
            MATCH (user:User {{id: '{}'}})
            OPTIONAL MATCH (user)<-[:ASSIGNED_TO]-(task:Task)
            WHERE task.status IN ['in_progress', 'ready', 'blocked']
            
            WITH user, count(task) as current_tasks
            
            RETURN current_tasks
            "#,
            user_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Return default capacity check
        Ok(CapacityCheck {
            user_id: user_id.to_string(),
            current_task_count: 0,
            max_concurrent_tasks: 5, // Default max
            capacity_utilization: 0.0,
            can_take_more_tasks: true,
            recommended_max_additional: 5,
        })
    }
    
    async fn validate_dependency_creation(&self, from_task: &str, to_task: &str, _dep_type: DependencyType) -> TylResult<ValidationResult> {
        // Check for circular dependencies
        let circular_check = self.check_circular_dependencies(from_task, to_task).await?;
        
        if circular_check {
            Ok(ValidationResult::invalid(RuleViolation {
                rule_name: "circular_dependency_prevention".to_string(),
                severity: ViolationSeverity::Error,
                message: "Creating this dependency would create a circular dependency".to_string(),
                field: Some("dependency".to_string()),
                suggested_fix: Some("Remove conflicting dependencies or restructure task hierarchy".to_string()),
            }))
        } else {
            Ok(ValidationResult::valid())
        }
    }
    
    async fn check_circular_dependencies(&self, from_task: &str, to_task: &str) -> TylResult<bool> {
        // Check if to_task already depends on from_task (directly or indirectly)
        let query = format!(
            r#"
            MATCH path = (to:Task {{id: '{}'}})-[:DEPENDS_ON*1..10]->(from:Task {{id: '{}'}})
            RETURN count(path) > 0 as would_create_cycle
            "#,
            to_task.replace('\'', "\\'"),
            from_task.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        Ok(false) // Default: no circular dependency
    }
    
    async fn validate_project_assignment(&self, task_id: &str, project_id: &str) -> TylResult<ValidationResult> {
        // Check if project exists and has capacity
        let query = format!(
            r#"
            MATCH (project:Project {{id: '{}'}})
            OPTIONAL MATCH (task:Task)-[:BELONGS_TO_PROJECT]->(project)
            
            WITH project, count(task) as current_task_count
            
            RETURN 
              project IS NOT NULL as project_exists,
              current_task_count < 100 as within_capacity  // Assume max 100 tasks per project
            "#,
            project_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        Ok(ValidationResult::valid())
    }
    
    async fn check_project_capacity(&self, project_id: &str) -> TylResult<ProjectCapacityCheck> {
        Ok(ProjectCapacityCheck {
            project_id: project_id.to_string(),
            current_active_tasks: 0,
            max_concurrent_tasks: Some(100), // Default limit
            team_size: 0,
            average_tasks_per_person: 0.0,
            capacity_status: CapacityStatus::Optimal,
        })
    }
    
    async fn validate_due_date(&self, task_id: &str, due_date: DateTime<Utc>) -> TylResult<ValidationResult> {
        // Check if due date is realistic based on dependencies
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            OPTIONAL MATCH (task)-[:DEPENDS_ON]->(dep:Task)
            
            WITH task, max(dep.estimated_completion_date) as latest_dependency_completion
            
            RETURN 
              latest_dependency_completion IS NULL OR 
              latest_dependency_completion <= datetime('{}') as is_realistic
            "#,
            task_id.replace('\'', "\\'"),
            due_date.to_rfc3339()
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        Ok(ValidationResult::valid())
    }
    
    async fn suggest_realistic_due_date(&self, task_id: &str) -> TylResult<Option<DateTime<Utc>>> {
        // Calculate based on dependency completion dates and estimated effort
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            OPTIONAL MATCH (task)-[:DEPENDS_ON]->(dep:Task)
            
            WITH task, 
                 CASE 
                   WHEN max(dep.estimated_completion_date) IS NULL THEN datetime()
                   ELSE max(dep.estimated_completion_date)
                 END as earliest_start_date
            
            RETURN earliest_start_date + duration('P7D') as suggested_due_date  // Default: 7 days after earliest start
            "#,
            task_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Return a reasonable default
        Ok(Some(Utc::now() + Duration::days(7)))
    }
    
    async fn check_completion_criteria(&self, task_id: &str) -> TylResult<CompletionCriteriaCheck> {
        // Check if task's success criteria are met
        // This would require parsing the success_criteria from the task
        Ok(CompletionCriteriaCheck {
            task_id: task_id.to_string(),
            has_success_criteria: false,
            criteria_met: 0,
            total_criteria: 0,
            completion_percentage: 100.0,
            blocking_criteria: vec![],
            can_complete: true,
        })
    }
    
    async fn validate_task_complexity(&self, _task_id: &str) -> TylResult<ValidationResult> {
        // Validate that task complexity matches its characteristics
        // (number of subtasks, dependencies, etc.)
        Ok(ValidationResult::valid())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation_result_creation() {
        let valid = ValidationResult::valid();
        assert!(valid.is_valid);
        assert!(valid.violations.is_empty());
        
        let violation = RuleViolation {
            rule_name: "test_rule".to_string(),
            severity: ViolationSeverity::Error,
            message: "Test violation".to_string(),
            field: None,
            suggested_fix: None,
        };
        
        let invalid = ValidationResult::invalid(violation);
        assert!(!invalid.is_valid);
        assert_eq!(invalid.violations.len(), 1);
    }
    
    #[test]
    fn test_capacity_check() {
        let capacity = CapacityCheck {
            user_id: "user123".to_string(),
            current_task_count: 3,
            max_concurrent_tasks: 5,
            capacity_utilization: 0.6,
            can_take_more_tasks: true,
            recommended_max_additional: 2,
        };
        
        assert_eq!(capacity.current_task_count, 3);
        assert!(capacity.can_take_more_tasks);
        assert_eq!(capacity.recommended_max_additional, 2);
    }
}