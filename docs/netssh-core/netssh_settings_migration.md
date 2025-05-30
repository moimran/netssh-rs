# NetSSH Settings Migration Guide

## Overview

This document describes the migration from netssh-core's individual `settings.json` file to the unified workspace configuration system using `config.toml`.

## Migration Summary

### Before (Old System)
- **File**: `crates/netssh-core/settings.json`
- **Format**: JSON with flat structure
- **Scope**: Only netssh-core crate
- **Initialization**: `Settings::init(Some("settings.json"))`

### After (New System)
- **File**: `config.toml` (workspace root)
- **Format**: TOML with hierarchical structure
- **Scope**: All crates in workspace
- **Initialization**: `Settings::init_from_workspace_config()`

## Configuration Structure Changes

### Old settings.json Structure
```json
{
    "network": {
        "tcp_connect_timeout_secs": 60,
        "tcp_read_timeout_secs": 30,
        // ... other network settings
    },
    "ssh": {
        "blocking_timeout_secs": 1,
        "auth_timeout_secs": 30,
        // ... other SSH settings
    },
    // ... other sections
}
```

### New config.toml Structure
```toml
[netssh.network]
tcp_connect_timeout_secs = 60
tcp_read_timeout_secs = 30
# ... other network settings

[netssh.ssh]
blocking_timeout_secs = 1
auth_timeout_secs = 30
# ... other SSH settings

# ... other sections
```

## Code Migration

### Old Initialization
```rust
// Initialize with settings.json
Settings::init(Some("crates/netssh-core/settings.json")).expect("Failed to initialize settings");
```

### New Initialization
```rust
// Initialize with shared workspace config (recommended)
Settings::init_from_workspace_config().expect("Failed to initialize settings");

// Or fallback to defaults
Settings::init(None).expect("Failed to initialize settings");
```

## Benefits of Migration

1. **Unified Configuration**: Single source of truth for all crates
2. **Environment Variables**: Support for `NETSSH_*` environment variables
3. **Hierarchical Structure**: Better organization of settings
4. **Type Safety**: Compile-time validation of configuration structure
5. **Consistency**: Same configuration system across all crates
6. **Maintainability**: Easier to manage and update settings

## Environment Variable Support

The new system supports environment variable overrides:

```bash
# Override network timeout
export NETSSH_NETSSH_NETWORK_TCP_CONNECT_TIMEOUT_SECS=120

# Override SSH settings
export NETSSH_NETSSH_SSH_AUTH_TIMEOUT_SECS=60

# Override buffer settings
export NETSSH_NETSSH_BUFFER_READ_BUFFER_SIZE=131072
```

## Backward Compatibility

The old `Settings::init()` method is still available for backward compatibility, but the new `Settings::init_from_workspace_config()` method is recommended for new code.

## Migration Steps

1. ✅ **Extended shared-config**: Added comprehensive netssh settings structure
2. ✅ **Updated config.toml**: Added all detailed netssh configuration sections
3. ✅ **Modified netssh-core**: Added `init_from_workspace_config()` method
4. ✅ **Updated examples**: Fixed all examples to use new structure
5. ✅ **Added tests**: Verified new configuration system works
6. ✅ **Updated documentation**: Updated README with new initialization method

## Files Modified

- `crates/shared-config/src/lib.rs` - Extended configuration structures
- `config.toml` - Added comprehensive netssh settings
- `crates/netssh-core/src/settings.rs` - Added workspace config integration
- `crates/netssh-core/src/config.rs` - Updated to use new structure
- `crates/netssh-core/README.md` - Updated documentation
- Various example files - Fixed to use new structure

## Next Steps

1. **Remove settings.json**: The old settings.json file can be removed
2. **Update applications**: Update any applications using netssh-core to use the new initialization method
3. **Documentation**: Update any external documentation referencing the old settings system

## Testing

All existing tests pass with the new configuration system. The new `test_settings_init_from_workspace_config()` test verifies the workspace configuration integration works correctly.
