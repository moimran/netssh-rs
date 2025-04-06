# Test Organization for netssh-rs

This directory contains tests for the netssh-core crate, organized into subdirectories based on their purpose and functionality.

## Directory Structure

- **common/** - Common functionality and integration tests
  - `integration_test.rs` - Basic integration tests
  - `parallel_execution_tests.rs` - Tests for parallel execution functionality

- **utils/** - Utility tests and test helper modules
  - `mock_device.rs` - Mock network device implementation for testing
  - `buffer_pool_test.rs` - Tests for buffer pool functionality
  - `semaphore_test.rs` - Tests for semaphore functionality
  - `settings_test.rs` - Tests for settings functionality

- **vendors/** - Vendor-specific tests
  - `common_tests.rs` - Tests that apply to all vendors
  - **cisco/** - Cisco-specific tests
    - `nxos_tests.rs` - Basic tests for Cisco NXOS devices
    - `nxos_specific_tests.rs` - Extended tests for NXOS-specific functionality

## Test Organization

The test organization follows these principles:

1. **Modularity**: Tests are organized by functionality and vendor
2. **Reusability**: Common test utilities are in the `utils` directory
3. **Isolation**: Vendor-specific tests are separated into their own directories
4. **Scalability**: Structure supports adding new vendor tests easily

## Running Tests

To run all tests:
```bash
cargo test
```

To run a specific test category (e.g., only NXOS tests):
```bash
cargo test --test vendors::cisco::nxos_tests
```

To run tests with real devices (requires environment variables):
```bash
# Set environment variables for device access
export NXOS_HOST=192.168.1.1
export NXOS_USERNAME=admin
export NXOS_PASSWORD=password

# Run with real devices
cargo test --test vendors::cisco::nxos_tests
```

To run tests with mock devices only:
```bash
MOCK_TESTS=1 cargo test --test vendors::cisco::nxos_tests
```

## Adding New Tests

When adding new tests:

1. Place utility functions in the appropriate subdirectory
2. For vendor-specific tests, add them to the respective vendor directory
3. For new vendors, create a new directory under `vendors/`
4. Update the relevant `mod.rs` files to expose the new modules 