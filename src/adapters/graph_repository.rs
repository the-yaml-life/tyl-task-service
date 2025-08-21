//! Graph-based repository implementation for task management
//!
//! This module implements the TaskRepository trait using tyl-graph-port and tyl-falkordb-adapter,
//! providing graph database operations for the task management system.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tyl_errors::{TylError, TylResult};
use tyl_falkordb_adapter::{FalkorDBAdapter, GraphNode as FalkorNode, GraphRelationship as FalkorRel};
use tyl_graph_port::{
    GraphStore, GraphTraversal, GraphAnalytics, TraversalDirection, TraversalParams, CentralityType,
};

use crate::domain::{
    TaskRepository, Task, TaskDependency, TaskFilter, Project, TaskStatus, TaskPriority, 
    TaskContext, TaskComplexity, TaskSource, TaskVisibility, DependencyType
};

/// Graph-based repository implementation using FalkorDB
pub struct GraphTaskRepository {
    adapter: Arc<FalkorDBAdapter>,
    graph_name: String,
}

impl GraphTaskRepository {
    pub fn new(adapter: FalkorDBAdapter, graph_name: String) -> Self {
        Self {
            adapter: Arc::new(adapter),
            graph_name,
        }
    }
    
    /// Convert domain Task to graph node
    fn task_to_graph_node(&self, task: &Task) -> TylResult<FalkorNode> {
        let mut properties = HashMap::new();
        
        // Core properties
        properties.insert("id".to_string(), json!(task.id));
        properties.insert("uuid".to_string(), json!(task.uuid));
        properties.insert("name".to_string(), json!(task.name));
        properties.insert("context".to_string(), json!(task.context));
        properties.insert("status".to_string(), json!(task.status));
        properties.insert("priority".to_string(), json!(task.priority));
        properties.insert("complexity".to_string(), json!(task.complexity));
        properties.insert("source".to_string(), json!(task.source));
        properties.insert("visibility".to_string(), json!(task.visibility));
        
        // Optional properties
        if let Some(ref description) = task.description {
            properties.insert("description".to_string(), json!(description));
        }
        if let Some(ref details) = task.implementation_details {
            properties.insert("implementation_details".to_string(), json!(details));
        }
        if let Some(ref test_strategy) = task.test_strategy {
            properties.insert("test_strategy".to_string(), json!(test_strategy));
        }
        if let Some(ref due_date) = task.due_date {
            properties.insert("due_date".to_string(), json!(due_date.to_rfc3339()));
        }
        if let Some(ref estimated_date) = task.estimated_date {
            properties.insert("estimated_date".to_string(), json!(estimated_date.to_rfc3339()));
        }
        if let Some(ref started_at) = task.started_at {
            properties.insert("started_at".to_string(), json!(started_at.to_rfc3339()));
        }
        if let Some(ref completed_at) = task.completed_at {
            properties.insert("completed_at".to_string(), json!(completed_at.to_rfc3339()));
        }
        
        // Timestamps
        properties.insert("created_at".to_string(), json!(task.created_at.to_rfc3339()));
        properties.insert("updated_at".to_string(), json!(task.updated_at.to_rfc3339()));
        
        // Complex fields as JSON
        if !task.success_criteria.is_empty() {
            properties.insert("success_criteria".to_string(), json!(task.success_criteria));
        }
        if let Some(ref recurrence) = task.recurrence {
            properties.insert("recurrence".to_string(), json!(recurrence));
        }
        if !task.attachments.is_empty() {
            properties.insert("attachments".to_string(), json!(task.attachments));
        }
        
        // Custom properties
        for (key, value) in &task.custom_properties {
            properties.insert(format!("custom_{}", key), value.clone());
        }
        
        let mut node = FalkorNode::new(task.id.clone());
        node.labels = vec!["Task".to_string(), format!("Task_{:?}", task.context)];
        node.properties = properties;
        
        Ok(node)
    }
    
    /// Parse Cypher result row to extract Task
    fn parse_task_from_cypher_result(&self, result_row: &serde_json::Value) -> TylResult<Task> {
        // Extract the task node from the result (assuming it's returned as 't')
        let task_node = result_row.get("t")
            .ok_or_else(|| TylError::internal("Missing task data in Cypher result"))?;
        
        // FalkorDB returns node structure with 'properties' field
        let task_data = task_node.get("properties")
            .unwrap_or(task_node); // Fallback to direct node data if no properties field
        
        // Parse task properties from the result
        self.parse_task_from_json(task_data)
    }
    
