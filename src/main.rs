//! # TYL Task Service Microservice Entry Point
//!
//! This is the main entry point for the task service microservice.
//! It initializes the configuration, sets up logging and tracing, and starts the HTTP server.

use tyl_task_service::{TaskServiceConfig, run_microservice};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    // Initialize configuration from environment
    let config = TaskServiceConfig::from_env()
        .map_err(|e| format!("Failed to load configuration: {}", e))?;

    // Initialize logging and tracing
    init_observability(&config)?;

    // Print startup information
    println!("🚀 Starting {} microservice", config.service_name);
    println!("📝 Version: {}", config.version);
    println!("🌐 API endpoint: http://{}:{}", config.api.host, config.api.port);
    println!("📊 Health check: http://{}:{}/health", config.api.host, config.api.port);

    // Start the microservice
    run_microservice(config).await?;

    println!("👋 Microservice shutdown complete");
    Ok(())
}

/// Initialize logging and tracing for the microservice
fn init_observability(config: &TaskServiceConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    std::env::set_var("RUST_LOG", log_level);
    
    // Initialize tracing subscriber for distributed tracing
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| format!("Failed to set tracing subscriber: {}", e))?;

    tracing::info!(
        service_name = %config.service_name,
        version = %config.version,
        "Observability initialized"
    );

    Ok(())
}