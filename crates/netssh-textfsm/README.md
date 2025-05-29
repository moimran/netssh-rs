# NetSSH TextFSM Parser

A Rust implementation of TextFSM parsing functionality for network device command outputs. This crate provides both template selection logic and parsing capabilities, replacing the Python modules from `crates/netssh-python/python/netssh_rs/textfsm/`.

## Features

- **Automatic Template Selection**: Automatically finds the appropriate TextFSM template based on platform and command
- **Template Index Parsing**: Parses the template index file to build a mapping of platforms to templates
- **Command Matching**: Supports both exact regex matching and substring matching for commands
- **JSON Output**: Converts parsed data to JSON format
- **CLI Interface**: Command-line tool for parsing network device outputs
- **Library Interface**: Can be used as a library in other Rust projects

## Usage

### Command Line Interface

The crate provides a command-line tool with two main modes:

#### 1. Parse with Platform and Command (Recommended)

```bash
# Parse Cisco ASA show interface output
cargo run -- parse-command cisco_asa "show interface" path/to/output.txt

# Parse Cisco XR BGP neighbors output  
cargo run -- parse-command cisco_xr "show bgp neighbors" path/to/output.txt

# Specify custom template directory
cargo run -- parse-command cisco_ios "show version" output.txt --template-dir /custom/templates
```

#### 2. Parse with Direct Template File

```bash
# Use a specific template file directly
cargo run -- parse-template templates/cisco_asa_show_interface.textfsm output.txt
```

### Library Interface

#### Using the NetworkOutputParser

```rust
use cmdparser::NetworkOutputParser;
use std::path::PathBuf;

// Create parser with default template directory
let mut parser = NetworkOutputParser::new(None);

// Or specify custom template directory
let template_dir = PathBuf::from("/path/to/templates");
let mut parser = NetworkOutputParser::new(Some(template_dir));

// Parse command output
let result = parser.parse_output("cisco_asa", "show interface", &command_output)?;

// Convert to JSON
let json_result = parser.parse_to_json("cisco_asa", "show interface", &command_output)?;
```

#### Using Global Functions

```rust
use cmdparser::{parse_output, parse_output_to_json};

// Parse using global functions (uses default template directory)
let result = parse_output("cisco_asa", "show interface", &command_output)?;
let json_result = parse_output_to_json("cisco_asa", "show interface", &command_output)?;
```

## Template Selection Logic

The parser uses the following logic to find templates:

1. **Platform Matching**: Case-insensitive matching of platform names (e.g., "cisco_asa", "CISCO_ASA")
2. **Command Matching**: 
   - First tries exact regex matching against template command patterns
   - Falls back to substring matching if no exact match found
3. **Template Validation**: Ensures the selected template file exists before using it

### Supported Platforms

The parser supports templates for 56+ network device platforms including:

- Cisco ASA, IOS, IOS-XR, NX-OS
- Arista EOS
- Juniper JunOS
- HP/HPE ProCurve, Comware
- Huawei VRP
- Fortinet FortiGate
- And many more...

## Template Index Format

The template index file (`templates/index`) uses a CSV-like format:

```
Template, Platform, Command
cisco_asa_show_interface.textfsm, cisco_asa, show interface
cisco_asa_show_version.textfsm, cisco_asa, show version
cisco_ios_show_version.textfsm, cisco_ios, show version
```

The parser automatically handles:
- Command completion patterns (e.g., `sh[[ow]] ver[[sion]]` → `show version`)
- Case-insensitive matching
- Flexible column ordering

## Output Format

Parsed data is returned as a vector of dictionaries (IndexMap), where each dictionary represents one parsed record with field names as keys and values as JSON values.

Example output for Cisco ASA show interface:
```json
[
  {
    "INTERFACE": "GigabitEthernet0/0",
    "INTERFACE_ZONE": "outside", 
    "LINK_STATUS": "up",
    "PROTOCOL_STATUS": "up",
    "IP_ADDRESS": "10.0.0.5",
    "NETMASK": "255.255.255.252",
    "MAC_ADDRESS": "fa16.3eb0.c3d3",
    "MTU": "1500"
  }
]
```

## Error Handling

The parser handles various error conditions gracefully:

- **Missing Templates**: Returns `None` if no suitable template is found
- **Empty Input**: Returns `None` for empty command output
- **Invalid Template Directory**: Continues with empty template list
- **Parsing Errors**: Returns detailed error messages from the TextFSM engine

## Testing

Run the test suite to verify functionality:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_template_selection
```

The test suite includes:
- Template selection logic tests
- Parsing functionality tests with real network device outputs
- Error handling tests
- Integration tests with sample data

## Dependencies

- `indexmap`: For ordered dictionaries
- `regex`: For command pattern matching
- `serde_json`: For JSON serialization
- `lazy_static`: For global parser instance
- `clap`: For command-line interface

## Comparison with Python Implementation

This Rust implementation provides the same functionality as the original Python `parse_output.py` module:

| Feature | Python | Rust |
|---------|--------|------|
| Template Selection | ✅ | ✅ |
| Index File Parsing | ✅ | ✅ |
| Command Matching | ✅ | ✅ |
| JSON Output | ✅ | ✅ |
| Error Handling | ✅ | ✅ |
| Performance | Slower | Faster |
| Memory Usage | Higher | Lower |
| Type Safety | Runtime | Compile-time |

## Contributing

When adding new templates or modifying the parser:

1. Ensure templates are added to the `templates/index` file
2. Add appropriate test cases
3. Run the full test suite
4. Update documentation as needed

## License

This project follows the same license as the parent netssh-rs project.
