# TYL Task Service

🎯 **Production-ready task management microservice built with the TYL framework**

A comprehensive task management system following hexagonal architecture principles, providing robust task lifecycle management, dependency tracking, and real-time event-driven communication.

## 🚀 Quick Start

### Local Development

```bash
# Clone the repository
git clone https://github.com/the-yaml-life/tyl-task-service.git
cd tyl-task-service

# Configure environment
cp .env.example .env
# Edit .env with your database and service configuration

# Run with Docker Compose (recommended)
docker compose up -d

# Or run locally with Rust
cargo run
```

### Docker Deployment

```bash
# Using pre-built image
docker run -p 3000:3000 \
  -e FALKORDB_HOST=your-falkordb-host \
  -e REDIS_PUBSUB_HOST=your-redis-host \
  ghcr.io/the-yaml-life/tyl-task-service:latest

# Build from source
docker build -t tyl-task-service .
docker run -p 3000:3000 tyl-task-service
```

### Validation

```bash
# Validate Docker setup end-to-end
./scripts/validate-docker.sh

# Test API endpoints
curl http://localhost:3000/health
curl http://localhost:3000/api/v1/tasks
```

## 🎯 Features

### ✅ **Task Management**
- **Task Lifecycle** - Create, update, complete, and archive tasks
- **Context & Priority** - Organize tasks by context (work, personal, learning) and priority levels
- **Complexity Tracking** - Simple, moderate, complex task categorization
- **Dependency Management** - Task dependencies with circular dependency detection
- **Status Transitions** - Controlled task status workflows
- **Assignment System** - User assignment and tracking

### ✅ **Technical Features**
- 🏛️ **Hexagonal Architecture** - Clean separation of concerns
- 🌐 **RESTful API** - Comprehensive HTTP endpoints with Axum
- 📊 **Graph Database** - FalkorDB for complex relationship modeling
- 🔄 **Event-Driven** - Real-time task events via Redis Pub/Sub
- 📈 **Analytics** - Task insights, completion metrics, and reporting
- ⚡ **Async Performance** - Built on Tokio for high concurrency
- 📋 **Health Monitoring** - Comprehensive health checks and metrics
- 🧪 **Test Coverage** - 61+ unit tests with integration testing

### ✅ **Production Ready**
- 🐳 **Docker & Compose** - Complete containerization
- 🔄 **CI/CD Pipelines** - GitHub Actions with automated releases
- 🔒 **Security** - Input validation and secure error handling
- 📊 **Observability** - Structured logging with TYL tracing
- ⚙️ **Configuration** - Environment-based config management
- 🌐 **Multi-platform** - Linux AMD64/ARM64 support

## 🏗️ Architecture

TYL Task Service follows hexagonal architecture with clear domain boundaries:

```rust
// Domain Layer - Core Business Logic
trait TaskService {
    async fn create_task(&self, request: CreateTaskRequest) -> TylResult<Task>;
    async fn update_task_status(&self, id: &str, status: TaskStatus) -> TylResult<Task>;
    async fn add_dependency(&self, task_id: &str, depends_on: &str) -> TylResult<()>;
}

// Application Layer - Use Cases
struct TaskApplicationService {
    task_repo: Arc<dyn TaskRepository>,
    event_publisher: Arc<dyn EventPublisher>,
    dependency_validator: Arc<dyn DependencyValidator>,
}

// Infrastructure Layer - Adapters
struct GraphTaskRepository {
    falkordb: Arc<FalkorDBClient>,
}

impl TaskRepository for GraphTaskRepository {
    async fn save(&self, task: &Task) -> TylResult<Task> {
        // FalkorDB Cypher implementation
    }
    
    async fn find_dependencies(&self, task_id: &str) -> TylResult<Vec<Task>> {
        // Graph traversal for dependencies
    }
}

// API Layer - HTTP Handlers
async fn create_task_handler(
    State(service): State<Arc<dyn TaskService>>,
    Json(request): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, ApiError> {
    let task = service.create_task(request).await?;
    Ok(Json(TaskResponse::from(task)))
}
```

