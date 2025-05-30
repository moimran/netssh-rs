#!/bin/bash

# Job Scheduler Demo Script
# This script demonstrates the core functionality of the job scheduler

set -e  # Exit on any error

echo "🚀 Job Scheduler Demo Script"
echo "================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_step() {
    echo -e "${BLUE}📋 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if server is running
check_server() {
    if curl -s http://127.0.0.1:8080/api/health > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Wait for server to start
wait_for_server() {
    print_step "Waiting for server to start..."
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if check_server; then
            print_success "Server is running!"
            return 0
        fi
        echo -n "."
        sleep 1
        attempt=$((attempt + 1))
    done
    
    print_error "Server failed to start within 30 seconds"
    return 1
}

# Test API endpoints
test_api() {
    print_step "Testing API endpoints..."
    
    # Test health endpoint
    echo "Testing health endpoint..."
    response=$(curl -s http://127.0.0.1:8080/api/health)
    if echo "$response" | grep -q "healthy"; then
        print_success "Health check passed"
    else
        print_error "Health check failed"
        echo "Response: $response"
    fi
    
    # Test jobs list endpoint
    echo "Testing jobs list endpoint..."
    response=$(curl -s http://127.0.0.1:8080/api/jobs)
    if echo "$response" | grep -q "count"; then
        print_success "Jobs list endpoint working"
    else
        print_error "Jobs list endpoint failed"
        echo "Response: $response"
    fi
    
    # Test job creation
    echo "Testing job creation..."
    if [ -f "test_job.json" ]; then
        response=$(curl -s -X POST http://127.0.0.1:8080/api/jobs \
            -H "Content-Type: application/json" \
            -d @test_job.json)
        if echo "$response" | grep -q "Job created successfully"; then
            print_success "Job creation successful"
            echo "Response: $response"
        else
            print_warning "Job creation response: $response"
        fi
    else
        print_warning "test_job.json not found, skipping job creation test"
    fi
    
    # Test connections endpoint
    echo "Testing connections endpoint..."
    response=$(curl -s http://127.0.0.1:8080/api/connections)
    if echo "$response" | grep -q "profiles"; then
        print_success "Connections endpoint working"
    else
        print_error "Connections endpoint failed"
        echo "Response: $response"
    fi
}

# Main demo function
run_demo() {
    print_step "Starting Job Scheduler Demo"
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust first."
        exit 1
    fi
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml not found. Please run this script from the project root."
        exit 1
    fi
    
    # Run tests first
    print_step "Running tests..."
    if cargo test --quiet; then
        print_success "All tests passed"
    else
        print_error "Some tests failed"
        exit 1
    fi
    
    # Run basic usage example
    print_step "Running basic usage example..."
    if cargo run --example basic_usage --quiet; then
        print_success "Basic usage example completed"
    else
        print_error "Basic usage example failed"
        exit 1
    fi
    
    # Check if server is already running
    if check_server; then
        print_warning "Server is already running on port 8080"
        print_step "Testing existing server..."
        test_api
    else
        print_step "Starting server in background..."
        
        # Start server in background
        cargo run --quiet &
        SERVER_PID=$!
        
        # Wait for server to start
        if wait_for_server; then
            # Test API endpoints
            test_api
            
            # Kill the server
            print_step "Stopping server..."
            kill $SERVER_PID 2>/dev/null || true
            wait $SERVER_PID 2>/dev/null || true
            print_success "Server stopped"
        else
            # Kill the server if it failed to start properly
            kill $SERVER_PID 2>/dev/null || true
            exit 1
        fi
    fi
    
    print_success "Demo completed successfully!"
    echo ""
    echo "🎯 Next steps:"
    echo "1. Start the server: cargo run"
    echo "2. Open the web UI: http://127.0.0.1:8080/board"
    echo "3. Test the API endpoints using the examples above"
    echo "4. Check the DEMO.md file for detailed documentation"
}

# Handle script interruption
cleanup() {
    if [ ! -z "$SERVER_PID" ]; then
        print_step "Cleaning up..."
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi
    exit 0
}

trap cleanup INT TERM

# Run the demo
run_demo
