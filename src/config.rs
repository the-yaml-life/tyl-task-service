//! Configuration management for the microservice
//!
//! This module handles loading and validating configuration from environment variables
//! and configuration files using the TYL framework patterns.

// Re-export TYL framework functionality
pub use tyl_config::{ConfigResult, RedisConfig};
pub use tyl_errors::{TylError, TylResult};

use serde::{Deserialize, Serialize};

/// Main configuration for the task service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskServiceConfig {
    /// Service identification
    pub service_name: String,
    pub version: String,
    
    /// API server configuration
    pub api: ApiConfig,
    
    /// Database configuration (required for FalkorDB)
    pub database: DatabaseConfig,
    
    /// External services configuration
    pub external: ExternalConfig,
    
    /// Event system configuration
    pub events: EventConfig,
    
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

/// FalkorDB database configuration - extends tyl-config RedisConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Redis connection configuration
    pub redis: RedisConfig,
    /// Graph database name in FalkorDB
    pub graph_name: String,
    /// Query timeout in milliseconds
    pub query_timeout_ms: u64,
}

/// External services configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalConfig {
    pub timeout_ms: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

/// Event system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventConfig {
    pub enabled: bool,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub batch_size: usize,
}

/// Monitoring and observability configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
    pub health_check_enabled: bool,
    pub log_level: String,
    pub log_format: String, // "console" or "json"
    pub trace_sampling_rate: f64,
    pub max_spans: usize,
}

impl TaskServiceConfig {
    /// Load configuration using environment variables with sensible defaults
    pub fn from_env() -> ConfigResult<Self> {
        Ok(Self {
            service_name: std::env::var("TYL_TASK_SERVICE_SERVICE_NAME")
                .unwrap_or_else(|_| "tyl-task-service".to_string()),
            version: std::env::var("TYL_TASK_SERVICE_VERSION")
                .unwrap_or_else(|_| "1.0.0".to_string()),
            
            api: ApiConfig {
                host: std::env::var("TYL_TASK_SERVICE_API_HOST")
                    .or_else(|_| std::env::var("HOST"))
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("TYL_TASK_SERVICE_API_PORT")
                    .or_else(|_| std::env::var("PORT"))
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(3000),
                request_timeout_ms: std::env::var("TYL_TASK_SERVICE_API_REQUEST_TIMEOUT_MS")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(30000),
                max_request_size: std::env::var("TYL_TASK_SERVICE_API_MAX_REQUEST_SIZE")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(1024 * 1024), // 1MB default
            },
            
            database: DatabaseConfig {
                redis: RedisConfig {
                    url: None,
                    host: std::env::var("TYL_TASK_SERVICE_DATABASE_REDIS_HOST")
                        .or_else(|_| std::env::var("FALKORDB_HOST"))
                        .unwrap_or_else(|_| "localhost".to_string()),
                    port: std::env::var("TYL_TASK_SERVICE_DATABASE_REDIS_PORT")
                        .or_else(|_| std::env::var("FALKORDB_PORT"))
                        .ok()
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(6379),
                    password: std::env::var("TYL_TASK_SERVICE_DATABASE_REDIS_PASSWORD")
                        .or_else(|_| std::env::var("FALKORDB_PASSWORD"))
                        .ok(),
                    database: std::env::var("TYL_TASK_SERVICE_DATABASE_REDIS_DATABASE")
                        .ok()
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(0),
                    pool_size: std::env::var("TYL_TASK_SERVICE_DATABASE_REDIS_POOL_SIZE")
                        .ok()
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(10),
                    timeout_seconds: std::env::var("TYL_TASK_SERVICE_DATABASE_REDIS_TIMEOUT_SECONDS")
                        .ok()
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(5),
                },
                graph_name: std::env::var("TYL_TASK_SERVICE_DATABASE_GRAPH_NAME")
                    .or_else(|_| std::env::var("FALKORDB_GRAPH_NAME"))
                    .unwrap_or_else(|_| "tyl_tasks".to_string()),
                query_timeout_ms: std::env::var("TYL_TASK_SERVICE_DATABASE_QUERY_TIMEOUT_MS")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(10000),
            },
            
            external: ExternalConfig {
                timeout_ms: std::env::var("TYL_TASK_SERVICE_EXTERNAL_TIMEOUT_MS")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(10000),
                retry_attempts: std::env::var("TYL_TASK_SERVICE_EXTERNAL_RETRY_ATTEMPTS")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(3),
                retry_delay_ms: std::env::var("TYL_TASK_SERVICE_EXTERNAL_RETRY_DELAY_MS")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(1000),
            },
            
            events: EventConfig {
                enabled: std::env::var("TYL_TASK_SERVICE_EVENTS_ENABLED")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true),
                retry_attempts: std::env::var("TYL_TASK_SERVICE_EVENTS_RETRY_ATTEMPTS")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(3),
                retry_delay_ms: std::env::var("TYL_TASK_SERVICE_EVENTS_RETRY_DELAY_MS")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(1000),
                batch_size: std::env::var("TYL_TASK_SERVICE_EVENTS_BATCH_SIZE")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(10),
            },
            
            monitoring: MonitoringConfig {
                metrics_enabled: std::env::var("TYL_TASK_SERVICE_MONITORING_METRICS_ENABLED")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true),
                tracing_enabled: std::env::var("TYL_TASK_SERVICE_MONITORING_TRACING_ENABLED")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true),
                health_check_enabled: std::env::var("TYL_TASK_SERVICE_MONITORING_HEALTH_CHECK_ENABLED")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true),
                log_level: std::env::var("TYL_TASK_SERVICE_MONITORING_LOG_LEVEL")
                    .or_else(|_| std::env::var("RUST_LOG"))
                    .unwrap_or_else(|_| "info".to_string()),
                log_format: std::env::var("TYL_TASK_SERVICE_MONITORING_LOG_FORMAT")
                    .or_else(|_| std::env::var("TYL_LOG_FORMAT"))
                    .unwrap_or_else(|_| "console".to_string()),
                trace_sampling_rate: std::env::var("TYL_TASK_SERVICE_MONITORING_TRACE_SAMPLING_RATE")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(1.0),
                max_spans: std::env::var("TYL_TASK_SERVICE_MONITORING_MAX_SPANS")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(1000),
            },
        })
    }

    /// Validate configuration values
    pub fn validate(&self) -> ConfigResult<()> {
        if self.service_name.is_empty() {
            return Err(TylError::configuration("Service name cannot be empty"));
        }
        
        if self.api.port == 0 {
            return Err(TylError::configuration("API port must be greater than 0"));
        }
        
        if self.database.redis.host.is_empty() {
            return Err(TylError::configuration("Database host cannot be empty"));
        }
        
        if self.database.graph_name.is_empty() {
            return Err(TylError::configuration("Graph name cannot be empty"));
        }
        
        if self.database.redis.port == 0 {
            return Err(TylError::configuration("Database port must be greater than 0"));
        }
        
        // Validate log level
        let valid_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_levels.contains(&self.monitoring.log_level.as_str()) {
            return Err(TylError::configuration(
                format!("Invalid log level '{}'. Must be one of: {}", 
                    self.monitoring.log_level, valid_levels.join(", "))
            ));
        }
        
        // Validate log format
        let valid_formats = ["console", "json"];
        if !valid_formats.contains(&self.monitoring.log_format.as_str()) {
            return Err(TylError::configuration(
                format!("Invalid log format '{}'. Must be one of: {}", 
                    self.monitoring.log_format, valid_formats.join(", "))
            ));
        }
        
        // Validate trace sampling rate
        if self.monitoring.trace_sampling_rate < 0.0 || self.monitoring.trace_sampling_rate > 1.0 {
            return Err(TylError::configuration(
                "Trace sampling rate must be between 0.0 and 1.0".to_string()
            ));
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
            database: DatabaseConfig {
                redis: RedisConfig::default(),
                graph_name: "tyl_tasks".to_string(),
                query_timeout_ms: 10000,
            },
            external: ExternalConfig {
                timeout_ms: 10000,
                retry_attempts: 3,
                retry_delay_ms: 1000,
            },
            events: EventConfig {
                enabled: true,
                retry_attempts: 3,
                retry_delay_ms: 1000,
                batch_size: 10,
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                tracing_enabled: true,
                health_check_enabled: true,
                log_level: "info".to_string(),
                log_format: "console".to_string(),
                trace_sampling_rate: 1.0,
                max_spans: 1000,
            },
        }
    }
}


