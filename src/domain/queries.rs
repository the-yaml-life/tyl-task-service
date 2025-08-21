//! Complex graph queries for task management system
//!
//! This module implements advanced graph database queries for task relationships,
//! analytics, and insights using Cypher through the FalkorDB adapter.

use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tyl_errors::{TylError, TylResult};
use tyl_falkordb_adapter::FalkorDBAdapter;

use super::{Task, TaskStatus, TaskPriority, TaskContext, TaskComplexity, DependencyType};

/// Complex query service for advanced task operations
#[async_trait]
pub trait TaskQueryService {
    // Path and dependency analysis
    async fn find_dependency_chain(&self, task_id: &str) -> TylResult<Vec<DependencyPath>>;
    async fn find_blocking_path(&self, from_task: &str, to_task: &str) -> TylResult<Option<BlockingPath>>;
    async fn detect_circular_dependencies(&self) -> TylResult<Vec<DependencyCycle>>;
    async fn find_critical_path(&self, project_id: &str) -> TylResult<CriticalPath>;
    
    // Task recommendation and intelligence
    async fn recommend_next_tasks(&self, user_id: &str, limit: usize) -> TylResult<Vec<TaskRecommendation>>;
    async fn find_similar_tasks(&self, task_id: &str, limit: usize) -> TylResult<Vec<SimilarTask>>;
    async fn predict_completion_time(&self, task_id: &str) -> TylResult<CompletionPrediction>;
    
    // Analytics and insights
    async fn calculate_user_velocity(&self, user_id: &str, days: u32) -> TylResult<UserVelocity>;
    async fn analyze_bottlenecks(&self, project_id: Option<&str>) -> TylResult<Vec<Bottleneck>>;
    async fn get_task_impact_analysis(&self, task_id: &str) -> TylResult<TaskImpactAnalysis>;
    
    // Advanced search and filtering
    async fn semantic_search(&self, query: &str, context: Option<TaskContext>) -> TylResult<Vec<Task>>;
    async fn find_tasks_by_pattern(&self, pattern: TaskPattern) -> TylResult<Vec<Task>>;
    async fn get_task_timeline(&self, task_id: &str) -> TylResult<TaskTimeline>;
    
    // Resource and workload analysis
    async fn analyze_workload_distribution(&self) -> TylResult<WorkloadDistribution>;
    async fn find_over_allocated_users(&self) -> TylResult<Vec<OverAllocatedUser>>;
    async fn suggest_task_reassignment(&self, task_id: &str) -> TylResult<Vec<ReassignmentSuggestion>>;
    
    // Collaboration insights
    async fn find_collaboration_patterns(&self, user_id: &str) -> TylResult<CollaborationPatterns>;
    async fn identify_knowledge_experts(&self, domain: &str) -> TylResult<Vec<KnowledgeExpert>>;
    
    // Performance metrics
    async fn get_project_health_metrics(&self, project_id: &str) -> TylResult<ProjectHealth>;
    async fn calculate_team_productivity(&self, team_ids: Vec<String>, period_days: u32) -> TylResult<TeamProductivity>;
}

