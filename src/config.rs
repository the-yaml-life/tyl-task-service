//! Configuration management for the microservice
//!
//! This module handles loading and validating configuration from environment variables
//! and configuration files using the TYL framework patterns.

use serde::{Deserialize, Serialize};
use std::env;

/// Main configuration for the task service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskServiceConfig {
    /// Service identification
    pub service_name: String,
    pub version: String,
    
    /// API server configuration
    pub api: ApiConfig,
    
    /// Database configuration (optional)
    pub database: Option<DatabaseConfig>,
    
    /// External services configuration
    pub external: ExternalConfig,
    
    /// Logging and monitoring
    pub monitoring: MonitoringConfig,
}

/// API server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout_ms: u64,
    pub max_request_size: usize,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout_ms: u64,
    pub query_timeout_ms: u64,
}

/// External services configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalConfig {
    pub timeout_ms: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

/// Monitoring and observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
    pub health_check_enabled: bool,
}

impl TaskServiceConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            service_name: get_env("TYL_SERVICE_NAME")
                .unwrap_or_else(|| "tyl-task-service".to_string()),
            version: get_env("TYL_VERSION")
                .unwrap_or_else(|| "1.0.0".to_string()),
            
            api: ApiConfig {
                host: get_env("HOST").unwrap_or_else(|| "0.0.0.0".to_string()),
                port: get_env("PORT")
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(3000),
                request_timeout_ms: get_env("REQUEST_TIMEOUT_MS")
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(30000),
                max_request_size: get_env("MAX_REQUEST_SIZE")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1024 * 1024), // 1MB default
            },
            
            database: get_env("DATABASE_URL").map(|url| DatabaseConfig {
                url,
                max_connections: get_env("DB_MAX_CONNECTIONS")
                    .and_then(|c| c.parse().ok())
                    .unwrap_or(10),
                connection_timeout_ms: get_env("DB_CONNECTION_TIMEOUT_MS")
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(5000),
                query_timeout_ms: get_env("DB_QUERY_TIMEOUT_MS")
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(10000),
            }),
            
            external: ExternalConfig {
                timeout_ms: get_env("EXTERNAL_TIMEOUT_MS")
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(10000),
                retry_attempts: get_env("EXTERNAL_RETRY_ATTEMPTS")
                    .and_then(|r| r.parse().ok())
                    .unwrap_or(3),
                retry_delay_ms: get_env("EXTERNAL_RETRY_DELAY_MS")
                    .and_then(|d| d.parse().ok())
                    .unwrap_or(1000),
            },
            
            monitoring: MonitoringConfig {
                metrics_enabled: get_env("METRICS_ENABLED")
                    .and_then(|e| e.parse().ok())
                    .unwrap_or(true),
                tracing_enabled: get_env("TRACING_ENABLED")
                    .and_then(|e| e.parse().ok())
                    .unwrap_or(true),
                health_check_enabled: get_env("HEALTH_CHECK_ENABLED")
                    .and_then(|e| e.parse().ok())
                    .unwrap_or(true),
            },
        })
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if self.service_name.is_empty() {
            return Err("Service name cannot be empty".to_string());
        }
        
        if self.api.port == 0 {
            return Err("API port must be greater than 0".to_string());
        }
        
        if let Some(ref db_config) = self.database {
            if db_config.url.is_empty() {
                return Err("Database URL cannot be empty when database is configured".to_string());
            }
        }
        
        Ok(())
    }
}

impl Default for TaskServiceConfig {
    fn default() -> Self {
        Self {
            service_name: "tyl-task-service".to_string(),
            version: "1.0.0".to_string(),
            api: ApiConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
                request_timeout_ms: 30000,
                max_request_size: 1024 * 1024,
            },
            database: None,
            external: ExternalConfig {
                timeout_ms: 10000,
                retry_attempts: 3,
                retry_delay_ms: 1000,
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                tracing_enabled: true,
                health_check_enabled: true,
            },
        }
    }
}

/// Helper function to get environment variable
fn get_env(key: &str) -> Option<String> {
    env::var(key).ok().filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TaskServiceConfig::default();
        assert_eq!(config.service_name, "tyl-task-service");
        assert_eq!(config.api.port, 3000);
        assert!(config.monitoring.health_check_enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = TaskServiceConfig::default();
        assert!(config.validate().is_ok());
        
        config.service_name = String::new();
        assert!(config.validate().is_err());
        
        config.service_name = "test-service".to_string();
        config.api.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_env_loading() {
        // Test with empty environment
        let config = TaskServiceConfig::from_env().unwrap();
        assert!(config.validate().is_ok());
    }
}