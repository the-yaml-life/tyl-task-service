# TYL Microservice Template

🚀 **Template repository for creating microservices with the TYL framework**

This template provides a complete foundation for building production-ready microservices using hexagonal architecture and the TYL framework ecosystem.

## 🚀 Quick Start

### Option 1: Use the Generation Script (Recommended)

```bash
# Generate a new microservice automatically
./scripts/create-microservice.sh order-service OrderService Order

# With custom GitHub organization
./scripts/create-microservice.sh order-service OrderService Order your-org

# This creates tyl-order-service/ with all placeholders replaced
# AND creates a GitHub repository automatically
cd ../tyl-order-service
cargo run
```

**Prerequisites:**
- [GitHub CLI](https://cli.github.com/) installed and authenticated (`gh auth login`)
- Git configured with your GitHub credentials

### Option 2: Manual Template Usage

1. **Use This Template**
   Click "Use this template" button on GitHub or:
   ```bash
   gh repo create your-org/your-microservice --template the-yaml-life/tyl-microservice --public
   ```

2. **Replace Placeholders**
   Search and replace the following placeholders throughout the codebase:

   - `task-service` → your microservice name (e.g., `order-service`, `user-auth`)
   - `TaskService` → PascalCase service class (e.g., `OrderService`, `UserAuthService`)  
   - `Task` → PascalCase domain model (e.g., `Order`, `User`)
- `{DomainService}` → your domain service trait (e.g., `UserManager`, `OrderProcessor`)
- `{DomainModel}` → your domain model (e.g., `User`, `Order`)
- `{RequestType}` → API request type (e.g., `CreateUserRequest`, `ProcessOrderRequest`)
- `{ResponseType}` → API response type (e.g., `UserResponse`, `OrderResponse`)

### 3. Configure Environment
```bash
cp .env.example .env
# Edit .env with your configuration
```

### 4. Run Your Microservice
```bash
cargo run
```

## 📁 What's Included

### ✅ **Complete Microservice Structure**
```
tyl-microservice/
├── src/
│   ├── main.rs                # Application entry point
│   ├── lib.rs                 # Library exports
│   ├── config.rs              # Configuration management
│   ├── handlers/              # HTTP request handlers
│   ├── domain/                # Business logic
│   ├── adapters/              # External integrations
│   └── routes.rs              # API route definitions
├── tests/                     # Integration and API tests
├── config/                    # Environment configurations
├── docker/                    # Container definitions
├── .github/workflows/         # CI/CD pipelines
└── Cargo.toml                 # Dependencies and metadata
```

### ✅ **Production-Ready Features**
- 🏛️ **Hexagonal Architecture** - Clean separation of concerns
- 🌐 **HTTP API** - RESTful endpoints with Axum
- ⚡ **Async-First** - Built on Tokio for high performance
- 📊 **Observability** - Structured logging and tracing
- ⚙️ **Configuration** - Environment-based config management
- 🧪 **Testing** - Unit, integration, and API tests
- 🐳 **Docker Ready** - Container support included
- 🔒 **Security** - Built-in security best practices
- 📈 **Health Checks** - Monitoring endpoints

## 🏗️ Architecture

This microservice follows hexagonal architecture principles:

```rust
// Domain Layer - Business Logic
trait UserService {
    async fn create_user(&self, request: CreateUserRequest) -> Result<User, UserError>;
}

// Application Layer - Use Cases
struct UserApplicationService {
    user_repo: Arc<dyn UserRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

// Infrastructure Layer - Adapters
impl UserRepository for PostgresUserRepository {
    async fn save(&self, user: &User) -> Result<(), RepoError> {
        // Database implementation
    }
}

// API Layer - HTTP Handlers
async fn create_user_handler(
    State(service): State<Arc<dyn UserService>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    // HTTP handling
}
```

## 🧪 Testing Strategy

### **Test Pyramid**
- **Unit Tests** - Domain logic and individual components
- **Integration Tests** - Service interactions and database operations
- **API Tests** - HTTP endpoint behavior and contracts

### **Running Tests**
```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests
cargo test --test integration_tests

# API tests
cargo test --test api_tests

# With coverage
cargo tarpaulin --out html
```

## 🐳 Docker Support

### **Local Development**
```bash
docker-compose -f docker/docker-compose.yml up
```

### **Production Build**
```bash
docker build -t your-microservice -f docker/Dockerfile .
docker run -p 3000:3000 your-microservice
```

## 📝 Configuration

### **Environment Variables**
```bash
# Server
PORT=3000
HOST=0.0.0.0

# Database
DATABASE_URL=postgres://user:pass@localhost/db

# Logging
RUST_LOG=info
LOG_FORMAT=json

# TYL Framework
TYL_SERVICE_NAME=your-microservice
TYL_VERSION=1.0.0
```

### **Configuration Files**
- `config/development.toml` - Development settings
- `config/production.toml` - Production settings
- `.env` - Environment overrides

## 🛠️ Development Workflow

### **Daily Development**
```bash
# Start with auto-reload
cargo watch -x run

# Format code
cargo fmt

# Check linting
cargo clippy

# Run tests
cargo test
```

### **Pre-commit Checklist**
- [ ] Tests pass (`cargo test`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Linting clean (`cargo clippy`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated

## 📊 Monitoring & Observability

### **Health Endpoints**
- `GET /health` - Basic health check
- `GET /health/ready` - Readiness probe
- `GET /health/live` - Liveness probe
- `GET /metrics` - Prometheus metrics (if enabled)

### **Logging**
All logs are structured JSON with:
- Request correlation IDs
- Performance metrics
- Error context
- Business event tracking

### **Tracing**
Distributed tracing with OpenTelemetry:
- Request span tracking
- Database operation tracing
- External service call monitoring

## 🔒 Security Features

- **Input Validation** - Comprehensive request validation
- **Error Handling** - Secure error responses without information leakage
- **CORS Configuration** - Proper cross-origin settings
- **Rate Limiting** - Built-in request throttling (configurable)
- **Health Checks** - Secure monitoring endpoints

## 📦 TYL Framework Integration

This template leverages the complete TYL framework ecosystem:

- **tyl-errors** - Comprehensive error handling with retry logic
- **tyl-config** - Environment-based configuration management
- **tyl-logging** - Structured logging with multiple backends
- **tyl-tracing** - Distributed tracing and observability
- **tyl-db-core** - Database abstractions and connection management

## 🎯 Best Practices Included

1. **Domain-Driven Design** - Clear domain boundaries
2. **CQRS Pattern** - Separate read/write operations when beneficial
3. **Event-Driven Architecture** - Async communication support
4. **Graceful Shutdown** - Proper resource cleanup on termination
5. **Circuit Breaker** - Resilience patterns for external dependencies
6. **Retry Logic** - Configurable retry strategies
7. **Bulkhead Pattern** - Resource isolation
8. **API Versioning** - Support for multiple API versions

## 📝 Checklist After Using Template

- [ ] Replace all placeholder text
- [ ] Update `Cargo.toml` metadata and dependencies
- [ ] Implement your domain models and services
- [ ] Create API handlers for your endpoints
- [ ] Add database adapters if needed
- [ ] Configure environment variables
- [ ] Write comprehensive tests
- [ ] Update documentation
- [ ] Set up CI/CD pipelines
- [ ] Configure monitoring and alerting

## 🔗 Related Resources

- [TYL Framework Documentation](https://github.com/the-yaml-life)
- [Hexagonal Architecture Guide](https://alistair.cockburn.us/hexagonal-architecture/)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Tokio Documentation](https://tokio.rs/)

## 📄 License

AGPL-3.0 - See [LICENSE](LICENSE) for details.

---

**Ready to build production microservices with confidence!** 🚀