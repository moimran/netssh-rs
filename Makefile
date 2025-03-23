# Makefile for netssh-rs workspace

.PHONY: all clean build-core build-api build-python develop-python test-core test-api test-python \
        install-python setup-python run-example run-develop-example run-build-example run-api help

# Default target
all: build-core build-api build-python

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf crates/netssh-python/target/wheels
	rm -rf *.egg-info
	rm -rf dist
	rm -rf build
	find . -name "__pycache__" -type d -exec rm -rf {} +

# Build core library
build-core:
	@echo "Building core library..."
	cargo build -p netssh-core

# Build API server
build-api:
	@echo "Building API server..."
	cargo build -p netssh-api

# Setup Python environment
setup-python:
	@echo "Setting up Python development environment..."
	cd crates/netssh-python && pip install maturin
	cd crates/netssh-python && pip install -r python/requirements.txt

# Build Python wheel
build-python: setup-python
	@echo "Building Python wheel..."
	cd crates/netssh-python && maturin build

# Build and install Python bindings in development mode
develop-python: setup-python
	@echo "Building and installing Python bindings in development mode..."
	cd crates/netssh-python && maturin develop

# Run core tests
test-core:
	@echo "Running core tests..."
	cargo test -p netssh-core

# Run API tests
test-api:
	@echo "Running API tests..."
	cargo test -p netssh-api

# Run Python tests
test-python: develop-python
	@echo "Running Python tests..."
	cd crates/netssh-python/python && python -m unittest test_netssh_rs.py

# Test all components
test: test-core test-api test-python
	@echo "All tests passed!"

# Install Python package
install-python: build-python
	@echo "Installing Python package..."
	uv pip install --force-reinstall crates/netssh-python/target/wheels/*.whl

# Run Python example using development mode
run-develop-example: develop-python
	@echo "Running Python example using development mode..."
	cd crates/netssh-python/python && python example.py

# Run Python example using installed package
run-build-example: build-python
	@echo "Uninstalling existing package..."
	pip uninstall -y netssh_rs || true
	@echo "Installing new package..."
	pip install --force-reinstall crates/netssh-python/target/wheels/*.whl
	@echo "Running Python example using installed package..."
	cd crates/netssh-python/python && python example.py

# Alias for run-build-example
run-example: run-build-example

# Run API server
run-api: build-api
	@echo "Running API server..."
	cargo run -p netssh-api

# Help
help:
	@echo "Available targets:"
	@echo "  all                   - Build all components (default)"
	@echo "  clean                 - Clean build artifacts"
	@echo "  build-core            - Build core library only"
	@echo "  build-api             - Build API server only"
	@echo "  build-python          - Build Python wheel only"
	@echo "  setup-python          - Set up Python development environment"
	@echo "  develop-python        - Build and install Python bindings in development mode"
	@echo "  test-core             - Run core tests only"
	@echo "  test-api              - Run API tests only"
	@echo "  test-python           - Run Python tests only"
	@echo "  test                  - Run all tests"
	@echo "  install-python        - Install Python package"
	@echo "  run-develop-example   - Run Python example using development mode"
	@echo "  run-build-example     - Run Python example using installed package"
	@echo "  run-example           - Alias for run-build-example"
	@echo "  run-api               - Run API server"
	@echo "  help                  - Show this help message"