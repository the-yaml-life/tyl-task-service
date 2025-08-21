#!/bin/bash

# Test script for tyl-task-service
# This script handles running integration tests with Docker containers

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if docker-compose is available
check_docker_compose() {
    if command -v docker-compose &> /dev/null; then
        DOCKER_COMPOSE_CMD="docker-compose"
    elif command -v docker &> /dev/null && docker compose version &> /dev/null; then
        DOCKER_COMPOSE_CMD="docker compose"
    else
        print_error "Neither 'docker-compose' nor 'docker compose' is available"
        exit 1
    fi
}

# Function to wait for service to be healthy
wait_for_service() {
    local service_name=$1
    local max_attempts=$2
    local attempt=0
    
    print_status "Waiting for $service_name to be healthy..."
    
    while [ $attempt -lt $max_attempts ]; do
        if $DOCKER_COMPOSE_CMD ps $service_name | grep -q "healthy"; then
            print_success "$service_name is healthy"
            return 0
        fi
        
        attempt=$((attempt + 1))
        print_status "Attempt $attempt/$max_attempts - waiting for $service_name..."
        sleep 5
    done
    
    print_error "$service_name did not become healthy within expected time"
    return 1
}

# Function to cleanup containers
cleanup() {
    print_status "Cleaning up containers..."
    $DOCKER_COMPOSE_CMD down -v --remove-orphans
    if [ $? -eq 0 ]; then
        print_success "Cleanup completed"
    else
        print_warning "Cleanup had some issues"
    fi
}

# Function to show container logs
show_logs() {
    print_status "=== FalkorDB Logs ==="
    $DOCKER_COMPOSE_CMD logs falkordb | tail -20
    
    print_status "=== Redis PubSub Logs ==="
    $DOCKER_COMPOSE_CMD logs redis-pubsub | tail -20
    
    print_status "=== Task Service Logs ==="
    $DOCKER_COMPOSE_CMD logs tyl-task-service | tail -20
}

# Function to run integration tests
run_integration_tests() {
    print_status "Starting integration test environment..."
    
    # Start infrastructure services
    $DOCKER_COMPOSE_CMD up -d falkordb redis-pubsub
    
    # Wait for services to be healthy
    wait_for_service "falkordb" 12
    if [ $? -ne 0 ]; then
        show_logs
        cleanup
        exit 1
    fi
    
    wait_for_service "redis-pubsub" 12
    if [ $? -ne 0 ]; then
        show_logs
        cleanup
        exit 1
    fi
    
    print_success "Infrastructure services are running"
    
    # Run integration tests
    print_status "Running integration tests..."
    $DOCKER_COMPOSE_CMD run --rm tyl-task-test
    test_exit_code=$?
    
    if [ $test_exit_code -eq 0 ]; then
        print_success "Integration tests passed!"
    else
        print_error "Integration tests failed!"
        show_logs
    fi
    
    cleanup
    exit $test_exit_code
}

# Function to run full service stack
run_full_stack() {
    print_status "Starting full task service stack..."
    
    # Build and start all services
    $DOCKER_COMPOSE_CMD up --build -d
    
    # Wait for all services
    wait_for_service "falkordb" 12
    wait_for_service "redis-pubsub" 12
    
    print_status "Waiting for task service to start..."
    sleep 10
    
    # Check if task service is running
    if $DOCKER_COMPOSE_CMD ps tyl-task-service | grep -q "Up"; then
        print_success "Task service is running"
        
        # Test health endpoint
        sleep 5  # Give service time to fully start
        if curl -f http://localhost:3000/health &> /dev/null; then
            print_success "Task service health check passed"
            print_status "Task service is available at: http://localhost:3000"
            print_status "Health endpoint: http://localhost:3000/health"
            print_status "API docs: http://localhost:3000/api/v1/tasks"
        else
            print_warning "Task service health check failed"
        fi
    else
        print_error "Task service failed to start"
        show_logs
        cleanup
        exit 1
    fi
    
    print_status "Full stack is running. Use 'docker-compose logs -f' to follow logs"
    print_status "Use './scripts/test.sh stop' to stop the stack"
}

# Function to stop the stack
stop_stack() {
    print_status "Stopping task service stack..."
    cleanup
}

# Function to run local tests (without Docker)
run_local_tests() {
    print_status "Running local unit tests..."
    
    # Set environment variables for local testing
    export RUST_LOG=debug
    export TEST_MODE=true
    
    # Run unit tests
    cargo test --lib
    unit_exit_code=$?
    
    if [ $unit_exit_code -eq 0 ]; then
        print_success "Unit tests passed!"
    else
        print_error "Unit tests failed!"
        exit $unit_exit_code
    fi
}

# Function to validate environment
validate_environment() {
    print_status "Validating environment..."
    
    # Check required tools
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed"
        exit 1
    fi
    
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed"
        exit 1
    fi
    
    check_docker_compose
    
    # Check if ports are available
    if netstat -tuln 2>/dev/null | grep -q ":6379 "; then
        print_warning "Port 6379 is already in use (FalkorDB)"
    fi
    
    if netstat -tuln 2>/dev/null | grep -q ":6380 "; then
        print_warning "Port 6380 is already in use (Redis PubSub)"
    fi
    
    if netstat -tuln 2>/dev/null | grep -q ":3000 "; then
        print_warning "Port 3000 is already in use (Task Service)"
    fi
    
    print_success "Environment validation completed"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  test      - Run integration tests with Docker"
    echo "  unit      - Run unit tests locally"
    echo "  start     - Start the full service stack"
    echo "  stop      - Stop the service stack"
    echo "  logs      - Show service logs"
    echo "  validate  - Validate environment"
    echo "  help      - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 test                    # Run integration tests"
    echo "  $0 start                   # Start full stack"
    echo "  $0 logs                    # Show logs"
    echo ""
}

# Main script logic
case "${1:-test}" in
    "test")
        validate_environment
        run_integration_tests
        ;;
    "unit")
        run_local_tests
        ;;
    "start")
        validate_environment
        run_full_stack
        ;;
    "stop")
        stop_stack
        ;;
    "logs")
        show_logs
        ;;
    "validate")
        validate_environment
        ;;
    "help"|"-h"|"--help")
        show_usage
        ;;
    *)
        print_error "Unknown command: $1"
        show_usage
        exit 1
        ;;
esac