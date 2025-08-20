# 🏗️ TYL Microservice Template

Este template proporciona una base completa y lista para producción para crear microservicios con el framework TYL, implementando arquitectura hexagonal y patrones event-driven.

## 🚀 Quick Start

### Generar un nuevo microservicio

```bash
# Syntax: ./scripts/create-microservice.sh <service-name> <ServiceClass> <DomainModel> [github-org] [output-dir]

# Ejemplo: Crear un servicio de órdenes (usa the-yaml-life por defecto)
./scripts/create-microservice.sh order-service OrderService Order

# Ejemplo: Crear un servicio de autenticación en tu organización
./scripts/create-microservice.sh user-auth UserAuthService User my-org

# Ejemplo: Crear un servicio de pagos con directorio custom
./scripts/create-microservice.sh payment-gateway PaymentService Payment my-org ~/my-services
```

**Prerequisites:**
- [GitHub CLI](https://cli.github.com/) instalado y autenticado
- Git configurado correctamente
- Permisos para crear repos en la organización especificada

### Después de la generación

```bash
cd ../tyl-order-service  # (o el directorio generado)

# Compilar y ejecutar
cargo run

# Ejecutar tests
cargo test

# Usar Docker
docker-compose up
```

## 📋 Parámetros del Template

| Parámetro | Descripción | Ejemplo | Transformaciones |
|-----------|-------------|---------|------------------|
| **service-name** | Nombre kebab-case del servicio | `order-service` | → `tyl-order-service` (package name) |
| **ServiceClass** | Nombre PascalCase de la clase principal | `OrderService` | → `OrderServiceConfig`, `OrderServiceError` |
| **DomainModel** | Nombre PascalCase del modelo de dominio | `Order` | → `CreateOrderRequest`, `OrderResponse` |

### Variables generadas automáticamente:

- **task-service** → `order-service`
- **task_service** → `order_service`
- **TASK_SERVICE** → `ORDER_SERVICE`
- **TaskService** → `OrderService`
- **TaskService** → `OrderService`
- **Task** → `Order`
- **task** → `order`
- **task** → `order`

## 🏗️ Arquitectura del Template

### Estructura de directorios generada:

```
tyl-{service-name}/
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library exports y setup
│   ├── config.rs               # Configuration management
│   ├── domain.rs               # Business logic y domain models
│   ├── routes.rs               # HTTP route definitions
│   ├── handlers/               # HTTP request handlers
│   │   ├── mod.rs
│   │   ├── api.rs              # Business logic endpoints
│   │   └── health.rs           # Health check endpoints
│   ├── adapters/               # External integrations
│   │   ├── mod.rs
│   │   ├── database.rs         # Database abstraction
│   │   └── http_client.rs      # External HTTP services
│   └── events/                 # Event-driven architecture
│       ├── mod.rs              # Event system exports
│       ├── service.rs          # Event publishing/subscription
│       ├── handlers.rs         # Event handler traits
│       └── examples.rs         # Domain event examples
├── tests/
│   ├── api_tests.rs            # API endpoint tests
│   └── integration_tests.rs    # Integration tests
├── examples/
│   ├── basic_usage.rs          # Basic usage example
│   └── event_driven_usage.rs   # Event-driven examples
├── config/
│   ├── development.toml        # Development config
│   └── production.toml         # Production config
├── docker/
│   ├── Dockerfile              # Container definition
│   ├── docker-compose.yml      # Local development
│   └── init.sql                # Database initialization
├── .env.example                # Environment variables template
├── Cargo.toml                  # Package configuration
├── README.md                   # Project documentation
├── CLAUDE.md                   # Implementation details
└── TEMPLATE.md                 # Este archivo
```

### Arquitectura Hexagonal

El template implementa **arquitectura hexagonal** con estas capas:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HTTP API      │    │   Event Bus     │    │   Database      │
│   (Adapters)    │    │   (Adapters)    │    │   (Adapters)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   Domain Core   │
                    │  ({{ServiceClass}}) │
                    │ ({{DomainModel}})  │
                    └─────────────────┘
```

## 🎯 Patrones Implementados

### 1. **Domain-Driven Design**
```rust
// Dominio rico con business logic
pub trait TaskService {
    async fn process(&self, request: CreateTaskRequest) -> TaskServiceResult<TaskResponse>;
    async fn get_by_id(&self, id: &str) -> TaskServiceResult<Option<Task>>;
    async fn create(&self, data: CreateTaskRequest) -> TaskServiceResult<Task>;
    async fn update(&self, id: &str, data: UpdateTaskRequest) -> TaskServiceResult<Task>;
    async fn delete(&self, id: &str) -> TaskServiceResult<()>;
}
```

### 2. **Repository Pattern**
```rust
// Abstracción de persistencia
pub trait TaskRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> TaskServiceResult<Option<Task>>;
    async fn save(&self, task: &Task) -> TaskServiceResult<()>;
    async fn update(&self, task: &Task) -> TaskServiceResult<()>;
    async fn delete(&self, id: &str) -> TaskServiceResult<()>;
}
```

### 3. **Event-Driven Architecture**
```rust
// Publishing events
let event = TaskCreated {
    task_id: task.id,
    created_at: Utc::now(),
};

