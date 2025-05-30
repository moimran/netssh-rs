# Builder Pattern API Guide

This guide demonstrates the new builder pattern API for netssh-core, which provides a clean, self-documenting way to send commands with optional parameters.

## 🎯 Quick Comparison

### New Builder API (Only Way)
```rust
// Clean and simple - just add .execute()!
device.send_command("show version").execute()?;

// Self-documenting with named parameters
device.send_command("show version")
    .timeout(30.0)
    .execute()?;
```

## 🚀 Builder Pattern Examples

### 1. Simple Commands (All Defaults)
```rust
// Just add .execute() - couldn't be simpler!
let version = device.send_command("show version").execute()?;
let interfaces = device.send_command("show ip interface brief").execute()?;
let uptime = device.send_command("show uptime").execute()?;
```

### 2. Commands with Custom Timeout
```rust
// Long-running commands with custom timeout
let tech_support = device.send_command("show tech-support")
    .timeout(120.0)  // 2 minutes
    .execute()?;

let config = device.send_command("show running-config")
    .timeout(60.0)   // 1 minute
    .execute()?;
```

### 3. Commands with Multiple Options
```rust
// Full control with readable, self-documenting code
let output = device.send_command("show running-config")
    .timeout(60.0)           // Custom timeout
    .strip_prompt(false)     // Keep prompt in output
    .strip_command(true)     // Remove command echo
    .normalize(true)         // Normalize line endings
    .cmd_verify(true)        // Verify command echo
    .execute()?;
```

### 4. Commands with Expect String
```rust
// Wait for specific pattern instead of prompt
let output = device.send_command("copy running-config startup-config")
    .expect_string("Copy complete")
    .timeout(30.0)
    .execute()?;
```

## 📋 Available Builder Methods

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `.timeout(seconds)` | `f64` | `10.0` | Read timeout in seconds |
| `.expect_string(pattern)` | `&str` | `None` | Wait for specific pattern instead of prompt |
| `.auto_find_prompt(bool)` | `bool` | `false` | Automatically find device prompt |
| `.strip_prompt(bool)` | `bool` | `true` | Remove prompt from output |
| `.strip_command(bool)` | `bool` | `true` | Remove command echo from output |
| `.normalize(bool)` | `bool` | `true` | Normalize line endings |
| `.cmd_verify(bool)` | `bool` | `false` | Verify command echoing |

## 🔧 Configuration Commands

The builder pattern also works with configuration commands:

```rust
let commands = vec![
    "interface eth0".to_string(),
    "ip address 192.168.1.1/24".to_string(),
    "no shutdown".to_string(),
];

// Simple config with defaults
device.send_config_set(commands).execute()?;

// Config with custom options
device.send_config_set(commands)
    .timeout(60.0)
    .exit_config_mode(false)
    .fast_cli(true)
    .execute()?;
```

## ✅ Benefits

1. **Readable**: Self-documenting code that's easy to understand
2. **Flexible**: Only specify the options you need
3. **Safe**: Compile-time checking of parameter types
4. **Backward Compatible**: Old API still works
5. **Consistent**: Same pattern for all command types

## 🔄 Migration Guide

The new API uses `send_command` directly with builder pattern - no more multiple parameters!

### New Builder API
```rust
// Clean and self-documenting
device.send_command("show version")
    .timeout(30.0)
    .strip_prompt(false)
    .execute()?;
```

## 📚 Example Files

Check out these example files to see the builder pattern in action:

- `basic_connection.rs` - Comprehensive builder pattern examples
- `multiple_commands.rs` - Builder pattern with different configurations
- `device_info.rs` - Real-world usage in device information gathering
- `env_config.rs` - Simple builder usage

## 🎉 Conclusion

The builder pattern makes netssh-core APIs much more user-friendly while maintaining full backward compatibility. Start using it today for cleaner, more maintainable code!
