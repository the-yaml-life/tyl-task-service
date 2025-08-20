# CLAUDE.md - tyl-task-service

## 📋 **Microservice Context**

**tyl-task-service** is a Task management microservice built with the TYL framework.

## 🏗️ **Architecture**

This microservice follows hexagonal architecture with clear separation between:

### **Application Core (Domain)**
```rust
// Business logic traits
trait TaskService {
    async fn process(&self, request: CreateTaskRequest) -> TaskServiceResult<TaskResponse>;
}

// Domain models
struct Task {
    id: String,
    // Add domain fields specific to Task
}
```

### **Adapters**
- **HTTP API** - REST endpoints using Axum
- **Event System** - Event-driven communication using tyl-pubsub-port
- **Database** - Data persistence layer
- **External Services** - HTTP clients for external APIs

### **Core Types**
- `TaskServiceConfig` - Microservice configuration
- `TaskServiceError` - Error types with thiserror
- `TaskServiceResult<T>` - Result type alias
- `EventService` - Event publishing and subscription service
- `DomainEventHandler<T>` - Type-safe event handler trait
- `CreateTaskRequest` - API request models
- `TaskResponse` - API response models

## 🧪 **Testing**

```bash
# Unit tests
cargo test -p tyl-task-service

# Integration tests
cargo test --test integration_tests -p tyl-task-service

# Run microservice locally
cargo run -p tyl-task-service

# API testing
cargo test --test api_tests -p tyl-task-service
```

## 📂 **File Structure**

```
tyl-task-service/
├── src/
│   ├── main.rs                # Application entry point
│   ├── lib.rs                 # Library exports
│   ├── config.rs              # Configuration management
│   ├── handlers/              # HTTP request handlers
│   │   ├── mod.rs
│   │   └── health.rs          # Health check endpoints
│   ├── domain/                # Business logic
│   │   ├── mod.rs
│   │   └── service.rs         # Domain services
│   ├── adapters/              # External integrations
│   │   ├── mod.rs
│   │   ├── database.rs        # Database adapter
│   │   └── http_client.rs     # External HTTP services
│   ├── events/                # Event-driven architecture
│   │   ├── mod.rs             # Event module exports
│   │   ├── service.rs         # Event service implementation
│   │   ├── handlers.rs        # Event handler traits and adapters
│   │   └── examples.rs        # Example events and handlers
│   └── routes.rs              # API route definitions
├── tests/
│   ├── integration_tests.rs   # Integration tests
│   └── api_tests.rs           # API endpoint tests
├── config/
│   ├── development.toml       # Development configuration
│   └── production.toml        # Production configuration
├── docker/
│   ├── Dockerfile             # Container definition
│   └── docker-compose.yml     # Local development setup
├── .env.example               # Environment variables template
├── README.md                  # Main documentation
├── CLAUDE.md                  # This file
└── Cargo.toml                 # Package metadata
```

## 🔧 **How to Use**

### **Quick Start**
```bash
# 1. Clone from template
gh repo create your-org/your-microservice --template the-yaml-life/tyl-microservice

# 2. Replace placeholders
# Search and replace: task-service, TaskService, Task, etc.

# 3. Configure environment
cp .env.example .env
# Edit .env with your configuration

# 4. Run locally
cargo run
```

### **API Usage**
```bash
# Health check
curl http://localhost:3000/health

# Your API endpoints
curl -X POST http://localhost:3000/api/v1/{{ENDPOINT}} \
  -H "Content-Type: application/json" \
  -d '{"key": "value"}'
```

### **Custom Domain Implementation**
```rust
use tyl_task-service::{TaskService, CreateTaskRequest, TaskResponse, TaskServiceResult};

struct MyTaskService {
    // Custom fields
}

#[async_trait::async_trait]
impl TaskService for MyTaskService {
    async fn process(&self, request: CreateTaskRequest) -> TaskServiceResult<TaskResponse> {
        // Custom business logic
        Ok(TaskResponse { /* ... */ })
    }
}
```

### **Event-Driven Architecture**

This microservice includes built-in support for event-driven architecture using `tyl-pubsub-port`.

#### **Publishing Events**
```rust
use tyl_task-service::{EventService, AppState};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct TaskCreated {
    id: String,
    // Add domain-specific fields
}

// In your HTTP handler
async fn create_Task_handler(State(state): State<AppState>) -> Result<Json<Response>, ApiError> {
    // 1. Create Task in database
    let Task = create_Task_in_db().await?;
    
    // 2. Publish domain event
    let event = TaskCreated {
        id: Task.id,
        // Add other relevant fields
    };
    
    state.event_service.publish("Task.created", event).await?;
    
    Ok(Json(response))
}
```

