#!/bin/bash

# TYL Framework Microservice Generator
# 
# Usage: ./scripts/create-microservice.sh <service-name> <ServiceClass> <DomainModel> [output-dir]
#
# Examples:
#   ./scripts/create-microservice.sh order-service OrderService Order
#   ./scripts/create-microservice.sh user-auth UserAuthService User ../my-services
#   ./scripts/create-microservice.sh payment-gateway PaymentService Payment

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to display usage
usage() {
    cat << EOF
🚀 TYL Framework Microservice Generator

Usage: $0 <service-name> <ServiceClass> <DomainModel> [github-org] [output-dir]

Arguments:
  service-name   : Kebab-case name (e.g., 'order-service', 'user-auth')
  ServiceClass   : PascalCase service class (e.g., 'OrderService', 'UserAuthService')  
  DomainModel    : PascalCase domain model (e.g., 'Order', 'User')
  github-org     : Optional GitHub organization (default: the-yaml-life)
  output-dir     : Optional output directory (default: ../tyl-<service-name>)

Examples:
  $0 order-service OrderService Order
  $0 user-auth UserAuthService User my-org
  $0 payment-gateway PaymentService Payment my-org ~/my-services

Generated structure:
  tyl-<service-name>/
  ├── src/
  │   ├── domain.rs          # <DomainModel> and <ServiceClass>
  │   ├── handlers/          # API handlers
  │   ├── adapters/          # Database and external adapters
  │   └── events/            # Event-driven architecture
  ├── tests/                 # Integration tests
  ├── config/                # Environment configurations
  └── docker/                # Container setup

EOF
}

# Check prerequisites
check_prerequisites() {
    # Check if gh CLI is installed and authenticated
    if ! command -v gh &> /dev/null; then
        log_error "GitHub CLI (gh) is required but not installed"
        log_info "Install it from: https://cli.github.com/"
        exit 1
    fi
    
    # Check if user is authenticated with GitHub
    if ! gh auth status &> /dev/null; then
        log_error "GitHub CLI is not authenticated"
        log_info "Run 'gh auth login' to authenticate with GitHub"
        exit 1
    fi
    
    log_success "GitHub CLI is installed and authenticated"
}

# Validate arguments
if [[ $# -lt 3 || $# -gt 5 ]]; then
    log_error "Invalid number of arguments"
    usage
    exit 1
fi

# Check prerequisites
check_prerequisites

SERVICE_NAME="$1"
SERVICE_CLASS="$2"
DOMAIN_MODEL="$3"
GITHUB_ORG="${4:-the-yaml-life}"
OUTPUT_DIR="${5:-../tyl-${SERVICE_NAME}}"

# Validation patterns
SERVICE_NAME_PATTERN='^[a-z][a-z0-9]*(-[a-z0-9]+)*$'
CLASS_NAME_PATTERN='^[A-Z][a-zA-Z0-9]*$'

# Validate inputs
if [[ ! $SERVICE_NAME =~ $SERVICE_NAME_PATTERN ]]; then
    log_error "Service name must be kebab-case (e.g., 'order-service')"
    exit 1
fi

if [[ ! $SERVICE_CLASS =~ $CLASS_NAME_PATTERN ]]; then
    log_error "Service class must be PascalCase (e.g., 'OrderService')"
    exit 1
fi

if [[ ! $DOMAIN_MODEL =~ $CLASS_NAME_PATTERN ]]; then
    log_error "Domain model must be PascalCase (e.g., 'Order')"
    exit 1
fi

# Derived variables
SERVICE_NAME_UPPER=$(echo "$SERVICE_NAME" | tr '[:lower:]' '[:upper:]' | tr '-' '_')
SERVICE_NAME_SNAKE=$(echo "$SERVICE_NAME" | tr '-' '_')
SERVICE_NAME_PASCAL=$(echo "$SERVICE_NAME" | sed -r 's/(^|-)([a-z])/\U\2/g')
DOMAIN_MODEL_LOWER=$(echo "$DOMAIN_MODEL" | tr '[:upper:]' '[:lower:]')
DOMAIN_MODEL_SNAKE=$(echo "$DOMAIN_MODEL" | sed 's/\([A-Z]\)/_\L\1/g' | sed 's/^_//')

# Check if template directory exists
TEMPLATE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ ! -d "$TEMPLATE_DIR/src" ]]; then
    log_error "Template directory not found: $TEMPLATE_DIR"
    exit 1
