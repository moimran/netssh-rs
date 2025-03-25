# Detect Python executable
PYTHON3 := $(shell which python3)
ifeq ($(PYTHON3),)
	PYTHON3 := python
endif

# Detect Python virtual environment
ifdef VIRTUAL_ENV
	# Virtual environment is active
	PYTHON := $(VIRTUAL_ENV)/bin/python
	PIP := $(VIRTUAL_ENV)/bin/pip
	MATURIN := $(VIRTUAL_ENV)/bin/maturin
	VENV_INFO := (using venv: $(VIRTUAL_ENV))
else
	# No virtual environment detected
	PYTHON := $(PYTHON3)
	PIP := pip3
	MATURIN := maturin
	VENV_INFO := (using system Python)
endif

.PHONY: all clean build-core build-api build-python develop-python test-core test-api test-python \
	install-python setup-python run-example run-develop-example run-build-example run-api help

# Default target
all: build-core build-api build-python

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
	test -d .venv || $(PYTHON3) -m venv .venv
	. .venv/bin/activate && pip install --upgrade pip
	. .venv/bin/activate && pip install --no-cache-dir maturin
	. .venv/bin/activate && cd crates/netssh-python && pip install --no-cache-dir -r python/requirements.txt

# Build Python wheel
build-python: setup-python
	@echo "Building Python wheel $(VENV_INFO)..."
	. .venv/bin/activate && cd crates/netssh-python && maturin build

# Build and install Python bindings in development mode
develop-python: setup-python
	@echo "Building and installing Python bindings in development mode $(VENV_INFO)..."
	cd crates/netssh-python && $(MATURIN) develop

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
	@echo "Running Python tests $(VENV_INFO)..."
	cd crates/netssh-python/python && $(PYTHON) -m unittest test_netssh_rs.py

# Test all components
test: test-core test-api test-python
	@echo "All tests passed!"

# Install Python package
install-python: build-python
	@echo "Installing Python package $(VENV_INFO)..."
	. .venv/bin/activate && pip install --force-reinstall target/wheels/*manylinux*.whl

# Run Python example using development mode
run-develop-example: develop-python
	@echo "Running Python example using development mode $(VENV_INFO)..."
	cd crates/netssh-python/python && $(PYTHON) example.py

# Run Python example using installed package
run-build-example: build-python
	@echo "Uninstalling existing package $(VENV_INFO)..."
	$(PIP) uninstall -y netssh_rs || true
	@echo "Installing new package $(VENV_INFO)..."
	$(PIP) install --force-reinstall crates/netssh-python/target/wheels/*.whl
	@echo "Running Python example using installed package $(VENV_INFO)..."
	cd crates/netssh-python/python && $(PYTHON) example.py

# Alias for run-build-example
run-example: run-build-example

# Run API server
run-api: build-api
	@echo "Running API server..."
	cargo run -p netssh-api

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf crates/netssh-python/target/

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
	@echo ""
	@echo "Current Python: $(PYTHON) $(VENV_INFO)"