event_service.publish("task.created", event).await?;

// Handling events
#[async_trait]
impl DomainEventHandler<TaskCreated> for TaskCreatedHandler {
    async fn handle_domain_event(&self, event: TaskCreated) -> HandlerResult {
        // Handle the event
        Ok(())
    }
}
```

### 4. **Configuration Management**
```rust
// Configuración tipada y validada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskServiceConfig {
    pub service_name: String,
    pub version: String,
    pub api: ApiConfig,
    pub database: Option<DatabaseConfig>,
    pub external: ExternalConfig,
    pub monitoring: MonitoringConfig,
}
```

## 🧪 Testing Strategy

El template incluye testing completo:

### Unit Tests
- Tests de cada módulo individual
- Mocks para dependencies externas
- Coverage de business logic

### Integration Tests
- Tests end-to-end de APIs
- Tests de integración entre componentes
- Tests con datos realistas

### API Tests
- Tests específicos de endpoints HTTP
- Validación de request/response
- Tests de error handling

### Event Tests
- Tests de publishing/subscribing
- Tests de event handlers
- Tests de integration entre eventos

## 🔧 Customización Post-Generación

### 1. **Implementar Business Logic**

Edita `src/domain.rs`:
```rust
impl TaskService for TaskServiceImpl {
    async fn create(&self, data: CreateTaskRequest) -> TaskServiceResult<Task> {
        // 1. Validar datos
        self.validate_task(&data)?;
        
        // 2. Crear entidad
        let task = Task::new(data);
        
        // 3. Persistir
        self.repository.save(&task).await?;
        
        // 4. Publicar evento
        let event = TaskCreated::from(&task);
        self.event_service.publish("task.created", event).await?;
        
        Ok(task)
    }
}
```

### 2. **Configurar Database**

Edita `src/adapters/database.rs`:
```rust
impl TaskRepository for PostgresTaskRepository {
    async fn save(&self, task: &Task) -> TaskServiceResult<()> {
        sqlx::query!(
            "INSERT INTO tasks (id, name, status, created_at) VALUES ($1, $2, $3, $4)",
            task.id,
            task.name,
            task.status as _,
            task.created_at
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

### 3. **Definir Domain Events**

Edita `src/events/examples.rs`:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskCreated {
    pub task_id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskUpdated {
    pub task_id: String,
    pub changes: HashMap<String, serde_json::Value>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

### 4. **Configurar Production Environment**

Edita `config/production.toml`:
```toml
service_name = "task-service"
version = "1.0.0"

[api]
host = "0.0.0.0"
port = 8080

[database]
url = "${DATABASE_URL}"
max_connections = 20

[monitoring]
enable_tracing = true
enable_metrics = true
```

## 🐳 Deployment con Docker

### Development
```bash
# Start all services
docker-compose up

# Access service
curl http://localhost:3000/health
```

### Production
```bash
# Build image
docker build -t tyl-task-service -f docker/Dockerfile .

# Run container
docker run -p 8080:8080 \
  -e DATABASE_URL=postgres://... \
  -e RUST_LOG=info \
  tyl-task-service
```

## 📚 Recursos Adicionales

### TYL Framework Components
- **tyl-errors**: Manejo comprehensivo de errores
- **tyl-config**: Gestión de configuración
- **tyl-logging**: Logging estructurado
- **tyl-tracing**: Distributed tracing
- **tyl-pubsub-port**: Event-driven architecture

### External Dependencies
- **Axum**: Web framework moderno
- **Tokio**: Async runtime
- **Serde**: Serialization
- **Sqlx**: Database toolkit (opcional)
- **Tracing**: Observability

## 🔄 Evolution Path

### Phase 1: Basic Implementation
1. Implementar domain logic básico
2. Conectar database
3. Agregar tests unitarios

### Phase 2: Advanced Features
1. Implementar event-driven workflows
2. Agregar métricas y monitoring
3. Performance tuning

### Phase 3: Production Ready
1. Security hardening
2. Load testing
3. CI/CD pipeline
4. Documentation completa

## 🤝 Best Practices Incluidas

✅ **Arquitectura**: Hexagonal + Event-driven  
✅ **Error Handling**: TylError integration  
✅ **Testing**: Unit + Integration + API tests  
✅ **Configuration**: Environment-based config  
✅ **Logging**: Structured logging + tracing  
✅ **Documentation**: Code + API + architecture  
✅ **DevEx**: Docker + examples + scripts  
✅ **Security**: Input validation + error sanitization  
✅ **Performance**: Async-first + connection pooling  
✅ **Monitoring**: Health checks + metrics ready  

Este template proporciona una base sólida y probada para crear microservicios de production-grade con el framework TYL. 🚀