#### **Handling Events**
```rust
use tyl_task-service::{DomainEventHandler, domain_handler};
use tyl_pubsub_port::HandlerResult;

struct TaskCreatedHandler;

#[async_trait::async_trait]
impl DomainEventHandler<TaskCreated> for TaskCreatedHandler {
    async fn handle_domain_event(&self, event: TaskCreated) -> HandlerResult {
        // Process the event (perform post-creation tasks, etc.)
        println!("Task created: {}", event.id);
        Ok(())
    }
}

// Set up during application startup
async fn setup_events(event_service: &EventService) -> Result<(), Error> {
    let handler = domain_handler!(TaskCreatedHandler);
    event_service.subscribe("Task.created", handler).await?;
    Ok(())
}
```

#### **Built-in Event Examples**
The template includes complete examples in `src/events/examples.rs`:
- `TaskCreated` - Task creation events
- `TaskUpdated` - Task update events  
- `SystemNotification` - System notifications with levels

#### **Event Testing**
```bash
# Run the event-driven example
cargo run --example event_driven_usage

# Test event handlers
cargo test events::

# Test with specific event types
cargo test test_Task_created_handler
```

## 🛠️ **Useful Commands**

```bash
# Development
cargo run                                    # Start microservice
cargo watch -x run                          # Auto-restart on changes
cargo clippy -p tyl-task-service     # Linting
cargo fmt -p tyl-task-service        # Formatting
cargo test -p tyl-task-service       # Run tests

# Docker
docker build -t tyl-task-service -f docker/Dockerfile .
docker-compose -f docker/docker-compose.yml up

# Documentation
cargo doc --no-deps -p tyl-task-service --open

# Release
cargo build --release
```

## 📦 **Dependencies**

### **TYL Framework**
- `tyl-errors` - Comprehensive error handling
- `tyl-config` - Configuration management  
- `tyl-logging` - Structured logging
- `tyl-tracing` - Distributed tracing
- `tyl-pubsub-port` - Event-driven pub/sub architecture

### **Microservice Runtime**
- `tokio` - Async runtime
- `axum` - Web framework
- `tower` / `tower-http` - Middleware
- `serde` / `serde_json` - Serialization
- `reqwest` - HTTP client
- `config` - Configuration loading
- `dotenvy` - Environment variables

### **Development**
- `tokio-test` - Async testing utilities
- `axum-test` - HTTP testing framework

## 🎯 **Design Principles**

1. **Hexagonal Architecture** - Clean separation between domain, adapters, and infrastructure
2. **Event-Driven Design** - Built-in support for publish/subscribe patterns with tyl-pubsub-port
3. **Single Responsibility** - Each microservice handles one bounded context
4. **Async-First** - Built on tokio for high-performance concurrent operations
5. **Configuration-Driven** - Environment-based configuration with sane defaults
6. **Observability** - Built-in logging, tracing, and health checks
7. **API-First** - RESTful HTTP APIs with proper error handling
8. **Testability** - Comprehensive unit, integration, and API tests
9. **Container-Ready** - Docker support for easy deployment

## ⚠️ **Known Limitations**

- Database adapters need to be implemented per use case
- Event system uses mock adapter by default (replace with Redis/Kafka for production)
- Metrics collection needs external monitoring setup
- Service discovery not included (use external tools)

## 📝 **Notes for Contributors**

- Follow TDD approach with async test patterns
- Maintain hexagonal architecture boundaries
- Document all public APIs with examples and async considerations
- Add integration tests for HTTP endpoints
- Use TYL framework modules instead of duplicating functionality
- Keep external dependencies minimal and well-justified
- Ensure all handlers are properly traced and logged

## 🔗 **Related TYL Modules**

- [`tyl-errors`](https://github.com/the-yaml-life/tyl-errors) - Error handling
- [`tyl-config`](https://github.com/the-yaml-life/tyl-config) - Configuration management
- [`tyl-logging`](https://github.com/the-yaml-life/tyl-logging) - Structured logging
- [`tyl-tracing`](https://github.com/the-yaml-life/tyl-tracing) - Distributed tracing
- [`tyl-db-core`](https://github.com/the-yaml-life/tyl-db-core) - Database abstractions