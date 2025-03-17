# Makefile for netssh-rs Python bindings

.PHONY: all clean build develop test install setup

# Default target
all: build

# Setup development environment
setup:
	@echo "Setting up development environment..."
	pip install maturin
	pip install -r python/requirements.txt

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	rm -rf target/wheels
	rm -rf target/debug
	rm -rf target/release
	rm -rf *.egg-info
	rm -rf dist
	rm -rf build
	find . -name "__pycache__" -type d -exec rm -rf {} +

# Build Python wheel
build: setup
	@echo "Building Python wheel..."
	maturin build

# Build and install in development mode
develop: setup
	@echo "Building and installing in development mode..."
	maturin develop

# Run tests
test: develop
	@echo "Running tests..."
	cd python && python -m unittest test_netssh_rs.py

# Install Python package
install: build
	@echo "Installing Python package..."
	uv pip install --force-reinstall target/wheels/*.whl

# Run the example
run-example: develop
	@echo "Running example..."
	cd python && python example.py

# Run the backend integration example
run-backend: develop
	@echo "Running backend integration example..."
	cd python && python backend_integration_example.py

# Install development dependencies
deps:
	@echo "Installing development dependencies..."
	uv pip install -r python/requirements.txt

# Help
help:
	@echo "Available targets:"
	@echo "  all        - Build the Python wheel (default)"
	@echo "  setup      - Set up development environment (install maturin and dependencies)"
	@echo "  clean      - Clean build artifacts"
	@echo "  build      - Build Python wheel"
	@echo "  develop    - Build and install in development mode"
	@echo "  test       - Run tests"
	@echo "  install    - Install Python package"
	@echo "  run-example - Run the example"
	@echo "  run-backend - Run the backend integration example"
	@echo "  deps       - Install development dependencies"
	@echo "  help       - Show this help message"