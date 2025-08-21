#!/bin/bash

# Script to validate Docker setup end-to-end
set -e

echo "🐳 Validating TYL Task Service Docker Setup"
echo "==========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Validate prerequisites
echo "1. Checking prerequisites..."
if ! command_exists docker; then
    print_error "Docker is not installed"
    exit 1
fi

if ! docker compose version >/dev/null 2>&1; then
    print_error "Docker Compose is not installed"
    exit 1
fi

print_success "Docker and Docker Compose are available"

# Check if ports are available
echo "2. Checking if required ports are available..."
for port in 3000 6379 6380; do
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        print_error "Port $port is already in use"
        exit 1
    fi
done
print_success "Required ports (3000, 6379, 6380) are available"

# Build the Docker image
echo "3. Building Docker image..."
if docker build -t tyl-task-service . > /tmp/docker-build.log 2>&1; then
    print_success "Docker image built successfully"
else
    print_error "Docker build failed"
    echo "Build log:"
    cat /tmp/docker-build.log
    exit 1
fi

# Start services with Docker Compose
echo "4. Starting services with Docker Compose..."
docker compose down --remove-orphans 2>/dev/null || true
if docker compose up -d --build > /tmp/docker-compose.log 2>&1; then
    print_success "Services started successfully"
else
    print_error "Failed to start services"
    echo "Docker Compose log:"
    cat /tmp/docker-compose.log
    exit 1
fi

# Wait for services to be healthy
echo "5. Waiting for services to be healthy..."
max_attempts=30
attempt=0

while [ $attempt -lt $max_attempts ]; do
    attempt=$((attempt + 1))
    
    if docker compose ps | grep -q "healthy"; then
        print_info "Attempt $attempt/$max_attempts: Some services are healthy"
    else
        print_info "Attempt $attempt/$max_attempts: Waiting for services to be healthy..."
    fi
    
    # Check if all services are healthy
    unhealthy_services=$(docker compose ps --filter "health=starting" --filter "health=unhealthy" -q | wc -l)
    
    if [ "$unhealthy_services" -eq 0 ]; then
        print_success "All services are healthy"
        break
    fi
    
    if [ $attempt -eq $max_attempts ]; then
        print_error "Services did not become healthy within expected time"
        echo "Service status:"
        docker compose ps
        echo "Logs from tyl-task-service:"
        docker compose logs tyl-task-service
        exit 1
    fi
    
    sleep 5
done

# Test API endpoints
echo "6. Testing API endpoints..."

# Test health endpoint
if curl -f -s http://localhost:3000/health > /dev/null; then
    print_success "Health endpoint is responding"
else
    print_error "Health endpoint is not responding"
    echo "Service logs:"
    docker compose logs tyl-task-service
    exit 1
fi

# Test health detail endpoint
if curl -f -s http://localhost:3000/health/detail > /dev/null; then
    print_success "Health detail endpoint is responding"
else
    print_error "Health detail endpoint is not responding"
    exit 1
fi

# Test task creation
echo "7. Testing task creation..."
task_response=$(curl -s -X POST http://localhost:3000/api/v1/tasks \
    -H "Content-Type: application/json" \
    -d '{
        "name": "Docker Test Task",
        "description": "Testing task creation via Docker",
        "context": "work",
        "priority": "medium",
        "complexity": "simple"
    }')

if echo "$task_response" | grep -q '"id"'; then
    print_success "Task creation endpoint is working"
    
    # Extract task ID
    task_id=$(echo "$task_response" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
    print_info "Created task with ID: $task_id"
    
    # Test task retrieval
    if curl -f -s "http://localhost:3000/api/v1/tasks/$task_id" > /dev/null; then
        print_success "Task retrieval endpoint is working"
    else
        print_error "Task retrieval endpoint failed"
        exit 1
    fi
    
else
    print_error "Task creation failed"
    echo "Response: $task_response"
    exit 1
fi

# Test list tasks
echo "8. Testing task listing..."
if curl -f -s http://localhost:3000/api/v1/tasks > /dev/null; then
    print_success "Task listing endpoint is working"
else
    print_error "Task listing endpoint failed"
    exit 1
fi

print_success "All Docker validation tests passed!"
print_info "You can now use the service at http://localhost:3000"
print_info "API documentation is available in the README.md"
print_info "To stop the services, run: docker compose down"

echo ""
echo "🎉 Docker setup validation completed successfully!"