# Unified Configuration Migration Guide

## Overview

This document outlines the migration from individual crate configurations to a unified workspace configuration system. The unified approach provides better consistency, easier deployment, and centralized configuration management.

## Benefits of Unified Configuration

### ✅ **Advantages**
- **Single Source of Truth**: One config file for all crates
- **Consistency**: Shared settings stay in sync across crates
- **Simplified Deployment**: One config file to manage in containers/environments
- **Environment Variable Support**: Hierarchical environment variable overrides
- **Type Safety**: Compile-time validation of configuration structure
- **Default Values**: Sensible defaults for all settings
- **Easier Configuration Management**: No configuration drift between crates

### ❌ **Trade-offs**
- **Larger Config File**: More complex structure
- **Shared Dependency**: All crates depend on shared-config
- **Migration Effort**: Need to update existing crate configurations

## Implementation Structure

### File Organization
```
netssh-rs/
├── config.toml                    # Unified workspace configuration
├── crates/
│   ├── shared-config/             # NEW: Shared configuration crate
│   │   ├── src/lib.rs            # Configuration structures and loading
│   │   └── examples/             # Configuration examples
│   ├── scheduler/
│   │   └── config.toml           # DEPRECATED: Move to workspace root
│   ├── netssh-core/              # Uses shared-config
│   ├── netssh-textfsm/           # Uses shared-config
│   └── netssh-python/            # Uses shared-config
```

### Configuration Hierarchy
```toml
# config.toml (workspace root)
[global]                          # Settings for all crates
[scheduler]                       # Scheduler-specific settings
[scheduler.database]              # Nested scheduler settings
[scheduler.worker]                # Worker configuration
[netssh]                          # netssh-core settings
[netssh.logging]                  # Nested netssh settings
[textfsm]                         # TextFSM settings
[python]                          # Python integration settings
```

## Migration Steps

### Step 1: Create Shared Configuration Crate ✅
- [x] Created `crates/shared-config/` with configuration structures
- [x] Added to workspace members
- [x] Implemented hierarchical configuration loading
- [x] Added environment variable support with `NETSSH_` prefix

### Step 2: Move Configuration to Workspace Root ✅
- [x] Created unified `config.toml` at workspace root
- [x] Includes all crate-specific configurations
- [x] Maintains backward compatibility with existing settings

### Step 3: Update Individual Crates (In Progress)

#### Scheduler Migration
```rust
// Before (scheduler-specific config)
use crate::config::Config;
let config = Config::from_env()?;

// After (shared config)
use shared_config::WorkspaceConfig;
let config = WorkspaceConfig::load()?;
let scheduler_config = config.scheduler();
```

#### netssh-core Integration
```rust
// In netssh-core
use shared_config::WorkspaceConfig;
let config = WorkspaceConfig::load()?;
let netssh_config = config.netssh();
```

### Step 4: Environment Variable Migration

#### Before (per-crate prefixes)
```bash
export SCHEDULER_DATABASE_URL="sqlite:prod.db"
export SCHEDULER_WORKER_CONCURRENCY=8
```

#### After (unified NETSSH prefix)
```bash
export NETSSH_SCHEDULER_DATABASE_URL="sqlite:prod.db"
export NETSSH_SCHEDULER_WORKER_CONCURRENCY=8
export NETSSH_GLOBAL_LOG_LEVEL=debug
export NETSSH_NETSSH_DEFAULT_SSH_TIMEOUT=60
```

## Configuration Examples

### Development Configuration
```toml
[global]
log_level = "debug"
environment = "development"

[scheduler.worker]
concurrency = 2
connection_reuse = true

[netssh.logging]
session_logging = true
debug_mode = true
```

### Production Configuration
```toml
[global]
log_level = "info"
environment = "production"

[scheduler.worker]
concurrency = 8
connection_reuse = true
max_connections_per_worker = 20

[netssh.security]
strict_host_key_checking = true
max_auth_attempts = 3
```

### Container/Docker Configuration
```toml
[scheduler.database]
url = "sqlite:/data/scheduler.db"

[scheduler.server]
host = "0.0.0.0"
port = 8080

[netssh]
connection_pool_size = 100
```

## Usage Patterns

### Loading Configuration in Crates
```rust
use shared_config::WorkspaceConfig;

// Load unified configuration
let config = WorkspaceConfig::load()?;

// Access crate-specific configuration
let scheduler_config = config.scheduler();
let netssh_config = config.netssh();
let global_config = config.global();

// Use configuration
let timeout = Duration::from_secs(netssh_config.default_ssh_timeout);
let log_level = &global_config.log_level;
```

### Environment Variable Overrides
```bash
# Override specific settings
export NETSSH_SCHEDULER_WORKER_CONCURRENCY=16
export NETSSH_NETSSH_DEFAULT_SSH_TIMEOUT=120
export NETSSH_GLOBAL_LOG_LEVEL=trace

# Run application with overrides
cargo run
```

### Configuration Validation
```rust
// The shared-config crate provides type-safe configuration
// with compile-time validation and runtime defaults

let config = WorkspaceConfig::load()?;

// All settings have sensible defaults
assert_eq!(config.global().default_timeout_seconds, 30);
assert_eq!(config.netssh().default_port, 22);
assert_eq!(config.scheduler().worker.concurrency, 4);
```

## Migration Checklist

### Phase 1: Infrastructure ✅
- [x] Create shared-config crate
- [x] Define configuration structures
- [x] Implement configuration loading
- [x] Add workspace root config.toml
- [x] Test unified configuration loading

### Phase 2: Crate Migration (Next Steps)
- [ ] Update scheduler to use shared-config
- [ ] Update netssh-core to use shared-config
- [ ] Update netssh-textfsm to use shared-config
- [ ] Update netssh-python to use shared-config

### Phase 3: Cleanup
- [ ] Remove individual config files
- [ ] Update documentation
- [ ] Update deployment scripts
- [ ] Update CI/CD configurations

## Deployment Considerations

### Container Deployment
```dockerfile
# Copy unified configuration
COPY config.toml /app/config.toml

# Set environment-specific overrides
ENV NETSSH_GLOBAL_ENVIRONMENT=production
ENV NETSSH_SCHEDULER_DATABASE_URL=sqlite:/data/scheduler.db
```

### Kubernetes Deployment
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: netssh-config
data:
  config.toml: |
    [global]
    environment = "production"
    [scheduler.database]
    url = "sqlite:/data/scheduler.db"
---
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
      - name: scheduler
        env:
        - name: NETSSH_SCHEDULER_WORKER_CONCURRENCY
          value: "8"
        volumeMounts:
        - name: config
          mountPath: /app/config.toml
          subPath: config.toml
```

## Best Practices

### Configuration Organization
1. **Group Related Settings**: Use nested sections for related configuration
2. **Consistent Naming**: Use snake_case for all configuration keys
3. **Sensible Defaults**: Provide defaults that work for development
4. **Environment Overrides**: Use environment variables for deployment-specific settings

### Security Considerations
1. **Sensitive Data**: Use environment variables for secrets, not config files
2. **File Permissions**: Restrict access to configuration files in production
3. **Validation**: Validate configuration values at startup

### Documentation
1. **Comment Configuration**: Add comments explaining complex settings
2. **Example Configurations**: Provide examples for different environments
3. **Migration Guides**: Document changes when updating configuration structure

## Conclusion

The unified configuration system provides a robust foundation for managing settings across all netssh-rs crates. It simplifies deployment, ensures consistency, and provides type-safe configuration management while maintaining flexibility for different environments and use cases.