    /// Parse Task from JSON data (from Cypher result or node properties)
    fn parse_task_from_json(&self, task_data: &serde_json::Value) -> TylResult<Task> {
        let properties = task_data.as_object()
            .ok_or_else(|| TylError::internal("Invalid task data format in result"))?;
        
        // Extract required fields
        let id = properties.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TylError::internal("Missing task id in result"))?
            .to_string();
        
        let name = properties.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TylError::internal("Missing task name in result"))?
            .to_string();
        
        let uuid = properties.get("uuid")
            .and_then(|v| v.as_str())
            .unwrap_or(&id)
            .to_string();
        
        // Parse enums with proper error handling
        let context: TaskContext = properties.get("context")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(TaskContext::Work);
        
        let status: TaskStatus = properties.get("status")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(TaskStatus::Backlog);
        
        let priority: TaskPriority = properties.get("priority")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(TaskPriority::Medium);
        
        let complexity: TaskComplexity = properties.get("complexity")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(TaskComplexity::Medium);
        
        let source: TaskSource = properties.get("source")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(TaskSource::Self_);
        
        let visibility: TaskVisibility = properties.get("visibility")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(TaskVisibility::Private);
        
        // Parse optional string fields
        let description = properties.get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let implementation_details = properties.get("implementation_details")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let test_strategy = properties.get("test_strategy")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // Parse dates with improved error handling
        let parse_date = |key: &str| -> Option<DateTime<Utc>> {
            properties.get(key)
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s)
                    .map_err(|e| {
                        eprintln!("Failed to parse date {}: {} - {}", key, s, e);
                        e
                    })
                    .ok())
                .map(|dt| dt.with_timezone(&Utc))
        };
        
        let due_date = parse_date("due_date");
        let estimated_date = parse_date("estimated_date");
        let started_at = parse_date("started_at");
        let completed_at = parse_date("completed_at");
        
        let created_at = parse_date("created_at")
            .unwrap_or_else(Utc::now);
        let updated_at = parse_date("updated_at")
            .unwrap_or_else(Utc::now);
        
        // Parse complex fields with better error handling
        let success_criteria = properties.get("success_criteria")
            .and_then(|v| serde_json::from_value(v.clone())
                .map_err(|e| {
                    eprintln!("Failed to parse success_criteria: {}", e);
                    e
                })
                .ok())
            .unwrap_or_default();
        
        let recurrence = properties.get("recurrence")
            .and_then(|v| serde_json::from_value(v.clone())
                .map_err(|e| {
                    eprintln!("Failed to parse recurrence: {}", e);
                    e
                })
                .ok());
        
        let attachments = properties.get("attachments")
            .and_then(|v| serde_json::from_value(v.clone())
                .map_err(|e| {
                    eprintln!("Failed to parse attachments: {}", e);
                    e
                })
                .ok())
            .unwrap_or_default();
        
        // Extract custom properties
        let mut custom_properties = HashMap::new();
        for (key, value) in properties {
            if key.starts_with("custom_") {
                let custom_key = key.strip_prefix("custom_").unwrap().to_string();
                custom_properties.insert(custom_key, value.clone());
            }
        }
        
        Ok(Task {
            id,
            uuid,
            name,
            description,
            context,
            status,
            priority,
            implementation_details,
            success_criteria,
            test_strategy,
            created_at,
            updated_at,
            started_at,
            completed_at,
            due_date,
            estimated_date,
            complexity,
            recurrence,
            source,
            visibility,
            attachments,
            custom_properties,
        })
    }
    
    /// Convert graph node to domain Task (legacy method for compatibility)
    fn graph_node_to_task(&self, node: &FalkorNode) -> TylResult<Task> {
        // Convert FalkorNode to JSON and use the enhanced parser
        let json_data = serde_json::to_value(&node.properties)
            .map_err(|e| TylError::internal(format!("Failed to convert node to JSON: {}", e)))?;
        
        self.parse_task_from_json(&json_data)
    }
    
    /// Parse TaskDependency from Cypher result
    fn parse_dependency_from_cypher_result(&self, result_row: &serde_json::Value) -> TylResult<TaskDependency> {
        // Extract the relationship from the result (assuming it's returned as 'r')
        let rel_data = result_row.get("r")
            .ok_or_else(|| TylError::internal("Missing relationship data in Cypher result"))?;
        
        self.parse_dependency_from_json(rel_data)
    }
    
    /// Parse TaskDependency from JSON data
    fn parse_dependency_from_json(&self, rel_data: &serde_json::Value) -> TylResult<TaskDependency> {
        let properties = rel_data.as_object()
            .ok_or_else(|| TylError::internal("Invalid relationship data format in result"))?;
        
        let id = properties.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TylError::internal("Missing dependency id in result"))?
            .to_string();
        
        let from_task_id = properties.get("from_task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TylError::internal("Missing from_task_id in dependency result"))?
            .to_string();
        
        let to_task_id = properties.get("to_task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TylError::internal("Missing to_task_id in dependency result"))?
            .to_string();
        
        let dependency_type: DependencyType = properties.get("dependency_type")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(DependencyType::Requires);
        
        let is_hard_dependency = properties.get("is_hard_dependency")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let delay_days = properties.get("delay_days")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        let created_at = properties.get("created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        
        // Extract additional properties
        let mut additional_properties = HashMap::new();
        for (key, value) in properties {
            if !matches!(key.as_str(), "id" | "from_task_id" | "to_task_id" | "dependency_type" | "is_hard_dependency" | "delay_days" | "created_at") {
                additional_properties.insert(key.clone(), value.clone());
            }
        }
        
        Ok(TaskDependency {
            id,
            from_task_id,
            to_task_id,
            dependency_type,
            is_hard_dependency,
            delay_days,
            created_at,
            properties: additional_properties,
        })
    }
    
    /// Parse Cypher query results into Tasks
    fn parse_tasks_from_cypher_results(&self, results: &serde_json::Value) -> TylResult<Vec<Task>> {
        let mut tasks = Vec::new();
        
        // Handle different result formats from FalkorDB
        if let Some(rows) = results.as_array() {
            for row in rows {
                match self.parse_task_from_cypher_result(row) {
                    Ok(task) => tasks.push(task),
                    Err(e) => {
                        eprintln!("Failed to parse task from result row: {}", e);
                        // Continue processing other rows rather than failing completely
                    }
                }
            }
        } else if results.is_object() {
            // Single result case
            if let Ok(task) = self.parse_task_from_cypher_result(results) {
                tasks.push(task);
            }
        }
        
        Ok(tasks)
    }
    
    /// Parse Cypher query results into TaskDependencies
    fn parse_dependencies_from_cypher_results(&self, results: &serde_json::Value) -> TylResult<Vec<TaskDependency>> {
        let mut dependencies = Vec::new();
        
        if let Some(rows) = results.as_array() {
            for row in rows {
                match self.parse_dependency_from_cypher_result(row) {
                    Ok(dependency) => dependencies.push(dependency),
                    Err(e) => {
                        eprintln!("Failed to parse dependency from result row: {}", e);
                        // Continue processing other rows
                    }
                }
            }
        } else if results.is_object() {
            // Single result case
            if let Ok(dependency) = self.parse_dependency_from_cypher_result(results) {
                dependencies.push(dependency);
            }
        }
        
        Ok(dependencies)
    }
    
    /// Convert domain TaskDependency to graph relationship
    fn dependency_to_graph_relationship(&self, dependency: &TaskDependency) -> FalkorRel {
        let mut properties = HashMap::new();
        properties.insert("id".to_string(), json!(dependency.id));
        properties.insert("dependency_type".to_string(), json!(dependency.dependency_type));
        properties.insert("is_hard_dependency".to_string(), json!(dependency.is_hard_dependency));
        properties.insert("delay_days".to_string(), json!(dependency.delay_days));
        properties.insert("created_at".to_string(), json!(dependency.created_at.to_rfc3339()));
        
        for (key, value) in &dependency.properties {
            properties.insert(key.clone(), value.clone());
        }
        
        FalkorRel {
            id: dependency.id.clone(),
            from_node_id: dependency.from_task_id.clone(),
            to_node_id: dependency.to_task_id.clone(),
            relationship_type: "DEPENDS_ON".to_string(),
            properties,
            created_at: dependency.created_at,
            updated_at: dependency.created_at,
        }
    }
    
    /// Build Cypher WHERE clause from TaskFilter
    fn build_filter_clause(&self, filter: &TaskFilter) -> String {
        let mut conditions = Vec::new();
        
        if let Some(ref statuses) = filter.status {
            let status_list: Vec<String> = statuses.iter()
                .map(|s| format!("'{:?}'", s).to_lowercase())
                .collect();
            conditions.push(format!("t.status IN [{}]", status_list.join(", ")));
        }
        
        if let Some(ref priorities) = filter.priority {
            let priority_list: Vec<String> = priorities.iter()
                .map(|p| format!("'{:?}'", p).to_lowercase())
                .collect();
            conditions.push(format!("t.priority IN [{}]", priority_list.join(", ")));
        }
        
        if let Some(ref contexts) = filter.context {
            let context_list: Vec<String> = contexts.iter()
                .map(|c| format!("'{:?}'", c).to_lowercase())
                .collect();
            conditions.push(format!("t.context IN [{}]", context_list.join(", ")));
        }
        
        if let Some(ref user_id) = filter.assigned_user_id {
            conditions.push(format!("EXISTS((t)<-[:ASSIGNED_TO]-(u:User {{id: '{}'}}))", user_id));
        }
        
        if let Some(ref project_id) = filter.project_id {
            conditions.push(format!("EXISTS((t)-[:BELONGS_TO_PROJECT]->(p:Project {{id: '{}'}}))", project_id));
        }
        
        if let Some(ref due_before) = filter.due_before {
            conditions.push(format!("t.due_date < '{}'", due_before.to_rfc3339()));
        }
        
        if let Some(ref due_after) = filter.due_after {
            conditions.push(format!("t.due_date > '{}'", due_after.to_rfc3339()));
        }
        
        if let Some(ref created_after) = filter.created_after {
            conditions.push(format!("t.created_at > '{}'", created_after.to_rfc3339()));
        }
        
        if filter.is_overdue == Some(true) {
            let now = Utc::now().to_rfc3339();
            conditions.push(format!("t.due_date < '{}' AND t.status NOT IN ['done', 'cancelled']", now));
        }
        
        if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        }
    }
}

