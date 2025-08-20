//! Health check handlers
//!
//! Provides health check endpoints for monitoring and load balancing.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

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
    pub database: HealthStatus,
    pub external_services: HealthStatus,
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
    // Add your readiness checks here:
    // - Database connectivity
    // - Required external services
    // - Configuration validation
    // - Cache warming
    
    // For template purposes, always return true
    // In a real implementation, you would check actual dependencies
    true
}

/// Check the health of all dependencies
async fn check_dependencies(state: &AppState) -> DependencyHealth {
    DependencyHealth {
        database: check_database_health(state).await,
        external_services: check_external_services_health(state).await,
    }
}

/// Check database health
async fn check_database_health(state: &AppState) -> HealthStatus {
    // Check database connectivity
    // This would typically involve a simple query or ping
    
    if state.config.database.is_some() {
        // In a real implementation, you would test the database connection
        HealthStatus::Healthy
    } else {
        HealthStatus::Unknown
    }
}

/// Check external services health
async fn check_external_services_health(_state: &AppState) -> HealthStatus {
    // Check external service connectivity
    // This would typically involve health check requests to upstream services
    
    HealthStatus::Healthy
}

/// Determine overall health status from dependencies
fn determine_overall_status(dependencies: &DependencyHealth) -> String {
    match (&dependencies.database, &dependencies.external_services) {
        (HealthStatus::Healthy, HealthStatus::Healthy) => "healthy".to_string(),
        (HealthStatus::Unknown, HealthStatus::Healthy) => "healthy".to_string(),
        (HealthStatus::Healthy, HealthStatus::Unknown) => "healthy".to_string(),
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
    }
}