### Tech Stack
- **Database**: FalkorDB (Redis + Graph Database)
- **Pub/Sub**: Redis for event-driven communication
- **Web Framework**: Axum with Tower middleware
- **Testing**: 61+ unit tests, integration tests, Docker validation
- **Observability**: TYL logging and tracing framework

## 🧪 Testing

### **Test Coverage** (61+ tests)
- **Unit Tests** - Domain logic, business rules, and validation
- **Integration Tests** - Database operations and service interactions
- **API Tests** - HTTP endpoint contracts and error handling
- **Docker Tests** - End-to-end validation with real services

### **Running Tests**
```bash
# All tests (61+ passing)
cargo test

# Unit tests only
cargo test --lib

# Integration tests with real FalkorDB
cargo test --test integration_tests

# Docker end-to-end validation
./scripts/validate-docker.sh

# Test with coverage
cargo tarpaulin --out html
```

### **Test Environment**
```bash
# Start test dependencies
docker compose -f docker-compose.test.yml up -d

# Run integration tests
export FALKORDB_HOST=localhost
export REDIS_PUBSUB_HOST=localhost
cargo test --test integration_tests
```

## 🐳 Docker & Deployment

### **Local Development**
```bash
# Start complete stack
docker compose up -d

# Check service health
curl http://localhost:3000/health/detail

# View logs
docker compose logs -f tyl-task-service
```

### **Production Images**
```bash
# Multi-platform images available
docker pull ghcr.io/the-yaml-life/tyl-task-service:latest

# Or specific version
docker pull ghcr.io/the-yaml-life/tyl-task-service:v1.0.0
```

### **Deployment Package**
Each release includes a complete Docker deployment package:
- Multi-platform Docker images (AMD64/ARM64)
- Docker Compose configurations
- Environment templates
- Installation scripts
- Health check utilities

## ⚙️ Configuration

### **Environment Variables**
```bash
# Server Configuration
PORT=3000
HOST=0.0.0.0

# FalkorDB (Graph Database)
FALKORDB_HOST=localhost
FALKORDB_PORT=6379
FALKORDB_PASSWORD=
FALKORDB_DATABASE=tasks

# Redis Pub/Sub
REDIS_PUBSUB_HOST=localhost
REDIS_PUBSUB_PORT=6380
REDIS_PUBSUB_PASSWORD=

# Logging & Tracing
RUST_LOG=info
TYL_LOG_LEVEL=info
TYL_TRACE_ENDPOINT=http://localhost:14268/api/traces

# TYL Framework
TYL_SERVICE_NAME=tyl-task-service
TYL_SERVICE_VERSION=1.0.0
TYL_ENVIRONMENT=development
```

### **Configuration Files**
- `config/development.toml` - Development settings
- `config/production.toml` - Production settings
- `config/test.toml` - Test environment settings
- `.env.example` - Environment template

## 🛠️ API Reference

### **Health Endpoints**
```bash
# Basic health check
GET /health

# Detailed health with dependencies
GET /health/detail
```

### **Task Management**
```bash
# Create a task
POST /api/v1/tasks
{
  "name": "Complete project documentation",
  "description": "Write comprehensive API docs",
  "context": "work",
  "priority": "high",
  "complexity": "moderate"
}

# Get task by ID
GET /api/v1/tasks/{id}

# List tasks with filtering
GET /api/v1/tasks?status=pending&context=work&priority=high

# Update task status
PATCH /api/v1/tasks/{id}/status
{
  "status": "in_progress"
}

# Add task dependency
POST /api/v1/tasks/{id}/dependencies
{
  "depends_on": "other-task-id"
}

# Get task analytics
GET /api/v1/tasks/analytics
```

### **Development Commands**
```bash
# Auto-reload development
cargo watch -x run

# Code quality
cargo fmt && cargo clippy

# Run all tests
cargo test

# Integration test with services
docker compose up -d && cargo test --test integration_tests
```

## 📊 Monitoring & Events

### **Health Monitoring**
```json
// GET /health/detail
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime": "2h 30m 45s",
  "dependencies": {
    "falkordb": "healthy",
    "redis_pubsub": "healthy"
  },
  "metrics": {
    "total_tasks": 1337,
    "active_tasks": 42,
    "completed_today": 18
  }
}
```