#[async_trait]
impl TaskRepository for GraphTaskRepository {
    async fn save_task(&self, task: &Task) -> TylResult<()> {
        let node = self.task_to_graph_node(task)?;
        
        // Check if task exists
        match self.adapter.get_node(&task.id).await? {
            Some(_) => {
                // Update existing node - in a real implementation we'd use graph update operations
                // For now, we'll delete and recreate
                let _result = self.adapter.execute_cypher(&format!(
                    "MATCH (t:Task {{id: '{}'}}) DELETE t", 
                    task.id.replace('\'', "\\'")
                )).await?;
            }
            None => {
                // Node doesn't exist, will be created below
            }
        }
        
        // Create the node
        self.adapter.create_node(node).await?;
        Ok(())
    }
    
    async fn find_task_by_id(&self, id: &str) -> TylResult<Option<Task>> {
        match self.adapter.get_node(id).await? {
            Some(node) => {
                let task = self.graph_node_to_task(&node)?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }
    
    async fn find_tasks_by_filter(&self, filter: &TaskFilter) -> TylResult<Vec<Task>> {
        let where_clause = self.build_filter_clause(filter);
        let limit_clause = if let Some(limit) = filter.limit {
            format!("LIMIT {}", limit)
        } else {
            String::new()
        };
        let offset_clause = if let Some(offset) = filter.offset {
            format!("SKIP {}", offset)
        } else {
            String::new()
        };
        
        let query = format!(
            "MATCH (t:Task) {} RETURN t ORDER BY t.created_at DESC {} {}",
            where_clause, offset_clause, limit_clause
        );
        
        let result = self.adapter.execute_cypher(&query).await?;
        
        // Parse the Cypher results into Task objects
        self.parse_tasks_from_cypher_results(&result)
    }
    
    async fn delete_task(&self, id: &str) -> TylResult<()> {
        let query = format!(
            "MATCH (t:Task {{id: '{}'}}) DETACH DELETE t", 
            id.replace('\'', "\\'")
        );
        self.adapter.execute_cypher(&query).await?;
        Ok(())
    }
    
    async fn save_dependency(&self, dependency: &TaskDependency) -> TylResult<()> {
        let relationship = self.dependency_to_graph_relationship(dependency);
        self.adapter.create_relationship(relationship).await?;
        Ok(())
    }
    
    async fn delete_dependency(&self, dependency_id: &str) -> TylResult<()> {
        let query = format!(
            "MATCH ()-[r:DEPENDS_ON {{id: '{}'}}]-() DELETE r", 
            dependency_id.replace('\'', "\\'")
        );
        self.adapter.execute_cypher(&query).await?;
        Ok(())
    }
    
    async fn find_dependencies_by_task(&self, task_id: &str) -> TylResult<Vec<TaskDependency>> {
        let query = format!(
            "MATCH (t:Task {{id: '{}'}})-[r:DEPENDS_ON]->(dep:Task) RETURN r", 
            task_id.replace('\'', "\\'")
        );
        let result = self.adapter.execute_cypher(&query).await?;
        
        // Parse the Cypher results into TaskDependency objects
        self.parse_dependencies_from_cypher_results(&result)
    }
    
    async fn find_blocking_tasks(&self, task_id: &str) -> TylResult<Vec<Task>> {
        let query = format!(
            "MATCH (t:Task {{id: '{}'}})<-[r:DEPENDS_ON]-(blocked:Task) WHERE r.dependency_type = 'blocks' RETURN blocked", 
            task_id.replace('\'', "\\'")
        );
        let result = self.adapter.execute_cypher(&query).await?;
        
        // Parse the Cypher results into Task objects
        self.parse_tasks_from_cypher_results(&result)
    }
    
    async fn add_parent_child_relationship(&self, parent_id: &str, child_id: &str) -> TylResult<()> {
        let query = format!(
            "MATCH (parent:Task {{id: '{}'}}), (child:Task {{id: '{}'}}) 
             CREATE (child)-[:SUBTASK_OF]->(parent)", 
            parent_id.replace('\'', "\\'"),
            child_id.replace('\'', "\\'")
        );
        self.adapter.execute_cypher(&query).await?;
        Ok(())
    }
    
    async fn remove_parent_child_relationship(&self, parent_id: &str, child_id: &str) -> TylResult<()> {
        let query = format!(
            "MATCH (parent:Task {{id: '{}'}})<-[r:SUBTASK_OF]-(child:Task {{id: '{}'}}) DELETE r", 
            parent_id.replace('\'', "\\'"),
            child_id.replace('\'', "\\'")
        );
        self.adapter.execute_cypher(&query).await?;
        Ok(())
    }
    
    async fn find_children(&self, parent_id: &str) -> TylResult<Vec<Task>> {
        let query = format!(
            "MATCH (parent:Task {{id: '{}'}})<-[:SUBTASK_OF]-(child:Task) RETURN child", 
            parent_id.replace('\'', "\\'")
        );
        let result = self.adapter.execute_cypher(&query).await?;
        
        // Parse the Cypher results into Task objects
        self.parse_tasks_from_cypher_results(&result)
    }
    
    async fn find_parent(&self, child_id: &str) -> TylResult<Option<Task>> {
        let query = format!(
            "MATCH (child:Task {{id: '{}'}})-[:SUBTASK_OF]->(parent:Task) RETURN parent", 
            child_id.replace('\'', "\\'")
        );
        let result = self.adapter.execute_cypher(&query).await?;
        
        // Parse the Cypher results - get first task if any
        let tasks = self.parse_tasks_from_cypher_results(&result)?;
        Ok(tasks.into_iter().next())
    }
    
    async fn assign_user_to_task(&self, task_id: &str, user_id: &str, role: &str) -> TylResult<()> {
        let query = format!(
            "MATCH (t:Task {{id: '{}'}}), (u:User {{id: '{}'}}) 
             CREATE (t)-[:ASSIGNED_TO {{role: '{}'}}]->(u)", 
            task_id.replace('\'', "\\'"),
            user_id.replace('\'', "\\'"),
            role.replace('\'', "\\'")
        );
        self.adapter.execute_cypher(&query).await?;
        Ok(())
    }
    
    async fn unassign_user_from_task(&self, task_id: &str, user_id: &str) -> TylResult<()> {
        let query = format!(
            "MATCH (t:Task {{id: '{}'}})-[r:ASSIGNED_TO]->(u:User {{id: '{}'}}) DELETE r", 
            task_id.replace('\'', "\\'"),
            user_id.replace('\'', "\\'")
        );
        self.adapter.execute_cypher(&query).await?;
        Ok(())
    }
    
    async fn find_assigned_tasks(&self, user_id: &str) -> TylResult<Vec<Task>> {
        let query = format!(
            "MATCH (t:Task)-[:ASSIGNED_TO]->(u:User {{id: '{}'}}) RETURN t", 
            user_id.replace('\'', "\\'")
        );
        let result = self.adapter.execute_cypher(&query).await?;
        
        // Parse the Cypher results into Task objects
        self.parse_tasks_from_cypher_results(&result)
    }
    
    async fn save_project(&self, project: &Project) -> TylResult<()> {
        let mut properties = HashMap::new();
        properties.insert("id".to_string(), json!(project.id));
        properties.insert("code".to_string(), json!(project.code));
        properties.insert("name".to_string(), json!(project.name));
        properties.insert("status".to_string(), json!(project.status));
        properties.insert("created_at".to_string(), json!(project.created_at.to_rfc3339()));
        properties.insert("updated_at".to_string(), json!(project.updated_at.to_rfc3339()));
        
        if let Some(ref description) = project.description {
            properties.insert("description".to_string(), json!(description));
        }
        if let Some(ref start_date) = project.start_date {
            properties.insert("start_date".to_string(), json!(start_date.to_rfc3339()));
        }
        if let Some(ref end_date) = project.end_date {
            properties.insert("end_date".to_string(), json!(end_date.to_rfc3339()));
        }
        
        let mut node = FalkorNode::new(project.id.clone());
        node.labels = vec!["Project".to_string()];
        node.properties = properties;
        
        self.adapter.create_node(node).await?;
        Ok(())
    }
    
    async fn add_task_to_project(&self, task_id: &str, project_id: &str) -> TylResult<()> {
        let query = format!(
            "MATCH (t:Task {{id: '{}'}}), (p:Project {{id: '{}'}}) 
             CREATE (t)-[:BELONGS_TO_PROJECT]->(p)", 
            task_id.replace('\'', "\\'"),
            project_id.replace('\'', "\\'")
        );
        self.adapter.execute_cypher(&query).await?;
        Ok(())
    }
    
    async fn find_project_tasks(&self, project_id: &str) -> TylResult<Vec<Task>> {
        let query = format!(
            "MATCH (t:Task)-[:BELONGS_TO_PROJECT]->(p:Project {{id: '{}'}}) RETURN t", 
            project_id.replace('\'', "\\'")
        );
        let result = self.adapter.execute_cypher(&query).await?;
        
        // Parse the Cypher results into Task objects
        self.parse_tasks_from_cypher_results(&result)
    }
    
    async fn calculate_completion_percentage(&self, task_id: &str) -> TylResult<f64> {
        let query = format!(
            "MATCH (parent:Task {{id: '{}'}})<-[:SUBTASK_OF]-(child:Task)
             WITH parent, count(child) as total_subtasks, 
                  size([c in collect(child) WHERE c.status = 'done']) as completed_subtasks
             RETURN CASE WHEN total_subtasks = 0 THEN 
                CASE WHEN parent.status = 'done' THEN 100.0 ELSE 0.0 END
                ELSE (completed_subtasks * 100.0 / total_subtasks) END as percentage", 
            task_id.replace('\'', "\\'")
        );
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // In a real implementation, we would parse the result
        // For now, return a default value
        Ok(0.0)
    }
    
    async fn find_critical_path(&self, project_id: &str) -> TylResult<Vec<Task>> {
        let query = format!(
            "MATCH (p:Project {{id: '{}'}})
             MATCH (t:Task)-[:BELONGS_TO_PROJECT]->(p)
             // Complex critical path algorithm would be implemented here
             RETURN t", 
            project_id.replace('\'', "\\'")
        );
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // In a real implementation, we would implement critical path algorithm
        Ok(vec![])
    }
    
    async fn detect_circular_dependencies(&self) -> TylResult<Vec<Vec<String>>> {
        let query = "
            MATCH (t:Task)-[:DEPENDS_ON*]->(t)
            WITH collect(DISTINCT t.id) as cycle
            WHERE size(cycle) > 1
            RETURN cycle
        ";
        let _result = self.adapter.execute_cypher(query).await?;
        
        // In a real implementation, we would parse the cycles
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tyl_config::RedisConfig;
    use crate::domain::{Task, TaskContext, TaskPriority};
    
    // Note: These tests would require a running FalkorDB instance
    // For CI/CD, we'd use integration test patterns with test containers
    
    #[tokio::test]
    #[ignore] // Ignore by default - requires FalkorDB instance
    async fn test_task_to_graph_node_conversion() {
        let task = Task::new(
            "PROJ1-T001".to_string(),
            "Test Task".to_string(),
            TaskContext::Work,
        );
        
        let config = RedisConfig::default();
        let adapter = FalkorDBAdapter::new(config, "test_graph".to_string()).await.unwrap();
        let repo = GraphTaskRepository::new(adapter, "test_graph".to_string());
        
        let node = repo.task_to_graph_node(&task).unwrap();
        
        assert_eq!(node.id, task.id);
        assert!(node.labels.contains(&"Task".to_string()));
        assert_eq!(node.properties.get("name").unwrap().as_str().unwrap(), task.name);
    }
    
    #[tokio::test]
    #[ignore] // Ignore by default - requires FalkorDB instance
    async fn test_graph_node_to_task_conversion() {
        let config = RedisConfig::default();
        let adapter = FalkorDBAdapter::new(config, "test_graph".to_string()).await.unwrap();
        let repo = GraphTaskRepository::new(adapter, "test_graph".to_string());
        
        let mut properties = HashMap::new();
        properties.insert("id".to_string(), json!("TEST-001"));
        properties.insert("name".to_string(), json!("Test Task"));
        properties.insert("context".to_string(), json!("work"));
        properties.insert("status".to_string(), json!("backlog"));
        properties.insert("priority".to_string(), json!("medium"));
        properties.insert("complexity".to_string(), json!("medium"));
        properties.insert("source".to_string(), json!("self"));
        properties.insert("visibility".to_string(), json!("private"));
        properties.insert("created_at".to_string(), json!(Utc::now().to_rfc3339()));
        properties.insert("updated_at".to_string(), json!(Utc::now().to_rfc3339()));
        
        let node = FalkorNode {
            id: "TEST-001".to_string(),
            labels: vec!["Task".to_string()],
            properties,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let task = repo.graph_node_to_task(&node).unwrap();
        
        assert_eq!(task.id, "TEST-001");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.context, TaskContext::Work);
        assert_eq!(task.priority, TaskPriority::Medium);
    }
    
    #[tokio::test]
    async fn test_build_filter_clause() {
        let config = RedisConfig::default();
        // Using a mock adapter for this test since we're only testing string building
        let mock_adapter = FalkorDBAdapter::new(config, "test".to_string()).await.unwrap();
        let repo = GraphTaskRepository::new(mock_adapter, "test".to_string());
        
        let filter = TaskFilter {
            status: Some(vec![TaskStatus::Ready, TaskStatus::InProgress]),
            priority: Some(vec![TaskPriority::High]),
            context: Some(vec![TaskContext::Work]),
            assigned_user_id: Some("user123".to_string()),
            is_overdue: Some(true),
            ..Default::default()
        };
        
        let clause = repo.build_filter_clause(&filter);
        
        // Check that the clause contains expected conditions
        assert!(clause.contains("WHERE"));
        assert!(clause.contains("t.status IN"));
        assert!(clause.contains("t.priority IN"));
        assert!(clause.contains("t.context IN"));
        assert!(clause.contains("EXISTS((t)<-[:ASSIGNED_TO]-(u:User {id: 'user123'}))"));
    }
}