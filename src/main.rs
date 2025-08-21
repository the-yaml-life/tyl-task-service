//! # TYL Task Service Microservice Entry Point
//!
//! This is the main entry point for the task service microservice.
//! It initializes the configuration, sets up logging and tracing, and starts the HTTP server.

use tyl_task_service::{TaskServiceConfig, run_microservice, LogLevel, LogRecord, ConsoleLogger, JsonLogger, Logger};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    // Initialize configuration from environment
    let config = TaskServiceConfig::from_env()
        .map_err(|e| format!("Failed to load configuration: {}", e))?;

    // Initialize TYL logging early
    let logger: Box<dyn Logger> = match config.monitoring.log_format.as_str() {
        "json" => Box::new(JsonLogger::new()),
        _ => Box::new(ConsoleLogger::new()),
    };

    // Log startup information using TYL logging
    logger.log(&LogRecord::new(LogLevel::Info, &format!("🚀 Starting {} microservice", config.service_name)));
    logger.log(&LogRecord::new(LogLevel::Info, &format!("📝 Version: {}", config.version)));
    logger.log(&LogRecord::new(LogLevel::Info, &format!("🌐 API endpoint: http://{}:{}", config.api.host, config.api.port)));
    logger.log(&LogRecord::new(LogLevel::Info, &format!("📊 Health check: http://{}:{}/health", config.api.host, config.api.port)));
    logger.log(&LogRecord::new(LogLevel::Info, &format!("🗄️ Database: {}", config.database_connection_info())));
    logger.log(&LogRecord::new(LogLevel::Debug, &format!("🔧 Log level: {} | Format: {}", config.monitoring.log_level, config.monitoring.log_format)));

    // Start the microservice
    run_microservice(config).await?;

    logger.log(&LogRecord::new(LogLevel::Info, "👋 Microservice shutdown complete"));
    Ok(())
}

