//! # TYL TaskService Microservice
//!
//! Task management microservice built with TYL framework and event-driven architecture
//!
//! ## Features
//!
//! - RESTful HTTP API with Axum
//! - Hexagonal architecture with ports and adapters
//! - Event-driven architecture with TYL PubSub
//! - Async-first design with Tokio
//! - Comprehensive error handling with TYL framework
//! - Structured logging and distributed tracing
//! - Configuration management
//! - Health check endpoints
//! - Full test coverage
//!
//! ## Quick Start
//!
//! ```rust
//! use tyl_task_service::{TaskServiceConfig, create_app};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = TaskServiceConfig::from_env()?;
//!     let app = create_app(config).await?;
//!     
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
//!     axum::serve(listener, app).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! This microservice follows hexagonal architecture:
//!
//! - **Domain Layer**: Core business logic and domain models
//! - **Application Layer**: Use cases and application services
//! - **Infrastructure Layer**: Adapters for external systems (DB, HTTP, etc.)
//! - **API Layer**: HTTP handlers and route definitions
//!
//! ## Examples
//!
//! See the `examples/` directory for complete usage examples.

// Re-export TYL framework functionality
pub use tyl_errors::{TylError, TylResult};
pub use tyl_config::ConfigManager;
pub use tyl_logging::{Logger, LogRecord};
pub use tyl_tracing::Span;

// Standard library imports
use std::sync::Arc;

// External crates
use axum::Router;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

// Internal modules
pub mod config;
pub mod domain;
pub mod handlers;
pub mod adapters;
pub mod routes;
pub mod events;

// Re-exports for convenience
pub use config::{TaskServiceConfig, DatabaseConfig, ApiConfig};
pub use domain::{TaskService, Task, CreateTaskRequest, TaskResponse};
pub use events::{EventService, DomainEventHandler};

/// Result type for task service operations
pub type TaskServiceResult<T> = Result<T, TaskServiceError>;

/// Errors that can occur during task service operations
#[derive(Debug, thiserror::Error)]
pub enum TaskServiceError {
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    #[error("Database error: {message}")]
    Database { message: String },
    
    #[error("API error: {message}")]
    Api { message: String },
    
    #[error("Domain logic error: {message}")]
    Domain { message: String },
    
    #[error("External service error: {message}")]
    ExternalService { message: String },
    
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
}

impl From<TaskServiceError> for TylError {
    fn from(err: TaskServiceError) -> Self {
        match err {
            TaskServiceError::Configuration { message } => TylError::configuration(message),
            TaskServiceError::Database { message } => TylError::database(message),
            TaskServiceError::Api { message } => TylError::network(message),
            TaskServiceError::Domain { message } => TylError::validation("domain", message),
            TaskServiceError::ExternalService { message } => TylError::network(message),
            TaskServiceError::InvalidInput { message } => TylError::validation("input", message),
        }
    }
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<TaskServiceConfig>,
    pub domain_service: Arc<dyn TaskService + Send + Sync>,
    pub event_service: Arc<EventService>,
    pub logger: Arc<dyn Logger + Send + Sync>,
}

/// Create the main application with all routes and middleware
pub async fn create_app(config: TaskServiceConfig) -> TaskServiceResult<Router> {
    // Initialize TYL framework components
    let logger = Arc::new(tyl_logging::loggers::console::ConsoleLogger::new());
    
    // Initialize event service
    let event_service = Arc::new(EventService::new().await.map_err(|e| {
        TaskServiceError::Configuration {
            message: format!("Failed to initialize event service: {}", e),
        }
    })?);
    
    // Initialize domain service with dependencies
    let domain_service = create_domain_service(&config).await?;
    
    // Create shared application state
    let state = AppState {
        config: Arc::new(config),
        domain_service,
        event_service,
        logger,
    };

    // Build the application with routes and middleware
    let app = Router::new()
        .merge(routes::health_routes())
        .merge(routes::api_routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()) // Configure based on your needs
        )
        .with_state(state);

    Ok(app)
}

/// Create domain service with all its dependencies
async fn create_domain_service(
    config: &TaskServiceConfig,
) -> TaskServiceResult<Arc<dyn TaskService + Send + Sync>> {
    // This is where you'd inject your specific domain service implementation
    // Example:
    // let db_adapter = create_database_adapter(&config.database).await?;
    // let external_client = create_http_client(&config.external_apis)?;
    // let service = Arc::new(UserServiceImpl::new(db_adapter, external_client));
    
    // For template purposes, return a mock implementation
    Ok(Arc::new(domain::MockTaskService::new()))
}

/// Start the microservice with graceful shutdown
pub async fn run_microservice(config: TaskServiceConfig) -> TaskServiceResult<()> {
    let app = create_app(config.clone()).await?;
    
    let listener = tokio::net::TcpListener::bind(&format!("{}:{}", config.api.host, config.api.port))
        .await
        .map_err(|e| TaskServiceError::Configuration {
            message: format!("Failed to bind to {}:{}: {}", config.api.host, config.api.port, e),
        })?;

    println!("🚀 Microservice started on {}:{}", config.api.host, config.api.port);
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| TaskServiceError::Api {
            message: format!("Server error: {}", e),
        })?;

    Ok(())
}

/// Handle graceful shutdown signals
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("🛑 Shutdown signal received, starting graceful shutdown");
}

/// Utility functions for microservice operations
pub mod utils {
    use uuid::Uuid;
    
    /// Generate a unique correlation ID for request tracking
    pub fn generate_correlation_id() -> String {
        Uuid::new_v4().to_string()
    }
    
    /// Generate a unique entity ID
    pub fn generate_entity_id() -> String {
        Uuid::new_v4().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_creation() {
        let config = TaskServiceConfig::default();
        let app = create_app(config).await;
        assert!(app.is_ok());
    }

    #[test]
    fn test_error_conversion() {
        let service_error = TaskServiceError::Domain {
            message: "Test error".to_string(),
        };
        
        let tyl_error: TylError = service_error.into();
        assert!(tyl_error.to_string().contains("Test error"));
    }

    #[test]
    fn test_correlation_id_generation() {
        let id1 = utils::generate_correlation_id();
        let id2 = utils::generate_correlation_id();
        
        assert_ne!(id1, id2);
        assert!(uuid::Uuid::parse_str(&id1).is_ok());
        assert!(uuid::Uuid::parse_str(&id2).is_ok());
    }
}