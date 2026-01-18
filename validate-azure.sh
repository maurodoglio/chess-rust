#!/bin/bash

# Azure Deployment Validation Script
# This script validates that all necessary files and dependencies are present
# for deploying the chess-rust application to Azure using Docker Compose

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0
WARNINGS=0

# Helper functions
print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED++))
}

print_error() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED++))
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
    ((WARNINGS++))
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Main validation
main() {
    print_header "Chess-Rust Azure Deployment Validation"
    
    # 1. Check Prerequisites
    print_header "1. Checking Prerequisites"
    
    if command_exists docker; then
        DOCKER_VERSION=$(docker --version | cut -d' ' -f3 | cut -d',' -f1)
        print_success "Docker installed (version $DOCKER_VERSION)"
    else
        print_error "Docker is not installed"
        echo "  Install from: https://docs.docker.com/get-docker/"
    fi
    
    if command_exists "docker compose"; then
        COMPOSE_VERSION=$(docker compose version | cut -d' ' -f4 | cut -d'v' -f2)
        print_success "Docker Compose V2 installed (version $COMPOSE_VERSION)"
    else
        print_error "Docker Compose V2 is not installed"
        echo "  Install Docker Desktop or Docker Compose V2"
    fi
    
    if command_exists az; then
        AZ_VERSION=$(az version --query '\"azure-cli\"' -o tsv)
        print_success "Azure CLI installed (version $AZ_VERSION)"
        
        # Check if logged in
        if az account show >/dev/null 2>&1; then
            SUBSCRIPTION=$(az account show --query name -o tsv)
            print_success "Logged into Azure (subscription: $SUBSCRIPTION)"
        else
            print_warning "Not logged into Azure - run 'az login' before deploying"
        fi
    else
        print_error "Azure CLI is not installed"
        echo "  Install from: https://docs.microsoft.com/en-us/cli/azure/install-azure-cli"
    fi
    
    if command_exists curl; then
        print_success "curl installed"
    else
        print_warning "curl not installed (optional, used for testing)"
    fi
    
    if command_exists jq; then
        print_success "jq installed"
    else
        print_warning "jq not installed (optional, useful for JSON parsing)"
    fi
    
    # 2. Check Required Files
    print_header "2. Checking Required Files"
    
    required_files=(
        "Dockerfile"
        "docker-compose.azure.yml"
        ".env.azure.example"
        "frontend/Dockerfile"
        "frontend/docker-entrypoint.sh"
        "frontend/nginx.conf"
        "Cargo.toml"
        "AZURE-DEPLOYMENT.md"
    )
    
    for file in "${required_files[@]}"; do
        if [ -f "$file" ]; then
            print_success "Found $file"
        else
            print_error "Missing $file"
        fi
    done
    
    # 3. Check Configuration Files
    print_header "3. Checking Configuration"
    
    if [ -f ".env.azure" ]; then
        print_success "Found .env.azure configuration file"
        
        # Check for required variables
        if grep -q "API_URL=" .env.azure; then
            API_URL=$(grep "API_URL=" .env.azure | cut -d'=' -f2)
            print_info "API_URL is set to: $API_URL"
        else
            print_warning "API_URL not set in .env.azure"
        fi
        
        if grep -q "BACKEND_PORT=" .env.azure; then
            print_success "BACKEND_PORT configured"
        else
            print_warning "BACKEND_PORT not set (will use default)"
        fi
    else
        print_warning ".env.azure not found (optional)"
        print_info "You can create it from .env.azure.example:"
        echo "  cp .env.azure.example .env.azure"
    fi
    
    # 4. Validate Docker Compose File
    print_header "4. Validating Docker Compose Configuration"
    
    if [ -f "docker-compose.azure.yml" ]; then
        if docker compose -f docker-compose.azure.yml config >/dev/null 2>&1; then
            print_success "docker-compose.azure.yml is valid"
        else
            print_error "docker-compose.azure.yml has syntax errors"
            echo "  Run: docker compose -f docker-compose.azure.yml config"
        fi
    fi
    
    # 5. Test Docker Build (Optional)
    print_header "5. Docker Build Test (Optional)"
    
    echo -e "${YELLOW}Do you want to test building the Docker images? (y/N)${NC}"
    read -r -t 10 response || response="n"
    
    if [[ "$response" =~ ^[Yy]$ ]]; then
        print_info "Building backend image..."
        if docker build -t chess-backend-test -f Dockerfile . >/dev/null 2>&1; then
            print_success "Backend image builds successfully"
            docker rmi chess-backend-test >/dev/null 2>&1
        else
            print_error "Backend image failed to build"
            echo "  Run: docker build -t chess-backend-test -f Dockerfile ."
        fi
        
        print_info "Building frontend image..."
        if docker build -t chess-frontend-test -f frontend/Dockerfile ./frontend >/dev/null 2>&1; then
            print_success "Frontend image builds successfully"
            docker rmi chess-frontend-test >/dev/null 2>&1
        else
            print_error "Frontend image failed to build"
            echo "  Run: docker build -t chess-frontend-test -f frontend/Dockerfile ./frontend"
        fi
    else
        print_info "Skipping Docker build test"
    fi
    
    # 6. Check Azure Context
    print_header "6. Checking Azure Docker Context"
    
    if command_exists docker && command_exists az; then
        if docker context ls | grep -q "aci"; then
            print_success "Azure ACI context exists"
            CURRENT_CONTEXT=$(docker context show)
            print_info "Current context: $CURRENT_CONTEXT"
            
            if [ "$CURRENT_CONTEXT" != "default" ]; then
                print_warning "Not using default context - make sure this is intentional"
            fi
        else
            print_info "No Azure ACI context found (normal for first-time setup)"
            echo "  Create one with: docker context create aci <context-name>"
        fi
    fi
    
    # 7. Check Azure Resources
    print_header "7. Checking Azure Resources (Optional)"
    
    if command_exists az && az account show >/dev/null 2>&1; then
        print_info "Checking for existing resource groups..."
        
        # Check if resource group exists
        if [ -f ".env.azure" ] && grep -q "RESOURCE_GROUP=" .env.azure; then
            RG=$(grep "RESOURCE_GROUP=" .env.azure | cut -d'=' -f2 | tr -d ' ' | tr -d '"')
            if [ -n "$RG" ]; then
                if az group show --name "$RG" >/dev/null 2>&1; then
                    print_success "Resource group '$RG' exists"
                else
                    print_info "Resource group '$RG' does not exist yet"
                    echo "  Create with: az group create --name $RG --location <location>"
                fi
            fi
        fi
        
        # List existing container instances
        CONTAINER_COUNT=$(az container list --query "length([])" -o tsv 2>/dev/null || echo "0")
        if [ "$CONTAINER_COUNT" -gt 0 ]; then
            print_info "Found $CONTAINER_COUNT existing container instance(s)"
        else
            print_info "No existing container instances found"
        fi
    fi
    
    # 8. Validate Documentation
    print_header "8. Checking Documentation"
    
    docs=(
        "README.md"
        "AZURE-DEPLOYMENT.md"
        "DOCKER-DEPLOYMENT.md"
        "DEPLOYMENT.md"
    )
    
    for doc in "${docs[@]}"; do
        if [ -f "$doc" ]; then
            print_success "Found $doc"
        else
            print_warning "Missing $doc"
        fi
    done
    
    # Summary
    print_header "Validation Summary"
    
    echo -e "${GREEN}Passed:${NC}   $PASSED"
    echo -e "${YELLOW}Warnings:${NC} $WARNINGS"
    echo -e "${RED}Failed:${NC}   $FAILED"
    
    echo ""
    
    if [ $FAILED -eq 0 ]; then
        if [ $WARNINGS -eq 0 ]; then
            echo -e "${GREEN}✓ All checks passed! Ready for Azure deployment.${NC}"
        else
            echo -e "${YELLOW}⚠ Validation passed with warnings. Review warnings above.${NC}"
        fi
        echo ""
        echo -e "${BLUE}Next steps:${NC}"
        echo "1. Ensure you're logged into Azure: az login"
        echo "2. Create a resource group: az group create --name <rg-name> --location <location>"
        echo "3. Create Azure context: docker context create aci <context-name>"
        echo "4. Deploy: docker compose -f docker-compose.azure.yml up"
        echo ""
        echo "See AZURE-DEPLOYMENT.md for detailed instructions."
        exit 0
    else
        echo -e "${RED}✗ Validation failed. Please fix the errors above before deploying.${NC}"
        exit 1
    fi
}

# Run main function
main
