# Network SSH Architecture

```mermaid
%%{init: {
  'theme': 'base',
  'themeVariables': {
    'primaryColor': '#2D4059',
    'primaryTextColor': '#fff',
    'primaryBorderColor': '#7C0000',
    'lineColor': '#F95959',
    'secondaryColor': '#006D77',
    'tertiaryColor': '#FFB400'
  }
}}%%

classDiagram
    %% Core Interfaces
    class NetworkDeviceConnection {
        <<interface>>
        +connect()
        +close()
        +check_config_mode()
        +enter_config_mode()
        +exit_config_mode()
        +session_preparation()
        +terminal_settings()
        +set_terminal_width()
        +disable_paging()
        +set_base_prompt()
        +save_configuration()
        +send_command()
        +get_device_type()
    }

    %% Core Classes with custom styling
    class BaseConnection:::coreClass {
        +session: Option~Session~
        +channel: SSHChannel
        +base_prompt: Option~String~
        +session_log: SessionLog
        +config: NetsshConfig
        +connect()
        +open_channel()
        +write_channel()
        +read_channel()
        +read_until_pattern()
    }

    class SSHChannel:::transportClass {
        -remote_conn: RefCell~Option~SSH2Channel~~
        -encoding: String
        -base_prompt: Option~String~
        -prompt_regex: Option~Regex~
        +write_channel()
        +read_buffer()
        +read_channel()
        +read_until_prompt()
    }

    %% Device Related Classes
    class DeviceFactory:::serviceClass {
        <<service>>
        +create_device()
    }

    %% Cisco Classes
    class CiscoBaseConnection:::vendorClass {
        +connection: BaseConnection
        +config: CiscoDeviceConfig
        +prompt: String
        +in_enable_mode: bool
        +in_config_mode: bool
        +enable()
        +check_enable_mode()
    }

    class CiscoIosDevice:::vendorClass {
        +base: CiscoBaseConnection
        +send_command()
        +save_config()
    }

    class CiscoXrDevice:::vendorClass {
        +base: CiscoBaseConnection
        +send_command()
        +commit()
    }

    class CiscoNxosDevice:::vendorClass {
        +base: CiscoBaseConnection
        +send_command()
        +save_config()
    }

    class CiscoAsaDevice:::vendorClass {
        +base: CiscoBaseConnection
        +send_command()
        +write_mem()
    }

    %% Juniper Classes
    class JuniperBaseConnection:::vendorClass {
        +connection: BaseConnection
        +config: JuniperDeviceConfig
        +prompt: String
        +enter_shell()
        +exit_shell()
    }

    class JuniperJunosDevice:::vendorClass {
        +base: JuniperBaseConnection
        +send_command()
        +commit()
    }

    %% Configuration Classes
    class DeviceConfig:::dataClass {
        +device_type: String
        +host: String
        +username: String
        +password: Option~String~
    }

    class CiscoDeviceConfig:::dataClass {
        +secret: Option~String~
    }

    class JuniperDeviceConfig:::dataClass {
        +ssh_key: Option~String~
    }



    %% Relationships
    BaseConnection o-- SSHChannel
    CiscoBaseConnection o-- BaseConnection
    JuniperBaseConnection o-- BaseConnection
    
    CiscoBaseConnection ..|> NetworkDeviceConnection
    JuniperBaseConnection ..|> NetworkDeviceConnection
    
    CiscoIosDevice --|> CiscoBaseConnection
    CiscoXrDevice --|> CiscoBaseConnection
    CiscoNxosDevice --|> CiscoBaseConnection
    CiscoAsaDevice --|> CiscoBaseConnection
    
    JuniperJunosDevice --|> JuniperBaseConnection
    
    DeviceFactory ..> NetworkDeviceConnection : creates
    DeviceFactory ..> CiscoBaseConnection : creates
    DeviceFactory ..> JuniperBaseConnection : creates
    
    CiscoDeviceConfig --|> DeviceConfig
    JuniperDeviceConfig --|> DeviceConfig

    %% Custom CSS classes
    classDef coreClass fill:#2D4059,stroke:#FFB400,color:#fff
    classDef transportClass fill:#006D77,stroke:#83C5BE,color:#fff
    classDef serviceClass fill:#E84545,stroke:#903749,color:#fff
    classDef vendorClass fill:#FFB400,stroke:#FF9000,color:#000
    classDef dataClass fill:#98C1D9,stroke:#3D5A80,color:#000
```

## Component Colors Legend

- 🔵 **Core Classes** (Dark Blue) - Core framework components
- 🌊 **Transport Classes** (Teal) - Network transport layer components
- 🔴 **Service Classes** (Red) - High-level service components
- 🟡 **Vendor Classes** (Yellow) - Vendor-specific implementations
- 🔷 **Data Classes** (Light Blue) - Data structures and models

## Component Responsibilities

1. **NetworkDeviceConnection**: Core interface for network device interactions
2. **BaseConnection**: Low-level SSH connection and communication
3. **SSHChannel**: SSH channel management and I/O operations
4. **DeviceFactory**: Device connection factory
5. **CiscoBaseConnection**: Base implementation for Cisco devices
6. **JuniperBaseConnection**: Base implementation for Juniper devices
7. **Vendor-Specific Devices**: 
   - CiscoIosDevice: Cisco IOS implementation
   - CiscoXrDevice: Cisco XR implementation
   - CiscoNxosDevice: Cisco NXOS implementation
   - CiscoAsaDevice: Cisco ASA implementation
   - JuniperJunosDevice: Juniper JUNOS implementation
8. **Configuration Classes**:
   - DeviceConfig: Base configuration
   - CiscoDeviceConfig: Cisco-specific configuration
   - JuniperDeviceConfig: Juniper-specific configuration