/// Configuration utilities
impl TaskServiceConfig {
    /// Create configuration for testing with minimal setup
    pub fn for_testing() -> Self {
        Self {
            database: DatabaseConfig {
                redis: RedisConfig {
                    host: "localhost".to_string(),
                    port: 6379,
                    password: None,
                    database: 15, // Use test database
                    pool_size: 5,
                    timeout_seconds: 1,
                    ..Default::default()
                },
                graph_name: "tyl_tasks_test".to_string(),
                query_timeout_ms: 5000,
            },
            monitoring: MonitoringConfig {
                log_level: "debug".to_string(),
                log_format: "console".to_string(),
                trace_sampling_rate: 1.0,
                max_spans: 100,
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    /// Get connection string for logging (without password)
    pub fn database_connection_info(&self) -> String {
        format!("{}:{}@{}", 
            self.database.redis.host, 
            self.database.redis.port, 
            self.database.graph_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TaskServiceConfig::default();
        assert_eq!(config.service_name, "tyl-task-service");
        assert_eq!(config.api.port, 3000);
        assert_eq!(config.database.redis.host, "localhost");
        assert_eq!(config.database.redis.port, 6379);
        assert_eq!(config.database.graph_name, "tyl_tasks");
        assert!(config.monitoring.health_check_enabled);
        assert!(config.events.enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = TaskServiceConfig::default();
        assert!(config.validate().is_ok());
        
        // Test empty service name
        config.service_name = String::new();
        assert!(config.validate().is_err());
        
        // Reset and test invalid port
        config.service_name = "test-service".to_string();
        config.api.port = 0;
        assert!(config.validate().is_err());
        
        // Reset and test empty database host
        config.api.port = 3000;
        config.database.redis.host = String::new();
        assert!(config.validate().is_err());
        
        // Reset and test invalid log level
        config.database.redis.host = "localhost".to_string();
        config.monitoring.log_level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_env_loading() {
        // Test with empty environment - should work with defaults
        let config = TaskServiceConfig::from_env().unwrap();
        assert!(config.validate().is_ok());
        assert_eq!(config.database.redis.host, "localhost");
        assert_eq!(config.database.redis.port, 6379);
    }
    
    #[test]
    fn test_database_config_fields() {
        let config = TaskServiceConfig::default();
        
        // Verify all required fields are present for FalkorDB
        assert!(!config.database.redis.host.is_empty());
        assert!(config.database.redis.port > 0);
        assert!(!config.database.graph_name.is_empty());
        assert!(config.database.redis.timeout_seconds > 0);
        assert!(config.database.query_timeout_ms > 0);
        assert!(config.database.redis.pool_size > 0);
    }
    
    #[test]
    fn test_event_config() {
        let config = TaskServiceConfig::default();
        
        // Verify event system configuration
        assert!(config.events.enabled);
        assert!(config.events.retry_attempts > 0);
        assert!(config.events.retry_delay_ms > 0);
        assert!(config.events.batch_size > 0);
    }
}