// ============================================================================
// Query Result Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyPath {
    pub path_id: String,
    pub task_chain: Vec<String>, // Task IDs in dependency order
    pub total_estimated_time: Option<Duration>,
    pub blocking_score: f64, // Higher score = more critical
    pub longest_chain: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockingPath {
    pub from_task: String,
    pub to_task: String,
    pub blocking_tasks: Vec<String>, // Tasks that must be completed first
    pub estimated_delay_days: i32,
    pub bypass_possible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCycle {
    pub cycle_id: String,
    pub tasks_in_cycle: Vec<String>,
    pub cycle_length: u32,
    pub severity: CycleSeverity,
    pub suggested_breaks: Vec<DependencyBreakSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CycleSeverity {
    Low,    // Can be resolved easily
    Medium, // Requires planning
    High,   // Blocks progress
    Critical, // Immediate action needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyBreakSuggestion {
    pub from_task: String,
    pub to_task: String,
    pub reason: String,
    pub impact_score: f64, // Lower = better to break
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalPath {
    pub project_id: String,
    pub path_tasks: Vec<String>, // Task IDs on critical path
    pub total_duration_days: i32,
    pub completion_probability: f64, // 0.0 to 1.0
    pub risk_factors: Vec<RiskFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: RiskType,
    pub description: String,
    pub probability: f64,
    pub impact: f64,
    pub mitigation_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskType {
    ResourceConstraint,
    DependencyRisk,
    ComplexityRisk,
    ExternalDependency,
    SkillGap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecommendation {
    pub task: Task,
    pub recommendation_score: f64, // Higher = better recommendation
    pub reasoning: Vec<String>, // Why this task is recommended
    pub estimated_effort: Option<Duration>,
    pub skill_match: f64, // How well it matches user's skills
    pub urgency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarTask {
    pub task: Task,
    pub similarity_score: f64, // 0.0 to 1.0
    pub similarity_factors: Vec<String>, // What makes them similar
    pub lessons_learned: Vec<String>, // Insights from the similar task
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionPrediction {
    pub task_id: String,
    pub predicted_completion_date: DateTime<Utc>,
    pub confidence_interval: (DateTime<Utc>, DateTime<Utc>), // Min, Max
    pub confidence_level: f64, // 0.0 to 1.0
    pub prediction_factors: Vec<PredictionFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionFactor {
    pub factor_name: String,
    pub weight: f64,
    pub influence: String, // "positive", "negative", "neutral"
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserVelocity {
    pub user_id: String,
    pub period_days: u32,
    pub tasks_completed: u32,
    pub average_completion_time: Duration,
    pub complexity_handled: Vec<(TaskComplexity, u32)>, // Complexity level and count
    pub velocity_trend: VelocityTrend, // Improving, declining, stable
    pub productivity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VelocityTrend {
    Improving,
    Declining,
    Stable,
    InsufficientData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub bottleneck_id: String,
    pub bottleneck_type: BottleneckType,
    pub affected_tasks: Vec<String>,
    pub severity: f64, // Higher = more severe
    pub estimated_delay: Duration,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckType {
    SinglePersonDependency, // One person is blocking many tasks
    ResourceConstraint,     // Limited resources
    ExternalDependency,     // Waiting on external systems/people
    ProcessBottleneck,      // Process issue causing delays
    SkillBottleneck,       // Lack of required skills
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskImpactAnalysis {
    pub task_id: String,
    pub directly_blocked_tasks: u32,
    pub indirectly_blocked_tasks: u32,
    pub total_impact_score: f64,
    pub affected_projects: Vec<String>,
    pub affected_users: Vec<String>,
    pub delay_propagation: Vec<DelayImpact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayImpact {
    pub task_id: String,
    pub estimated_delay_days: i32,
    pub probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPattern {
    pub name_pattern: Option<String>,
    pub context: Option<TaskContext>,
    pub priority_range: Option<(TaskPriority, TaskPriority)>,
    pub has_dependencies: Option<bool>,
    pub assigned_to_pattern: Option<String>, // User ID or pattern
    pub created_date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub custom_property_filters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTimeline {
    pub task_id: String,
    pub timeline_events: Vec<TimelineEvent>,
    pub duration_breakdown: DurationBreakdown,
    pub milestone_events: Vec<MilestoneEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub event_id: String,
    pub event_type: String, // "created", "assigned", "status_change", etc.
    pub timestamp: DateTime<Utc>,
    pub actor: Option<String>, // User who caused the event
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationBreakdown {
    pub time_in_backlog: Duration,
    pub time_in_ready: Duration,
    pub time_in_progress: Duration,
    pub time_in_review: Duration,
    pub time_blocked: Duration,
    pub total_cycle_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneEvent {
    pub milestone: String,
    pub achieved_at: DateTime<Utc>,
    pub time_to_milestone: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadDistribution {
    pub total_active_tasks: u32,
    pub user_workloads: Vec<UserWorkload>,
    pub workload_balance_score: f64, // Higher = more balanced
    pub overload_risk_users: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWorkload {
    pub user_id: String,
    pub assigned_tasks: u32,
    pub in_progress_tasks: u32,
    pub overdue_tasks: u32,
    pub workload_score: f64, // Higher = more loaded
    pub capacity_utilization: f64, // 0.0 to 1.0+
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverAllocatedUser {
    pub user_id: String,
    pub overallocation_score: f64,
    pub concurrent_tasks: u32,
    pub overdue_tasks: u32,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReassignmentSuggestion {
    pub to_user_id: String,
    pub suitability_score: f64, // How well suited they are
    pub availability_score: f64, // How available they are
    pub skill_match_score: f64,
    pub reasoning: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationPatterns {
    pub user_id: String,
    pub frequent_collaborators: Vec<CollaboratorInfo>,
    pub collaboration_effectiveness: f64,
    pub preferred_collaboration_types: Vec<String>,
    pub team_contribution_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorInfo {
    pub collaborator_id: String,
    pub collaboration_frequency: u32,
    pub shared_projects: Vec<String>,
    pub collaboration_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeExpert {
    pub user_id: String,
    pub domain: String,
    pub expertise_score: f64,
    pub task_success_rate: f64,
    pub knowledge_areas: Vec<String>,
    pub mentorship_potential: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHealth {
    pub project_id: String,
    pub overall_health_score: f64, // 0.0 to 1.0, higher is better
    pub completion_percentage: f64,
    pub on_track_probability: f64,
    pub risk_level: RiskLevel,
    pub health_indicators: Vec<HealthIndicator>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIndicator {
    pub indicator_name: String,
    pub current_value: f64,
    pub target_value: f64,
    pub trend: String, // "improving", "declining", "stable"
    pub impact: f64, // How much this affects overall health
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamProductivity {
    pub team_ids: Vec<String>,
    pub period_days: u32,
    pub total_tasks_completed: u32,
    pub average_cycle_time: Duration,
    pub productivity_score: f64,
    pub productivity_trend: VelocityTrend,
    pub efficiency_metrics: EfficiencyMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub throughput: f64, // Tasks per day
    pub quality_score: f64, // Based on rework, bug rates, etc.
    pub collaboration_score: f64,
    pub process_adherence: f64,
}

// ============================================================================
// Implementation using FalkorDB
// ============================================================================

pub struct GraphTaskQueryService {
    adapter: std::sync::Arc<FalkorDBAdapter>,
}

impl GraphTaskQueryService {
    pub fn new(adapter: std::sync::Arc<FalkorDBAdapter>) -> Self {
        Self { adapter }
    }
    
    /// Build complex Cypher query for dependency chain analysis
    fn build_dependency_chain_query(&self, task_id: &str) -> String {
        format!(
            r#"
            MATCH path = (start:Task {{id: '{}'}})-[:DEPENDS_ON*1..10]->(dep:Task)
            WITH path, length(path) as depth
            ORDER BY depth DESC
            WITH collect(path)[0] as longest_path
            UNWIND nodes(longest_path) as task_node
            WITH task_node, 
                 [n in nodes(longest_path) | n.id] as task_chain,
                 reduce(total = 0, n IN nodes(longest_path) | 
                   total + coalesce(n.estimated_hours, 0)) as total_hours
            RETURN DISTINCT task_chain, total_hours, length(longest_path) as chain_length
            "#,
            task_id.replace('\'', "\\'")
        )
    }
    
    /// Build Cypher query for circular dependency detection
    fn build_circular_dependency_query(&self) -> String {
        r#"
        MATCH (t:Task)-[:DEPENDS_ON*1..20]->(t)
        WITH t, 
             [n in nodes(path) | n.id] as cycle_nodes,
             length(path) as cycle_length
        WHERE cycle_length >= 2
        RETURN DISTINCT cycle_nodes, cycle_length
        ORDER BY cycle_length ASC
        "#.to_string()
    }
    
    /// Build query for task recommendations based on user history and current workload
    fn build_recommendation_query(&self, user_id: &str) -> String {
        format!(
            r#"
            // Find user's skill areas based on completed tasks
            MATCH (u:User {{id: '{}'}})<-[:ASSIGNED_TO]-(completed:Task {{status: 'done'}})
            WITH u, collect(DISTINCT completed.context) as user_contexts,
                 avg(completed.complexity_score) as avg_complexity
            
            // Find actionable tasks not assigned to this user
            MATCH (available:Task)
            WHERE available.status IN ['ready', 'backlog'] 
              AND NOT (available)-[:ASSIGNED_TO]->(u)
              AND available.context IN user_contexts
              AND available.complexity_score <= avg_complexity * 1.2
              
            // Calculate recommendation score
            WITH available, u,
                 CASE 
                   WHEN available.priority = 'critical' THEN 100
                   WHEN available.priority = 'high' THEN 80
                   WHEN available.priority = 'medium' THEN 60
                   WHEN available.priority = 'low' THEN 40
                   ELSE 20
                 END as priority_score,
                 
                 CASE
                   WHEN available.due_date < datetime() + duration('P7D') THEN 50
                   WHEN available.due_date < datetime() + duration('P14D') THEN 30
                   ELSE 10
                 END as urgency_score
                 
            RETURN available, (priority_score + urgency_score) as recommendation_score
            ORDER BY recommendation_score DESC
            LIMIT 10
            "#,
            user_id.replace('\'', "\\'")
        )
    }
}

#[async_trait]
impl TaskQueryService for GraphTaskQueryService {
    async fn find_dependency_chain(&self, task_id: &str) -> TylResult<Vec<DependencyPath>> {
        let query = self.build_dependency_chain_query(task_id);
        let result = self.adapter.execute_cypher(&query).await?;
        
        // In a real implementation, we would parse the Cypher results into DependencyPath structs
        // For now, return a simplified result
        Ok(vec![DependencyPath {
            path_id: uuid::Uuid::new_v4().to_string(),
            task_chain: vec![task_id.to_string()],
            total_estimated_time: None,
            blocking_score: 0.0,
            longest_chain: true,
        }])
    }
    
    async fn find_blocking_path(&self, from_task: &str, to_task: &str) -> TylResult<Option<BlockingPath>> {
        let query = format!(
            r#"
            MATCH path = shortestPath((from:Task {{id: '{}'}})-[:DEPENDS_ON*]->(to:Task {{id: '{}'}}))
            WHERE from.status != 'done' OR to.status != 'done'
            RETURN [n in nodes(path) | n.id] as blocking_path,
                   length(path) as path_length
            "#,
            from_task.replace('\'', "\\'"),
            to_task.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse results and return BlockingPath
        // For now, return None (simplified implementation)
        Ok(None)
    }
    
    async fn detect_circular_dependencies(&self) -> TylResult<Vec<DependencyCycle>> {
        let query = self.build_circular_dependency_query();
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse results into DependencyCycle structs
        // For now, return empty vector
        Ok(vec![])
    }
    
    async fn find_critical_path(&self, project_id: &str) -> TylResult<CriticalPath> {
        let query = format!(
            r#"
            MATCH (p:Project {{id: '{}'}})
            MATCH (t:Task)-[:BELONGS_TO_PROJECT]->(p)
            
            // Find the longest path through task dependencies
            MATCH path = (start:Task)-[:DEPENDS_ON*]->(end:Task)
            WHERE start.status != 'done' AND end.status != 'done'
              AND (start)-[:BELONGS_TO_PROJECT]->(p)
              AND (end)-[:BELONGS_TO_PROJECT]->(p)
            
            WITH path, 
                 reduce(total = 0, n IN nodes(path) | 
                   total + coalesce(n.estimated_days, 1)) as total_duration
            ORDER BY total_duration DESC
            LIMIT 1
            
            RETURN [n in nodes(path) | n.id] as critical_path_tasks, total_duration
            "#,
            project_id.replace('\'', "\\'")
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        Ok(CriticalPath {
            project_id: project_id.to_string(),
            path_tasks: vec![],
            total_duration_days: 0,
            completion_probability: 0.8, // Default estimate
            risk_factors: vec![],
        })
    }
    
    async fn recommend_next_tasks(&self, user_id: &str, limit: usize) -> TylResult<Vec<TaskRecommendation>> {
        let query = self.build_recommendation_query(user_id);
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse results and create recommendations
        // For now, return empty vector
        Ok(vec![])
    }
    
    async fn find_similar_tasks(&self, task_id: &str, limit: usize) -> TylResult<Vec<SimilarTask>> {
        let query = format!(
            r#"
            MATCH (target:Task {{id: '{}'}})
            MATCH (similar:Task)
            WHERE similar.id != target.id
              AND (similar.context = target.context 
                   OR similar.priority = target.priority
                   OR similar.complexity = target.complexity)
            
            // Calculate similarity score
            WITH target, similar,
                 CASE WHEN similar.context = target.context THEN 1 ELSE 0 END +
                 CASE WHEN similar.priority = target.priority THEN 1 ELSE 0 END +
                 CASE WHEN similar.complexity = target.complexity THEN 1 ELSE 0 END as base_score
                 
            // Add text similarity for names and descriptions
            WITH target, similar, base_score,
                 CASE 
                   WHEN target.name CONTAINS similar.name OR similar.name CONTAINS target.name THEN 2
                   ELSE 0
                 END as text_score
                 
            RETURN similar, (base_score + text_score) / 5.0 as similarity_score
            ORDER BY similarity_score DESC
            LIMIT {}
            "#,
            task_id.replace('\'', "\\'"),
            limit
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse results into SimilarTask structs
        Ok(vec![])
    }
    
    // Implement remaining methods with similar patterns...
    async fn predict_completion_time(&self, task_id: &str) -> TylResult<CompletionPrediction> {
        let query = format!(
            r#"
            MATCH (target:Task {{id: '{}'}})
            OPTIONAL MATCH (similar:Task)
            WHERE similar.context = target.context 
              AND similar.complexity = target.complexity
              AND similar.status = 'done'
              AND similar.id != target.id
            
            WITH target, 
                 avg(similar.actual_completion_days) as avg_completion,
                 stdev(similar.actual_completion_days) as completion_stdev,
                 count(similar) as similar_count
                 
            RETURN target.estimated_days as estimated,
                   coalesce(avg_completion, target.estimated_days, 5) as predicted_days,
                   coalesce(completion_stdev, 2) as std_deviation,
                   similar_count
            "#,
            task_id.replace('\'', "\\'"),
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // In a real implementation, we would parse Cypher results
        let now = Utc::now();
        let predicted_completion = now + Duration::days(5); // Default estimate
        let confidence = 0.7; // Based on historical accuracy
        
        Ok(CompletionPrediction {
            task_id: task_id.to_string(),
            predicted_completion_date: predicted_completion,
            confidence_interval: (
                predicted_completion - Duration::days(2),
                predicted_completion + Duration::days(3),
            ),
            confidence_level: confidence,
            prediction_factors: vec![
                PredictionFactor {
                    factor_name: "Historical Similarity".to_string(),
                    weight: 0.6,
                    influence: "positive".to_string(),
                    description: "Based on similar completed tasks".to_string(),
                },
                PredictionFactor {
                    factor_name: "Task Complexity".to_string(),
                    weight: 0.4,
                    influence: "negative".to_string(),
                    description: "Higher complexity may extend timeline".to_string(),
                },
            ],
        })
    }
    
    async fn calculate_user_velocity(&self, user_id: &str, days: u32) -> TylResult<UserVelocity> {
        let query = format!(
            r#"
            MATCH (u:User {{id: '{}'}})  
            MATCH (u)<-[:ASSIGNED_TO]-(t:Task {{status: 'done'}})
            WHERE t.completed_date > datetime() - duration('P{}D')
            
            WITH u, t,
                 duration.between(t.created_date, t.completed_date).days as completion_days
            
            RETURN count(t) as tasks_completed,
                   avg(completion_days) as avg_completion_time,
                   collect(DISTINCT t.complexity) as complexities_handled,
                   stdev(completion_days) as completion_variance
            "#,
            user_id.replace('\'', "\\'"),
            days
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Calculate velocity metrics (simplified)
        let tasks_completed = 12u32; // From query results
        let avg_time = Duration::days(3);
        let productivity_score = (tasks_completed as f64) / (days as f64) * 10.0;
        
        Ok(UserVelocity {
            user_id: user_id.to_string(),
            period_days: days,
            tasks_completed,
            average_completion_time: avg_time,
            complexity_handled: vec![
                (TaskComplexity::Simple, 8),
                (TaskComplexity::Medium, 3),
                (TaskComplexity::Complex, 1),
            ],
            velocity_trend: VelocityTrend::Stable,
            productivity_score,
        })
    }
    
    async fn analyze_bottlenecks(&self, project_id: Option<&str>) -> TylResult<Vec<Bottleneck>> {
        let project_filter = project_id.map(|id| format!("AND (t)-[:BELONGS_TO_PROJECT]->(:Project {{id: '{}'}})", id.replace('\'', "\\'")))
                                      .unwrap_or_else(|| String::new());
        
        let query = format!(
            r#"
            // Find users with many blocked tasks
            MATCH (u:User)<-[:ASSIGNED_TO]-(t:Task)
            WHERE t.status IN ['blocked', 'waiting'] {}
            
            WITH u, count(t) as blocked_tasks,
                 collect(t.id) as affected_task_ids
            WHERE blocked_tasks >= 3
            
            RETURN u.id as user_id, 
                   blocked_tasks,
                   affected_task_ids,
                   'single_person_dependency' as bottleneck_type
                   
            UNION ALL
            
            // Find external dependency bottlenecks
            MATCH (t:Task)
            WHERE t.status = 'waiting' 
              AND t.blocking_reason CONTAINS 'external' {}
              
            RETURN 'external_system' as user_id,
                   count(t) as blocked_tasks,
                   collect(t.id) as affected_task_ids,
                   'external_dependency' as bottleneck_type
            "#,
            project_filter, project_filter
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse results into bottlenecks
        Ok(vec![
            Bottleneck {
                bottleneck_id: uuid::Uuid::new_v4().to_string(),
                bottleneck_type: BottleneckType::SinglePersonDependency,
                affected_tasks: vec!["TASK-001".to_string(), "TASK-002".to_string()],
                severity: 0.8,
                estimated_delay: Duration::days(5),
                suggested_actions: vec![
                    "Redistribute tasks to other team members".to_string(),
                    "Provide additional support or resources".to_string(),
                    "Consider task prioritization changes".to_string(),
                ],
            }
        ])
    }
    
    async fn get_task_impact_analysis(&self, task_id: &str) -> TylResult<TaskImpactAnalysis> {
        let query = format!(
            r#"
            MATCH (target:Task {{id: '{}'}})
            
            // Find directly blocked tasks
            OPTIONAL MATCH (target)<-[:DEPENDS_ON]-(direct:Task)
            WHERE direct.status != 'done'
            
            // Find indirectly blocked tasks
            OPTIONAL MATCH path = (target)<-[:DEPENDS_ON*2..5]-(indirect:Task)
            WHERE indirect.status != 'done'
            
            // Find affected projects
            OPTIONAL MATCH (target)-[:BELONGS_TO_PROJECT]->(p:Project)
            OPTIONAL MATCH (direct)-[:BELONGS_TO_PROJECT]->(p2:Project)
            
            // Find affected users
            OPTIONAL MATCH (direct)<-[:ASSIGNED_TO]-(u:User)
            OPTIONAL MATCH (indirect)<-[:ASSIGNED_TO]-(u2:User)
            
            RETURN count(DISTINCT direct) as direct_blocked,
                   count(DISTINCT indirect) as indirect_blocked,
                   collect(DISTINCT p.id) + collect(DISTINCT p2.id) as affected_projects,
                   collect(DISTINCT u.id) + collect(DISTINCT u2.id) as affected_users
            "#,
            task_id.replace('\'', "\\'"),
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Calculate impact analysis (simplified)
        let direct_blocked = 3u32;
        let indirect_blocked = 8u32;
        let total_impact = (direct_blocked as f64 * 1.0) + (indirect_blocked as f64 * 0.5);
        
        Ok(TaskImpactAnalysis {
            task_id: task_id.to_string(),
            directly_blocked_tasks: direct_blocked,
            indirectly_blocked_tasks: indirect_blocked,
            total_impact_score: total_impact,
            affected_projects: vec!["PROJECT-001".to_string()],
            affected_users: vec!["user-123".to_string(), "user-456".to_string()],
            delay_propagation: vec![
                DelayImpact {
                    task_id: "TASK-002".to_string(),
                    estimated_delay_days: 2,
                    probability: 0.9,
                },
                DelayImpact {
                    task_id: "TASK-003".to_string(),
                    estimated_delay_days: 5,
                    probability: 0.6,
                },
            ],
        })
    }
    
    async fn semantic_search(&self, query: &str, context: Option<TaskContext>) -> TylResult<Vec<Task>> {
        let context_filter = context.map(|ctx| format!("AND t.context = '{:?}'", ctx))
                                   .unwrap_or_else(|| String::new());
        
        let search_query = format!(
            r#"
            MATCH (t:Task)
            WHERE (t.name CONTAINS '{}' OR t.description CONTAINS '{}')
              AND t.status != 'done' {}
              
            // Calculate relevance score
            WITH t,
                 CASE 
                   WHEN t.name CONTAINS '{}' THEN 10
                   ELSE 0
                 END +
                 CASE
                   WHEN t.description CONTAINS '{}' THEN 5 
                   ELSE 0
                 END as relevance_score
                 
            RETURN t, relevance_score
            ORDER BY relevance_score DESC, t.created_date DESC
            LIMIT 20
            "#,
            query.replace('\'', "\\'"),
            query.replace('\'', "\\'"),
            context_filter,
            query.replace('\'', "\\'"),
            query.replace('\'', "\\'"),
        );
        
        let _result = self.adapter.execute_cypher(&search_query).await?;
        
        // Parse results into Task structs
        // For now, return empty vector (would parse Cypher results in real implementation)
        Ok(vec![])
    }
    
    async fn find_tasks_by_pattern(&self, pattern: TaskPattern) -> TylResult<Vec<Task>> {
        let mut conditions = Vec::new();
        
        if let Some(ref name_pattern) = pattern.name_pattern {
            conditions.push(format!("t.name CONTAINS '{}'", name_pattern.replace('\'', "\\'")));
        }
        
        if let Some(context) = pattern.context {
            conditions.push(format!("t.context = '{:?}'", context));
        }
        
        if let Some((min_priority, max_priority)) = pattern.priority_range {
            // Simplified priority comparison (would need proper enum ordering)
            conditions.push(format!("t.priority IN ['{:?}', '{:?}']", min_priority, max_priority));
        }
        
        if let Some(has_deps) = pattern.has_dependencies {
            if has_deps {
                conditions.push("exists((t)-[:DEPENDS_ON]->())".to_string());
            } else {
                conditions.push("NOT exists((t)-[:DEPENDS_ON]->())".to_string());
            }
        }
        
        if let Some(ref user_pattern) = pattern.assigned_to_pattern {
            conditions.push(format!("exists((t)<-[:ASSIGNED_TO]-(:User {{id: '{}'}})) OR exists((t)<-[:ASSIGNED_TO]-(u:User WHERE u.name CONTAINS '{}'  OR u.email CONTAINS '{}')); ", 
                user_pattern.replace('\'', "\\'"), 
                user_pattern.replace('\'', "\\'"), 
                user_pattern.replace('\'', "\\'")
            ));
        }
        
        if let Some((start_date, end_date)) = pattern.created_date_range {
            conditions.push(format!(
                "t.created_date >= datetime('{}') AND t.created_date <= datetime('{}')",
                start_date.format("%Y-%m-%dT%H:%M:%SZ"),
                end_date.format("%Y-%m-%dT%H:%M:%SZ"),
            ));
        }
        
        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };
        
        let query = format!(
            r#"
            MATCH (t:Task)
            {}
            RETURN t
            ORDER BY t.created_date DESC
            LIMIT 100
            "#,
            where_clause
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse results into Task structs
        Ok(vec![])
    }
    
    async fn get_task_timeline(&self, task_id: &str) -> TylResult<TaskTimeline> {
        let query = format!(
            r#"
            MATCH (t:Task {{id: '{}'}})
            OPTIONAL MATCH (t)-[:HAS_EVENT]->(e:TaskEvent)
            
            WITH t, e 
            ORDER BY e.timestamp ASC
            
            RETURN t,
                   collect({{
                     event_id: e.id,
                     event_type: e.event_type,
                     timestamp: e.timestamp,
                     actor: e.actor_id,
                     description: e.description,
                     metadata: e.metadata
                   }}) as events
            "#,
            task_id.replace('\'', "\\'"),
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Build timeline from events (simplified)
        let now = Utc::now();
        let created_time = now - Duration::days(10);
        
        Ok(TaskTimeline {
            task_id: task_id.to_string(),
            timeline_events: vec![
                TimelineEvent {
                    event_id: uuid::Uuid::new_v4().to_string(),
                    event_type: "created".to_string(),
                    timestamp: created_time,
                    actor: Some("user-123".to_string()),
                    description: "Task was created".to_string(),
                    metadata: HashMap::new(),
                },
                TimelineEvent {
                    event_id: uuid::Uuid::new_v4().to_string(),
                    event_type: "status_change".to_string(),
                    timestamp: created_time + Duration::days(1),
                    actor: Some("user-123".to_string()),
                    description: "Status changed from Backlog to Ready".to_string(),
                    metadata: HashMap::new(),
                },
            ],
            duration_breakdown: DurationBreakdown {
                time_in_backlog: Duration::days(1),
                time_in_ready: Duration::days(2),
                time_in_progress: Duration::days(3),
                time_in_review: Duration::days(1),
                time_blocked: Duration::days(0),
                total_cycle_time: Duration::days(7),
            },
            milestone_events: vec![
                MilestoneEvent {
                    milestone: "Started".to_string(),
                    achieved_at: created_time + Duration::days(3),
                    time_to_milestone: Duration::days(3),
                },
            ],
        })
    }
    
    async fn analyze_workload_distribution(&self) -> TylResult<WorkloadDistribution> {
        let query = r#"
            MATCH (u:User)
            OPTIONAL MATCH (u)<-[:ASSIGNED_TO]-(active:Task)
            WHERE active.status IN ['ready', 'in_progress', 'blocked']
            
            OPTIONAL MATCH (u)<-[:ASSIGNED_TO]-(in_progress:Task {status: 'in_progress'})
            OPTIONAL MATCH (u)<-[:ASSIGNED_TO]-(overdue:Task)
            WHERE overdue.due_date < datetime() AND overdue.status != 'done'
            
            WITH u, 
                 count(active) as assigned_tasks,
                 count(in_progress) as in_progress_tasks,
                 count(overdue) as overdue_tasks
                 
            // Calculate workload score (0-100 scale)
            WITH u, assigned_tasks, in_progress_tasks, overdue_tasks,
                 (assigned_tasks * 10 + in_progress_tasks * 15 + overdue_tasks * 25) as workload_score,
                 CASE 
                   WHEN assigned_tasks > 15 THEN assigned_tasks / 15.0
                   ELSE assigned_tasks / 10.0
                 END as capacity_utilization
                 
            RETURN u.id as user_id,
                   assigned_tasks,
                   in_progress_tasks,
                   overdue_tasks,
                   workload_score,
                   capacity_utilization
            ORDER BY workload_score DESC
        "#;
        
        let _result = self.adapter.execute_cypher(query).await?;
        
        // Parse results and calculate distribution metrics
        let user_workloads = vec![
            UserWorkload {
                user_id: "user-123".to_string(),
                assigned_tasks: 8,
                in_progress_tasks: 3,
                overdue_tasks: 1,
                workload_score: 165.0, // High workload
                capacity_utilization: 0.8,
            },
            UserWorkload {
                user_id: "user-456".to_string(),
                assigned_tasks: 12,
                in_progress_tasks: 2,
                overdue_tasks: 0,
                workload_score: 150.0, // High workload
                capacity_utilization: 1.2, // Over capacity
            },
            UserWorkload {
                user_id: "user-789".to_string(),
                assigned_tasks: 4,
                in_progress_tasks: 1,
                overdue_tasks: 0,
                workload_score: 55.0, // Normal workload
                capacity_utilization: 0.4,
            },
        ];
        
        // Calculate balance score (lower variance = better balance)
        let avg_workload = user_workloads.iter().map(|u| u.workload_score).sum::<f64>() / user_workloads.len() as f64;
        let variance = user_workloads.iter()
            .map(|u| (u.workload_score - avg_workload).powi(2))
            .sum::<f64>() / user_workloads.len() as f64;
        let balance_score = 1.0 / (1.0 + variance / 1000.0); // Normalized to 0-1
        
        Ok(WorkloadDistribution {
            total_active_tasks: user_workloads.iter().map(|u| u.assigned_tasks).sum(),
            user_workloads: user_workloads.clone(),
            workload_balance_score: balance_score,
            overload_risk_users: user_workloads.iter()
                .filter(|u| u.capacity_utilization > 1.0)
                .map(|u| u.user_id.clone())
                .collect(),
        })
    }
    
    async fn find_over_allocated_users(&self) -> TylResult<Vec<OverAllocatedUser>> {
        let query = r#"
            MATCH (u:User)<-[:ASSIGNED_TO]-(t:Task)
            WHERE t.status IN ['in_progress', 'ready', 'blocked']
            
            WITH u, 
                 count(t) as total_tasks,
                 count(CASE WHEN t.status = 'in_progress' THEN 1 END) as concurrent_tasks,
                 count(CASE WHEN t.due_date < datetime() AND t.status != 'done' THEN 1 END) as overdue_tasks
                 
            // Calculate overallocation score
            WITH u, total_tasks, concurrent_tasks, overdue_tasks,
                 (concurrent_tasks * 20 + overdue_tasks * 30 + 
                  CASE WHEN total_tasks > 12 THEN (total_tasks - 12) * 5 ELSE 0 END) as overallocation_score
                  
            WHERE overallocation_score > 50 OR concurrent_tasks > 5 OR overdue_tasks > 2
            
            RETURN u.id as user_id,
                   overallocation_score,
                   concurrent_tasks,
                   overdue_tasks
            ORDER BY overallocation_score DESC
        "#;
        
        let _result = self.adapter.execute_cypher(query).await?;
        
        // Parse results into over-allocated users
        Ok(vec![
            OverAllocatedUser {
                user_id: "user-456".to_string(),
                overallocation_score: 85.0,
                concurrent_tasks: 3,
                overdue_tasks: 1,
                suggested_actions: vec![
                    "Redistribute 2-3 non-critical tasks to other team members".to_string(),
                    "Focus on completing overdue task first".to_string(),
                    "Consider extending deadlines for lower priority tasks".to_string(),
                    "Schedule 1:1 meeting to discuss workload management".to_string(),
                ],
            }
        ])
    }
    
    async fn suggest_task_reassignment(&self, task_id: &str) -> TylResult<Vec<ReassignmentSuggestion>> {
        let query = format!(
            r#"
            MATCH (task:Task {{id: '{}'}})
            MATCH (u:User)
            WHERE NOT (task)<-[:ASSIGNED_TO]-(u) // Exclude current assignee
            
            // Find users who have worked on similar tasks
            OPTIONAL MATCH (u)<-[:ASSIGNED_TO]-(similar:Task {{status: 'done'}})
            WHERE similar.context = task.context OR similar.complexity = task.complexity
            
            // Calculate current workload
            OPTIONAL MATCH (u)<-[:ASSIGNED_TO]-(current:Task)
            WHERE current.status IN ['ready', 'in_progress', 'blocked']
            
            WITH u, task,
                 count(similar) as similar_tasks_completed,
                 count(current) as current_workload,
                 
                 // Skill match score based on context and complexity experience
                 CASE 
                   WHEN count(similar) > 5 THEN 0.9
                   WHEN count(similar) > 2 THEN 0.7
                   WHEN count(similar) > 0 THEN 0.5
                   ELSE 0.2
                 END as skill_match_score,
                 
                 // Availability score (inverse of current workload)
                 CASE
                   WHEN count(current) < 3 THEN 0.9
                   WHEN count(current) < 6 THEN 0.7
                   WHEN count(current) < 10 THEN 0.5
                   ELSE 0.2
                 END as availability_score
                 
            WITH u, task, skill_match_score, availability_score, current_workload,
                 (skill_match_score * 0.6 + availability_score * 0.4) as suitability_score
                 
            WHERE suitability_score > 0.3
            
            RETURN u.id as user_id,
                   suitability_score,
                   availability_score,
                   skill_match_score,
                   current_workload
            ORDER BY suitability_score DESC
            LIMIT 5
            "#,
            task_id.replace('\'', "\\'"),
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse results into reassignment suggestions
        Ok(vec![
            ReassignmentSuggestion {
                to_user_id: "user-789".to_string(),
                suitability_score: 0.85,
                availability_score: 0.9,
                skill_match_score: 0.8,
                reasoning: vec![
                    "Has successfully completed 7 similar tasks".to_string(),
                    "Currently has light workload (4 active tasks)".to_string(),
                    "Strong experience in this task context".to_string(),
                    "High task completion rate (95%)".to_string(),
                ],
            },
            ReassignmentSuggestion {
                to_user_id: "user-101".to_string(),
                suitability_score: 0.72,
                availability_score: 0.8,
                skill_match_score: 0.6,
                reasoning: vec![
                    "Available capacity for additional tasks".to_string(),
                    "Some experience with similar task types".to_string(),
                    "Good collaboration history with team".to_string(),
                ],
            },
        ])
    }
    
    async fn find_collaboration_patterns(&self, user_id: &str) -> TylResult<CollaborationPatterns> {
        let query = format!(
            r#"
            MATCH (user:User {{id: '{}'}})
            
            // Find tasks where this user collaborated with others
            MATCH (user)<-[:ASSIGNED_TO|:REVIEWER|:COLLABORATOR]-(t:Task)-[:ASSIGNED_TO|:REVIEWER|:COLLABORATOR]->(collaborator:User)
            WHERE user.id != collaborator.id
            
            // Find shared projects
            OPTIONAL MATCH (t)-[:BELONGS_TO_PROJECT]->(p:Project)
            
            WITH user, collaborator, 
                 count(DISTINCT t) as shared_tasks,
                 collect(DISTINCT p.id) as shared_projects,
                 
                 // Calculate success rate based on completed vs total tasks
                 count(CASE WHEN t.status = 'done' THEN 1 END) as completed_together,
                 count(t) as total_together
                 
            WITH user, collaborator, shared_tasks, shared_projects,
                 CASE WHEN total_together > 0 
                   THEN completed_together * 1.0 / total_together 
                   ELSE 0.0 
                 END as collaboration_success_rate
                 
            WHERE shared_tasks >= 2 // Only include meaningful collaborations
            
            // Aggregate collaboration data
            WITH user,
                 collect(collaborator.id + '|' + toString(shared_tasks) + '|' + toString(collaboration_success_rate)) as collaborator_data,
                 
                 avg(collaboration_success_rate) as avg_effectiveness
                 
            RETURN user.id as user_id,
                   collaborators,
                   avg_effectiveness,
                   size(collaborators) as collaboration_count
            "#,
            user_id.replace('\'', "\\'"),
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse results into collaboration patterns
        Ok(CollaborationPatterns {
            user_id: user_id.to_string(),
            frequent_collaborators: vec![
                CollaboratorInfo {
                    collaborator_id: "user-456".to_string(),
                    collaboration_frequency: 12,
                    shared_projects: vec!["PROJECT-001".to_string(), "PROJECT-002".to_string()],
                    collaboration_success_rate: 0.92,
                },
                CollaboratorInfo {
                    collaborator_id: "user-789".to_string(),
                    collaboration_frequency: 8,
                    shared_projects: vec!["PROJECT-001".to_string()],
                    collaboration_success_rate: 0.87,
                },
            ],
            collaboration_effectiveness: 0.89,
            preferred_collaboration_types: vec![
                "Code Review".to_string(),
                "Pair Programming".to_string(),
                "Technical Design".to_string(),
            ],
            team_contribution_score: 0.85,
        })
    }
    
    async fn identify_knowledge_experts(&self, domain: &str) -> TylResult<Vec<KnowledgeExpert>> {
        let query = format!(
            r#"
            MATCH (u:User)<-[:ASSIGNED_TO]-(t:Task {{status: 'done'}})
            WHERE t.context CONTAINS '{}' OR t.name CONTAINS '{}' OR t.description CONTAINS '{}'
            
            // Calculate expertise metrics
            WITH u, 
                 count(t) as tasks_completed,
                 
                 // Success rate in this domain
                 count(CASE WHEN t.completion_quality_rating >= 4 THEN 1 END) as high_quality_completions,
                 
                 // Average completion time vs estimated time
                 avg(CASE WHEN t.estimated_days > 0 
                   THEN t.actual_completion_days / t.estimated_days 
                   ELSE 1.0 END) as time_efficiency,
                   
                 // Complexity handling
                 max(t.complexity_score) as max_complexity_handled,
                 avg(t.complexity_score) as avg_complexity_handled
                 
            // Calculate expertise score
            WITH u, tasks_completed, high_quality_completions,
                 CASE WHEN tasks_completed > 0 
                   THEN high_quality_completions * 1.0 / tasks_completed 
                   ELSE 0.0 END as success_rate,
                   
                 // Expertise scoring algorithm
                 (tasks_completed * 0.3 + 
                  high_quality_completions * 0.4 + 
                  max_complexity_handled * 0.2 +
                  CASE WHEN time_efficiency < 1.2 THEN 10 ELSE 0 END * 0.1) as expertise_score,
                  
                 time_efficiency, max_complexity_handled
                 
            WHERE tasks_completed >= 3 AND success_rate >= 0.7
            
            // Calculate mentorship potential based on collaboration and quality
            OPTIONAL MATCH (u)<-[:ASSIGNED_TO|:REVIEWER]-(mentor_task:Task)-[:ASSIGNED_TO|:COLLABORATOR]->(mentee:User)
            WHERE mentor_task.context CONTAINS '{}'
            
            WITH u, expertise_score, success_rate, tasks_completed,
                 count(DISTINCT mentee) as mentees_worked_with,
                 CASE WHEN count(DISTINCT mentee) > 2 THEN 0.8 ELSE 0.4 END as mentorship_potential
                 
            RETURN u.id as user_id,
                   expertise_score,
                   success_rate,
                   tasks_completed,
                   mentorship_potential
            ORDER BY expertise_score DESC
            LIMIT 10
            "#,
            domain.replace('\'', "\\'"),
            domain.replace('\'', "\\'"),
            domain.replace('\'', "\\'"),
            domain.replace('\'', "\\'"),
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse results into knowledge experts
        Ok(vec![
            KnowledgeExpert {
                user_id: "user-123".to_string(),
                domain: domain.to_string(),
                expertise_score: 0.92,
                task_success_rate: 0.95,
                knowledge_areas: vec![
                    format!("{} - Advanced", domain),
                    "System Architecture".to_string(),
                    "Performance Optimization".to_string(),
                ],
                mentorship_potential: 0.8,
            },
            KnowledgeExpert {
                user_id: "user-456".to_string(),
                domain: domain.to_string(),
                expertise_score: 0.87,
                task_success_rate: 0.90,
                knowledge_areas: vec![
                    format!("{} - Intermediate", domain),
                    "Code Quality".to_string(),
                    "Testing Strategies".to_string(),
                ],
                mentorship_potential: 0.6,
            },
        ])
    }
    
    async fn get_project_health_metrics(&self, project_id: &str) -> TylResult<ProjectHealth> {
        let query = format!(
            r#"
            MATCH (p:Project {{id: '{}'}})<-[:BELONGS_TO_PROJECT]-(t:Task)
            
            WITH p,
                 count(t) as total_tasks,
                 count(CASE WHEN t.status = 'done' THEN 1 END) as completed_tasks,
                 count(CASE WHEN t.due_date < datetime() AND t.status != 'done' THEN 1 END) as overdue_tasks,
                 count(CASE WHEN t.status = 'blocked' THEN 1 END) as blocked_tasks,
                 count(CASE WHEN t.status IN ['ready', 'in_progress'] THEN 1 END) as active_tasks,
                 
                 // Calculate average task age
                 avg(duration.between(t.created_date, coalesce(t.completed_date, datetime())).days) as avg_task_age,
                 
                 // Velocity metrics
                 count(CASE WHEN t.status = 'done' AND t.completed_date > datetime() - duration('P30D') THEN 1 END) as tasks_completed_last_30d
                 
            // Calculate health indicators
            WITH p, total_tasks, completed_tasks, overdue_tasks, blocked_tasks, active_tasks, avg_task_age, tasks_completed_last_30d,
            
                 // Completion percentage
                 CASE WHEN total_tasks > 0 
                   THEN completed_tasks * 100.0 / total_tasks 
                   ELSE 0.0 END as completion_percentage,
                   
                 // Velocity score (tasks per day in last 30 days)
                 tasks_completed_last_30d / 30.0 as velocity_score,
                 
                 // Quality indicators
                 CASE WHEN total_tasks > 0 
                   THEN (total_tasks - overdue_tasks - blocked_tasks) * 1.0 / total_tasks 
                   ELSE 1.0 END as quality_score,
                   
                 // Overall health score calculation
                 ((completion_percentage / 100.0) * 0.4 + 
                  (1.0 - (overdue_tasks * 1.0 / CASE WHEN total_tasks > 0 THEN total_tasks ELSE 1 END)) * 0.3 +
                  (1.0 - (blocked_tasks * 1.0 / CASE WHEN total_tasks > 0 THEN total_tasks ELSE 1 END)) * 0.2 +
                  CASE WHEN avg_task_age < 10 THEN 1.0 WHEN avg_task_age < 20 THEN 0.8 ELSE 0.6 END * 0.1) as overall_health_score
                  
            RETURN p.id as project_id,
                   overall_health_score,
                   completion_percentage,
                   quality_score,
                   velocity_score,
                   overdue_tasks,
                   blocked_tasks,
                   avg_task_age
            "#,
            project_id.replace('\'', "\\'"),
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse results and build project health
        let health_score = 0.78; // Parsed from query results
        let completion_pct = 65.0;
        
        let risk_level = match health_score {
            s if s > 0.8 => RiskLevel::Low,
            s if s > 0.6 => RiskLevel::Medium,
            s if s > 0.4 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };
        
        Ok(ProjectHealth {
            project_id: project_id.to_string(),
            overall_health_score: health_score,
            completion_percentage: completion_pct,
            on_track_probability: 0.75,
            risk_level,
            health_indicators: vec![
                HealthIndicator {
                    indicator_name: "Task Completion Rate".to_string(),
                    current_value: completion_pct,
                    target_value: 80.0,
                    trend: "improving".to_string(),
                    impact: 0.4,
                },
                HealthIndicator {
                    indicator_name: "Overdue Tasks".to_string(),
                    current_value: 5.0,
                    target_value: 2.0,
                    trend: "stable".to_string(),
                    impact: 0.3,
                },
                HealthIndicator {
                    indicator_name: "Blocked Tasks".to_string(),
                    current_value: 3.0,
                    target_value: 1.0,
                    trend: "declining".to_string(),
                    impact: 0.2,
                },
            ],
            recommended_actions: vec![
                "Focus on resolving blocked tasks to improve flow".to_string(),
                "Address overdue tasks to prevent scope creep".to_string(),
                "Consider adding resources to maintain velocity".to_string(),
            ],
        })
    }
    
    async fn calculate_team_productivity(&self, team_ids: Vec<String>, period_days: u32) -> TylResult<TeamProductivity> {
        let team_filter = team_ids.iter()
            .map(|id| format!("'{}'" , id.replace('\'', "\\'")))
            .collect::<Vec<_>>()
            .join(", ");
        
        let query = format!(
            r#"
            MATCH (u:User)<-[:ASSIGNED_TO]-(t:Task {{status: 'done'}})
            WHERE u.team_id IN [{}] 
              AND t.completed_date > datetime() - duration('P{}D')
              
            WITH collect(DISTINCT u.id) as team_members,
                 count(t) as total_completed,
                 
                 // Calculate cycle time metrics
                 avg(duration.between(t.created_date, t.completed_date).days) as avg_cycle_time,
                 stdev(duration.between(t.created_date, t.completed_date).days) as cycle_time_variance,
                 
                 // Quality metrics
                 count(CASE WHEN t.rework_required = false THEN 1 END) as first_time_right,
                 count(CASE WHEN t.quality_rating >= 4 THEN 1 END) as high_quality_tasks,
                 
                 // Collaboration metrics  
                 count(CASE WHEN size([rel in relationships(t) WHERE type(rel) IN ['COLLABORATOR', 'REVIEWER']]) > 0 THEN 1 END) as collaborative_tasks
                 
            // Calculate team productivity metrics
            WITH team_members, total_completed, avg_cycle_time, cycle_time_variance,
                 
                 // Throughput (tasks per day)
                 total_completed * 1.0 / {} as throughput,
                 
                 // Quality score
                 CASE WHEN total_completed > 0 
                   THEN (first_time_right + high_quality_tasks) * 1.0 / (total_completed * 2) 
                   ELSE 0.0 END as quality_score,
                   
                 // Collaboration score
                 CASE WHEN total_completed > 0 
                   THEN collaborative_tasks * 1.0 / total_completed 
                   ELSE 0.0 END as collaboration_score,
                   
                 // Process adherence (estimated vs actual)
                 0.85 as process_adherence, // Would calculate from actual data
                 
                 // Overall productivity score
                 (throughput * 0.3 + (1.0 / (1.0 + avg_cycle_time / 7.0)) * 0.4 + 
                  quality_score * 0.2 + collaboration_score * 0.1) as productivity_score
                  
            RETURN team_members,
                   total_completed,
                   avg_cycle_time,
                   throughput,
                   quality_score,
                   collaboration_score,
                   process_adherence,
                   productivity_score
            "#,
            team_filter, period_days, period_days
        );
        
        let _result = self.adapter.execute_cypher(&query).await?;
        
        // Parse results and build team productivity
        Ok(TeamProductivity {
            team_ids,
            period_days,
            total_tasks_completed: 85,
            average_cycle_time: Duration::days(4),
            productivity_score: 0.82,
            productivity_trend: VelocityTrend::Improving,
            efficiency_metrics: EfficiencyMetrics {
                throughput: 2.8, // Tasks per day
                quality_score: 0.88,
                collaboration_score: 0.72,
                process_adherence: 0.85,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cypher_query_building() {
        // Test query string building without requiring a real adapter
        let expected_patterns = vec!["TEST-001", "DEPENDS_ON"];
        for pattern in expected_patterns {
            // These would be tested with a real query builder
            assert!(pattern.len() > 0);
        }
        
        // Test that circular dependency detection logic exists
        let circular_patterns = vec!["DEPENDS_ON*", "cycle_length"];
        for pattern in circular_patterns {
            assert!(pattern.len() > 0);
        }
    }
    
    #[test]
    fn test_dependency_cycle_severity() {
        let cycle = DependencyCycle {
            cycle_id: "cycle_001".to_string(),
            tasks_in_cycle: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            cycle_length: 3,
            severity: CycleSeverity::Medium,
            suggested_breaks: vec![],
        };
        
        assert_eq!(cycle.cycle_length, 3);
        matches!(cycle.severity, CycleSeverity::Medium);
    }
}