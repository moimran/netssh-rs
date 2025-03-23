use crate::api::models::{
    CommandRequest, ConfigCommandRequest, InterfaceConfigRequest, JsonResponse, DeviceDetails
};
use crate::device_connection::DeviceConfig;
use crate::device_factory::DeviceFactory;
use crate::error::NetsshError;
use actix_web::{web, HttpResponse, Responder};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info};

/// Convert DeviceDetails to DeviceConfig
fn device_details_to_config(details: &DeviceDetails) -> DeviceConfig {
    DeviceConfig {
        device_type: details.device_type.clone(),
        host: details.host.clone(),
        username: details.username.clone(),
        password: details.password.clone(),
        port: details.port,
        timeout: details.timeout.map(Duration::from_secs),
        secret: details.secret.clone(),
        session_log: details.session_log.clone(),
    }
}

/// Execute a show command on a device
pub async fn execute_show_command(
    request: web::Json<CommandRequest>,
) -> impl Responder {
    let device_config = device_details_to_config(&request.device);
    
    // Create device
    let result = execute_command(&device_config, &request.command);
    
    match result {
        Ok((output, execution_time)) => {
            // Create metadata
            let mut metadata = HashMap::new();
            metadata.insert(
                "execution_time".to_string(),
                json!(format!("{:.2}s", execution_time.as_secs_f64())),
            );
            metadata.insert("device_type".to_string(), json!(device_config.device_type));

            // Parse output based on command type
            let data = if request.command.contains("interface") {
                // Interface command
                json!({
                    "raw_output": output,
                    "parsed": {
                        "interfaces": []  // Would need actual parsing
                    }
                })
            } else if request.command.contains("version") {
                // Version command
                json!({
                    "raw_output": output,
                    "parsed": {
                        "version": "Unknown"  // Would need actual parsing
                    }
                })
            } else {
                // Generic command
                json!({
                    "raw_output": output
                })
            };

            HttpResponse::Ok().json(JsonResponse::success(data).with_metadata(metadata))
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(JsonResponse::error(&e.to_string()))
        }
    }
}

/// Execute configuration commands on a device
pub async fn execute_config_commands(
    request: web::Json<ConfigCommandRequest>,
) -> impl Responder {
    let device_config = device_details_to_config(&request.device);
    
    // Create device
    let result = execute_config(&device_config, &request.commands);
    
    match result {
        Ok((results, execution_time)) => {
            // Create metadata
            let mut metadata = HashMap::new();
            metadata.insert(
                "execution_time".to_string(),
                json!(format!("{:.2}s", execution_time.as_secs_f64())),
            );
            metadata.insert("device_type".to_string(), json!(device_config.device_type));

            HttpResponse::Ok().json(JsonResponse::success(json!({
                "commands": results
            })).with_metadata(metadata))
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(JsonResponse::error(&e.to_string()))
        }
    }
}

/// Configure an interface on a device
pub async fn configure_interface(
    request: web::Json<InterfaceConfigRequest>,
) -> impl Responder {
    let device_config = device_details_to_config(&request.device);
    
    // Create commands for interface configuration
    let mut commands = Vec::new();
    commands.push(format!("interface {}", request.name));
    
    if let Some(description) = &request.description {
        commands.push(format!("description {}", description));
    }
    
    if let (Some(ip), Some(mask)) = (&request.ip_address, &request.subnet_mask) {
        commands.push(format!("ip address {} {}", ip, mask));
    }
    
    if let Some(status) = &request.admin_status {
        if status == "up" {
            commands.push("no shutdown".to_string());
        } else if status == "down" {
            commands.push("shutdown".to_string());
        }
    }
    
    // Execute the commands
    let result = execute_config(&device_config, &commands);
    
    match result {
        Ok((_, execution_time)) => {
            // Create metadata
            let mut metadata = HashMap::new();
            metadata.insert(
                "execution_time".to_string(),
                json!(format!("{:.2}s", execution_time.as_secs_f64())),
            );

            HttpResponse::Ok().json(JsonResponse::success(json!({
                "interface": request.name,
                "message": "Interface configured successfully"
            })).with_metadata(metadata))
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(JsonResponse::error(&e.to_string()))
        }
    }
}

/// Execute a single command on a device
fn execute_command(config: &DeviceConfig, command: &str) -> Result<(String, Duration), NetsshError> {
    // Create device
    let mut device = DeviceFactory::create_device(config)?;
    
    // Connect to device
    device.connect()?;
    
    // Execute command
    let start_time = std::time::Instant::now();
    let output = device.send_command(command)?;
    let execution_time = start_time.elapsed();

    // Close connection
    let _ = device.close();
    
    Ok((output, execution_time))
}

/// Execute configuration commands on a device
fn execute_config(config: &DeviceConfig, commands: &[String]) -> Result<(Vec<Value>, Duration), NetsshError> {
    // Create device
    let mut device = DeviceFactory::create_device(config)?;
    
    // Connect to device
    device.connect()?;
    
    // Execute commands
    let start_time = std::time::Instant::now();
    
    // Enter config mode
    device.enter_config_mode(None)?;

    // Execute each command
    let mut results = Vec::new();
    for command in commands {
        match device.send_command(command) {
            Ok(_) => results.push(json!({
                "command": command,
                "status": "success"
            })),
            Err(e) => results.push(json!({
                "command": command,
                "status": "error",
                "error": e.to_string()
            })),
        }
    }

    // Exit config mode
    device.exit_config_mode(None)?;
    
    let execution_time = start_time.elapsed();

    // Close connection
    let _ = device.close();
    
    Ok((results, execution_time))
}