fi

# Check if output directory already exists
if [[ -d "$OUTPUT_DIR" ]]; then
    log_error "Output directory already exists: $OUTPUT_DIR"
    exit 1
fi

log_info "🏗️  Generating microservice..."
log_info "   Service Name: $SERVICE_NAME"
log_info "   Service Class: $SERVICE_CLASS"  
log_info "   Domain Model: $DOMAIN_MODEL"
log_info "   GitHub Org: $GITHUB_ORG"
log_info "   Output Directory: $OUTPUT_DIR"

# Create output directory
mkdir -p "$OUTPUT_DIR"
log_success "Created output directory: $OUTPUT_DIR"

# Copy template structure
log_info "📂 Copying template structure..."
cp -r "$TEMPLATE_DIR"/* "$OUTPUT_DIR/"
cp -r "$TEMPLATE_DIR"/.env.example "$OUTPUT_DIR/" 2>/dev/null || true
cp -r "$TEMPLATE_DIR"/.gitignore "$OUTPUT_DIR/" 2>/dev/null || true
log_success "Template structure copied"

# Function to replace placeholders in a file
replace_placeholders() {
    local file="$1"
    
    # Skip binary files and target directory
    if [[ "$file" == *"/target/"* ]] || [[ -d "$file" ]]; then
        return
    fi
    
    # Check if file is text
    if file "$file" | grep -q "text\|empty"; then
        # Replace placeholders - order matters! (most specific first)
        sed -i "s|TASK_SERVICE|$SERVICE_NAME_UPPER|g" "$file"
        sed -i "s|task_service|$SERVICE_NAME_SNAKE|g" "$file"
        sed -i "s|TaskService|$SERVICE_NAME_PASCAL|g" "$file"
        sed -i "s|task|$DOMAIN_MODEL_LOWER|g" "$file"
        sed -i "s|task|$DOMAIN_MODEL_SNAKE|g" "$file"
        sed -i "s|task-service|$SERVICE_NAME|g" "$file"
        sed -i "s|TaskService|$SERVICE_CLASS|g" "$file"
        sed -i "s|Task|$DOMAIN_MODEL|g" "$file"
    fi
}

# Export the function and variables so they can be used by find
export -f replace_placeholders
export SERVICE_NAME SERVICE_CLASS DOMAIN_MODEL
export SERVICE_NAME_UPPER SERVICE_NAME_SNAKE SERVICE_NAME_PASCAL
export DOMAIN_MODEL_LOWER DOMAIN_MODEL_SNAKE

# Replace placeholders in all files
log_info "🔄 Replacing placeholders..."
find "$OUTPUT_DIR" -type f -exec bash -c 'replace_placeholders "$0"' {} \;
log_success "Placeholders replaced"

# Remove target directory if it exists
if [[ -d "$OUTPUT_DIR/target" ]]; then
    rm -rf "$OUTPUT_DIR/target"
    log_success "Removed target directory"
fi

# Rename files if needed
log_info "📝 Updating file names..."
# No specific files to rename in this template, but adding for future use

# Initialize git repository
log_info "🔧 Initializing git repository..."
cd "$OUTPUT_DIR"
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    git init
    git add .
    git commit -m "feat: Initial microservice generation from TYL template

Generated $SERVICE_CLASS microservice with:
- Service: $SERVICE_NAME
- Domain Model: $DOMAIN_MODEL
- Event-driven architecture
- Hexagonal architecture
- Complete test suite
- Docker support

🤖 Generated with TYL Framework template"
    log_success "Git repository initialized"
else
    log_warning "Git repository already exists, skipping initialization"
fi

# Create GitHub repository and push
log_info "🐙 Creating GitHub repository..."
REPO_NAME="tyl-${SERVICE_NAME}"
REPO_DESCRIPTION="${DOMAIN_MODEL} management microservice built with TYL framework"

# Check if repository already exists
if gh repo view "${GITHUB_ORG}/${REPO_NAME}" &> /dev/null; then
    log_warning "Repository ${GITHUB_ORG}/${REPO_NAME} already exists on GitHub"
    log_info "You can manually push to the existing repository with:"
    if ssh -T git@github.com &>/dev/null || [[ $? == 1 ]]; then
        log_info "  git remote add origin git@github.com:${GITHUB_ORG}/${REPO_NAME}.git"
    else
        log_info "  git remote add origin https://github.com/${GITHUB_ORG}/${REPO_NAME}.git"
    fi
    log_info "  git push -u origin main"
else
    # Create the repository
    if gh repo create "${GITHUB_ORG}/${REPO_NAME}" --description "$REPO_DESCRIPTION" --public; then
        log_success "GitHub repository created: https://github.com/${GITHUB_ORG}/${REPO_NAME}"
        
        # Add remote and push (use SSH if available, fallback to HTTPS)
        if ssh -T git@github.com &>/dev/null || [[ $? == 1 ]]; then
            git remote add origin "git@github.com:${GITHUB_ORG}/${REPO_NAME}.git"
        else
            git remote add origin "https://github.com/${GITHUB_ORG}/${REPO_NAME}.git"
        fi
        
        # Rename master to main if needed
        if git rev-parse --verify master &> /dev/null; then
            git branch -M main
        fi
        
        # Push to GitHub
        if git push -u origin main; then
            log_success "Code pushed to GitHub successfully!"
        else
            log_warning "Failed to push to GitHub, but repository was created"
            log_info "You can manually push with: git push -u origin main"
        fi
    else
        log_warning "Failed to create GitHub repository"
        log_info "You can manually create it later and push with:"
        log_info "  gh repo create ${GITHUB_ORG}/${REPO_NAME} --public"
        if ssh -T git@github.com &>/dev/null || [[ $? == 1 ]]; then
            log_info "  git remote add origin git@github.com:${GITHUB_ORG}/${REPO_NAME}.git"
        else
            log_info "  git remote add origin https://github.com/${GITHUB_ORG}/${REPO_NAME}.git"
        fi
        log_info "  git push -u origin main"
    fi
fi

# Test that the generated service compiles
log_info "🧪 Testing compilation..."
if cargo check --quiet; then
    log_success "Generated microservice compiles successfully!"
else
    log_error "Generated microservice has compilation errors"
    exit 1
fi

# Run tests
log_info "🧪 Running tests..."
if cargo test --lib --quiet; then
    log_success "All tests pass!"
else
    log_warning "Some tests failed - check generated test data"
fi

# Generate usage instructions
cat << EOF

🎉 SUCCESS! Microservice generated successfully!

📁 Location: $OUTPUT_DIR
🏗️  Service: $SERVICE_CLASS
📊 Domain: $DOMAIN_MODEL
🐙 GitHub: https://github.com/${GITHUB_ORG}/tyl-${SERVICE_NAME}

🚀 Next steps:
   cd $OUTPUT_DIR
   
   # Run the service
   cargo run
   
   # Run tests
   cargo test
   
   # Build for production
   cargo build --release
   
   # Start with Docker
   docker-compose up

📚 Key files to customize:
   src/domain.rs        - Add your business logic
   src/adapters/        - Implement database/external integrations
   src/events/examples.rs - Define your domain events
   config/*.toml        - Configure environments

🔗 Documentation:
   README.md           - Project overview
   CLAUDE.md           - Implementation details

Happy coding! 🚀

EOF

log_success "Microservice generation completed!"