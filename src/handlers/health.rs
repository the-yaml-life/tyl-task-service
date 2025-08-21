//! Health check handlers
//!
//! Provides health check endpoints for monitoring and load balancing.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::{AppState, LogLevel, LogRecord};
use tokio::time::{timeout, Duration};

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
}

/// Detailed health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthDetailResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
    pub dependencies: DependencyHealth,
}

/// Dependency health status
#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyHealth {
    pub database: DependencyStatus,
    pub event_system: DependencyStatus,
    pub domain_service: DependencyStatus,
}

/// Individual dependency status with details
#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub status: HealthStatus,
    pub name: String,
    pub message: Option<String>,
    pub response_time_ms: Option<u64>,
}

/// Health status enumeration
#[derive(Debug, Serialize, Deserialize)]
pub enum HealthStatus {
    #[serde(rename = "healthy")]
    Healthy,
    #[serde(rename = "unhealthy")]
    Unhealthy,
    #[serde(rename = "unknown")]
    Unknown,
}

/// Basic health check endpoint
/// 
/// Returns a simple health status for basic monitoring.
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: state.config.service_name.clone(),
        version: state.config.version.clone(),
        timestamp: chrono::Utc::now(),
        uptime_seconds: get_uptime_seconds(),
    })
}

/// Readiness probe endpoint
/// 
/// Checks if the service is ready to accept traffic.
/// This should return 200 when the service can handle requests.
pub async fn readiness_check(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    // Check if service is ready (databases connected, etc.)
    let is_ready = check_service_readiness(&state).await;
    
    if is_ready {
        Ok(Json(HealthResponse {
            status: "ready".to_string(),
            service: state.config.service_name.clone(),
            version: state.config.version.clone(),
            timestamp: chrono::Utc::now(),
            uptime_seconds: get_uptime_seconds(),
        }))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Liveness probe endpoint
/// 
/// Checks if the service is alive and should not be restarted.
/// This should return 200 unless the service is in an unrecoverable state.
pub async fn liveness_check(State(state): State<AppState>) -> Json<HealthResponse> {
    // Basic liveness check - service is running if this handler is called
    Json(HealthResponse {
        status: "alive".to_string(),
        service: state.config.service_name.clone(),
        version: state.config.version.clone(),
        timestamp: chrono::Utc::now(),
        uptime_seconds: get_uptime_seconds(),
    })
}

/// Detailed health check endpoint
/// 
/// Returns comprehensive health information including dependency status.
pub async fn health_detail(State(state): State<AppState>) -> Json<HealthDetailResponse> {
    let dependency_health = check_dependencies(&state).await;
    
    Json(HealthDetailResponse {
        status: determine_overall_status(&dependency_health),
        service: state.config.service_name.clone(),
        version: state.config.version.clone(),
        timestamp: chrono::Utc::now(),
        uptime_seconds: get_uptime_seconds(),
        dependencies: dependency_health,
    })
}

/// Check if the service is ready to accept traffic
async fn check_service_readiness(state: &AppState) -> bool {
    // Log readiness check start
    state.logger.log(&LogRecord::new(LogLevel::Debug, "Starting service readiness check"));
    
    // Check all critical dependencies
    let dependencies = check_dependencies(state).await;
    
    // Service is ready if database and event system are healthy
    let is_ready = matches!(dependencies.database.status, HealthStatus::Healthy) &&
                   matches!(dependencies.event_system.status, HealthStatus::Healthy) &&
                   matches!(dependencies.domain_service.status, HealthStatus::Healthy);
    
    state.logger.log(&LogRecord::new(LogLevel::Info, 
        &format!("Service readiness check result: {}", if is_ready { "ready" } else { "not ready" })));
    
    is_ready
}

/// Check the health of all dependencies
async fn check_dependencies(state: &AppState) -> DependencyHealth {
    // Run all dependency checks concurrently for better performance
    let (database_result, event_system_result, domain_service_result) = tokio::join!(
        check_database_health(state),
        check_event_system_health(state),
        check_domain_service_health(state)
    );
    
    DependencyHealth {
        database: database_result,
        event_system: event_system_result,
        domain_service: domain_service_result,
    }
}

/// Check database health
async fn check_database_health(state: &AppState) -> DependencyStatus {
    let start_time = std::time::Instant::now();
    let db_name = format!("FalkorDB ({})", state.config.database.graph_name);
    
    // Try to perform a simple health check query with timeout
    let health_check_result = timeout(Duration::from_secs(5), async {
        // Try to create a simple test task to verify database connectivity
        let test_filter = crate::domain::TaskFilter {
            search_text: Some("health-check-non-existent".to_string()),
            limit: Some(1),
            ..Default::default()
        };
        
        // This should return an empty result but proves database connectivity
        state.domain_service.list_tasks(test_filter).await
    }).await;
    
    let response_time = start_time.elapsed().as_millis() as u64;
    
    match health_check_result {
        Ok(Ok(_)) => {
            state.logger.log(&LogRecord::new(LogLevel::Debug, "Database health check: SUCCESS"));
            DependencyStatus {
                status: HealthStatus::Healthy,
                name: db_name,
                message: Some("Connection verified".to_string()),
                response_time_ms: Some(response_time),
            }
        },
        Ok(Err(e)) => {
            let error_msg = format!("Database query failed: {}", e);
            state.logger.log(&LogRecord::new(LogLevel::Error, &format!("Database health check: {}", error_msg)));
            DependencyStatus {
                status: HealthStatus::Unhealthy,
                name: db_name,
                message: Some(error_msg),
                response_time_ms: Some(response_time),
            }
        },
        Err(_) => {
            let error_msg = "Database health check timeout (>5s)".to_string();
            state.logger.log(&LogRecord::new(LogLevel::Error, &format!("Database health check: {}", error_msg)));
            DependencyStatus {
                status: HealthStatus::Unhealthy,
                name: db_name,
                message: Some(error_msg),
                response_time_ms: Some(response_time),
            }
        }
    }
}

/// Check event system health
async fn check_event_system_health(state: &AppState) -> DependencyStatus {
    let start_time = std::time::Instant::now();
    
    // Simple check - if event service is configured and available
    let is_healthy = state.config.events.enabled;
    let response_time = start_time.elapsed().as_millis() as u64;
    
    if is_healthy {
        state.logger.log(&LogRecord::new(LogLevel::Debug, "Event system health check: SUCCESS"));
        DependencyStatus {
            status: HealthStatus::Healthy,
            name: "Event System".to_string(),
            message: Some("Event service is enabled and configured".to_string()),
            response_time_ms: Some(response_time),
        }
    } else {
        state.logger.log(&LogRecord::new(LogLevel::Warn, "Event system health check: DISABLED"));
        DependencyStatus {
            status: HealthStatus::Unknown,
            name: "Event System".to_string(),
            message: Some("Event service is disabled in configuration".to_string()),
            response_time_ms: Some(response_time),
        }
    }
}

/// Check domain service health  
async fn check_domain_service_health(state: &AppState) -> DependencyStatus {
    let start_time = std::time::Instant::now();
    
    // The domain service is healthy if it's initialized and available
    // Since we have it in our state, it's available
    let response_time = start_time.elapsed().as_millis() as u64;
    
    state.logger.log(&LogRecord::new(LogLevel::Debug, "Domain service health check: SUCCESS"));
    DependencyStatus {
        status: HealthStatus::Healthy,
        name: "Task Domain Service".to_string(), 
        message: Some("Domain service is initialized and available".to_string()),
        response_time_ms: Some(response_time),
    }
}

/// Determine overall health status from dependencies
fn determine_overall_status(dependencies: &DependencyHealth) -> String {
    let db_status = &dependencies.database.status;
    let event_status = &dependencies.event_system.status; 
    let domain_status = &dependencies.domain_service.status;
    
    // Service is healthy only if all critical components are healthy
    match (db_status, event_status, domain_status) {
        (HealthStatus::Healthy, HealthStatus::Healthy, HealthStatus::Healthy) => "healthy".to_string(),
        (HealthStatus::Healthy, HealthStatus::Unknown, HealthStatus::Healthy) => "healthy".to_string(), // Events can be optional
        (HealthStatus::Unhealthy, _, _) => "unhealthy".to_string(), // Database is critical
        (_, _, HealthStatus::Unhealthy) => "unhealthy".to_string(), // Domain service is critical
        _ => "degraded".to_string(),
    }
}

/// Get service uptime in seconds
fn get_uptime_seconds() -> u64 {
    // This is a simplified implementation
    // In a real service, you'd track startup time and calculate actual uptime
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() % 86400 // Simplified: show uptime as seconds in current day
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
            tracer: Arc::new(tyl_tracing::SimpleTracer::new(tyl_tracing::TraceConfig::new("test-service"))),
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let state = create_test_state().await;
        let response = health_check(State(state)).await;
        
        assert_eq!(response.status, "healthy");
        assert!(!response.service.is_empty());
    }

    #[tokio::test]
    async fn test_readiness_check() {
        let state = create_test_state().await;
        let result = readiness_check(State(state)).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, "ready");
    }

    #[tokio::test]
    async fn test_liveness_check() {
        let state = create_test_state().await;
        let response = liveness_check(State(state)).await;
        
        assert_eq!(response.status, "alive");
    }

    #[tokio::test]
    async fn test_health_detail() {
        let state = create_test_state().await;
        let response = health_detail(State(state)).await;
        
        assert!(!response.status.is_empty());
        assert!(!response.service.is_empty());
        
        // Check that all dependencies are present
        assert_eq!(response.dependencies.database.name, "FalkorDB (tyl_tasks)");
        assert_eq!(response.dependencies.event_system.name, "Event System");
        assert_eq!(response.dependencies.domain_service.name, "Task Domain Service");
        
        // Check that response times are recorded
        assert!(response.dependencies.database.response_time_ms.is_some());
        assert!(response.dependencies.event_system.response_time_ms.is_some());
        assert!(response.dependencies.domain_service.response_time_ms.is_some());
    }
    
    #[tokio::test]
    async fn test_dependency_health_checks() {
        let state = create_test_state().await;
        let dependencies = check_dependencies(&state).await;
        
        // Database should be healthy (using MockTaskService)
        assert!(matches!(dependencies.database.status, HealthStatus::Healthy));
        assert!(dependencies.database.message.is_some());
        
        // Event system should be healthy (enabled by default)
        assert!(matches!(dependencies.event_system.status, HealthStatus::Healthy));
        
        // Domain service should be healthy (mock service)
        assert!(matches!(dependencies.domain_service.status, HealthStatus::Healthy));
    }
    
    #[tokio::test]
    async fn test_service_readiness() {
        let state = create_test_state().await;
        let is_ready = check_service_readiness(&state).await;
        
        // Service should be ready with mock components
        assert!(is_ready);
    }
}