### **Event System**
Real-time task events published via Redis:
- `task.created` - New task creation
- `task.status_changed` - Status transitions
- `task.assigned` - User assignment
- `task.dependency_added` - Dependency changes
- `task.completed` - Task completion

### **Observability**
- **Structured Logging** - JSON logs with correlation IDs
- **Distributed Tracing** - TYL tracing with span tracking
- **Performance Metrics** - Request timing and database operations
- **Business Metrics** - Task completion rates and analytics

## 🔒 Security & Performance

### **Security Features**
- **Input Validation** - Comprehensive request validation with Serde
- **Error Sanitization** - Secure error responses without information leakage
- **Dependency Validation** - Circular dependency detection
- **Status Transition Control** - Validated task status workflows
- **Database Security** - Parameterized Cypher queries

### **Performance**
- **Async Architecture** - Tokio-based for high concurrency
- **Graph Database** - Efficient relationship queries with FalkorDB
- **Connection Pooling** - Optimized database connections
- **Event-Driven** - Non-blocking real-time updates
- **Multi-platform** - Optimized Docker images for AMD64/ARM64

## 📦 TYL Framework Integration

Built with the complete TYL framework ecosystem:

- **tyl-errors** - Comprehensive error handling with retry logic
- **tyl-config** - Environment-based configuration management  
- **tyl-logging** - Structured logging with multiple backends
- **tyl-tracing** - Distributed tracing and observability
- **tyl-pubsub-port** - Event-driven pub/sub architecture
- **tyl-falkordb-adapter** - Graph database integration

## 🚀 CI/CD & Releases

### **Automated Pipelines**
- **Continuous Integration** - Automated testing on every push
- **Security Scanning** - Vulnerability assessment with Trivy
- **Multi-platform Builds** - Docker images for AMD64/ARM64
- **Semantic Versioning** - Automated version management
- **Release Artifacts** - Binary releases and deployment packages

### **Release Assets**
Each release includes:
- Multi-platform Docker images
- Binary releases for Linux (GNU/musl/ARM64)
- Complete Docker deployment package
- Installation scripts and systemd services
- Security scanning reports (SBOM)
- Comprehensive documentation

### **Quick Installation**
```bash
# Download and install latest release
wget https://github.com/the-yaml-life/tyl-task-service/releases/latest/download/tyl-task-service-x86_64-unknown-linux-gnu.tar.gz
tar -xzf tyl-task-service-*.tar.gz
cd tyl-task-service-*/
sudo ./install.sh
```

## 🎯 Domain Model

### **Task Entity**
```rust
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub context: TaskContext,
    pub priority: TaskPriority,
    pub complexity: TaskComplexity,
    pub assigned_user_id: Option<String>,
    pub dependencies: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
```

### **Enumerations**
- **TaskStatus**: `pending`, `in_progress`, `completed`, `cancelled`, `on_hold`
- **TaskContext**: `work`, `personal`, `learning`, `health`, `finance`
- **TaskPriority**: `low`, `medium`, `high`, `urgent`
- **TaskComplexity**: `simple`, `moderate`, `complex`

### **Business Rules**
- Tasks can have multiple dependencies
- Circular dependencies are automatically detected and prevented
- Status transitions follow defined workflows
- Completion tracking with automatic timestamps
- User assignment and responsibility tracking

## 🔗 Related Resources

- [TYL Framework](https://github.com/the-yaml-life) - Complete microservice framework
- [FalkorDB](https://www.falkordb.com/) - Graph database built on Redis
- [Axum](https://docs.rs/axum/latest/axum/) - Modern async web framework
- [API Documentation](./docs/api.md) - Detailed endpoint reference

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup
```bash
# Setup development environment
git clone https://github.com/the-yaml-life/tyl-task-service.git
cd tyl-task-service
docker compose up -d  # Start dependencies
cargo test            # Run tests
cargo run            # Start service
```

## 📄 License

AGPL-3.0 - See [LICENSE](LICENSE) for details.

---

**Production-ready task management with the TYL framework!** 🎯🚀