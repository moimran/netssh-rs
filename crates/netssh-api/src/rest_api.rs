use crate::device_connection::{DeviceConfig, DeviceInfo};
use crate::device_factory::DeviceFactory;
use crate::device_service::{DeviceService, Interface};
use crate::error::NetsshError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Error type for API operations
#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    InternalError(String),
    Unauthorized(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ApiError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<NetsshError> for ApiError {
    fn from(err: NetsshError) -> Self {
        ApiError::InternalError(err.to_string())
    }
}

/// Repository for storing device configurations
pub struct ConfigRepository {
    configs: Mutex<HashMap<String, DeviceConfig>>,
}

impl ConfigRepository {
    pub fn new() -> Self {
        Self {
            configs: Mutex::new(HashMap::new()),
        }
    }
    
    pub fn add_device(&self, device_id: String, config: DeviceConfig) {
        let mut configs = self.configs.lock().unwrap();
        configs.insert(device_id, config);
    }
    
    pub fn get_device(&self, device_id: &str) -> Option<DeviceConfig> {
        let configs = self.configs.lock().unwrap();
        configs.get(device_id).cloned()
    }
    
    pub fn remove_device(&self, device_id: &str) -> bool {
        let mut configs = self.configs.lock().unwrap();
        configs.remove(device_id).is_some()
    }
    
    pub fn list_devices(&self) -> Vec<String> {
        let configs = self.configs.lock().unwrap();
        configs.keys().cloned().collect()
    }
}

/// Controller for device operations
pub struct DeviceController {
    config_repo: Arc<ConfigRepository>,
}

impl DeviceController {
    pub fn new(config_repo: Arc<ConfigRepository>) -> Self {
        Self { config_repo }
    }
    
    /// Register a new device
    pub fn register_device(&self, device_id: String, config: DeviceConfig) {
        self.config_repo.add_device(device_id, config);
    }
    
    /// List all registered devices
    pub fn list_devices(&self) -> Vec<String> {
        self.config_repo.list_devices()
    }
    
    /// Get device information
    pub fn get_device_info(&self, device_id: &str) -> Result<DeviceInfo, ApiError> {
        // Get device config
        let config = self.config_repo.get_device(device_id)
            .ok_or_else(|| ApiError::NotFound(format!("Device not found: {}", device_id)))?;
        
        // Create device
        let device = DeviceFactory::create_device(&config)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        // Create service
        let mut service = DeviceService::new(device);
        
        // Connect to device
        service.connect()
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        // Get device info
        let info = service.get_device_info()
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        // Close connection
        let _ = service.close();
        
        Ok(info)
    }
    
    /// Get interfaces from a device
    pub fn get_interfaces(&self, device_id: &str) -> Result<Vec<Interface>, ApiError> {
        // Get device config
        let config = self.config_repo.get_device(device_id)
            .ok_or_else(|| ApiError::NotFound(format!("Device not found: {}", device_id)))?;
        
        // Create device
        let device = DeviceFactory::create_device(&config)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        // Create service
        let mut service = DeviceService::new(device);
        
        // Connect to device
        service.connect()
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        // Get interfaces
        let interfaces = service.get_interfaces()
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        // Close connection
        let _ = service.close();
        
        Ok(interfaces)
    }
    
    /// Configure an interface on a device
    pub fn configure_interface(&self, device_id: &str, interface_name: &str, description: &str) -> Result<(), ApiError> {
        // Get device config
        let config = self.config_repo.get_device(device_id)
            .ok_or_else(|| ApiError::NotFound(format!("Device not found: {}", device_id)))?;
        
        // Create device
        let device = DeviceFactory::create_device(&config)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        // Create service
        let mut service = DeviceService::new(device);
        
        // Connect to device
        service.connect()
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        // Configure interface
        service.configure_interface(interface_name, description)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        // Close connection
        let _ = service.close();
        
        Ok(())
    }
    
    /// Execute a command on a device
    pub fn execute_command(&self, device_id: &str, command: &str) -> Result<String, ApiError> {
        // Get device config
        let config = self.config_repo.get_device(device_id)
            .ok_or_else(|| ApiError::NotFound(format!("Device not found: {}", device_id)))?;
        
        // Create device
        let device = DeviceFactory::create_device(&config)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        // Create service
        let mut service = DeviceService::new(device);
        
        // Connect to device
        service.connect()
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        // Execute command
        let output = service.execute_command(command)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        // Close connection
        let _ = service.close();
        
        Ok(output)
    }
}

// Example of how to use with a web framework like Actix
/*
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn list_devices(controller: web::Data<DeviceController>) -> impl Responder {
    let devices = controller.list_devices();
    HttpResponse::Ok().json(devices)
}

async fn get_device_info(
    controller: web::Data<DeviceController>,
    path: web::Path<String>,
) -> impl Responder {
    let device_id = path.into_inner();
    match controller.get_device_info(&device_id) {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(ApiError::NotFound(msg)) => HttpResponse::NotFound().body(msg),
        Err(ApiError::InternalError(msg)) => HttpResponse::InternalServerError().body(msg),
        Err(ApiError::BadRequest(msg)) => HttpResponse::BadRequest().body(msg),
        Err(ApiError::Unauthorized(msg)) => HttpResponse::Unauthorized().body(msg),
    }
}

async fn get_interfaces(
    controller: web::Data<DeviceController>,
    path: web::Path<String>,
) -> impl Responder {
    let device_id = path.into_inner();
    match controller.get_interfaces(&device_id) {
        Ok(interfaces) => HttpResponse::Ok().json(interfaces),
        Err(ApiError::NotFound(msg)) => HttpResponse::NotFound().body(msg),
        Err(ApiError::InternalError(msg)) => HttpResponse::InternalServerError().body(msg),
        Err(ApiError::BadRequest(msg)) => HttpResponse::BadRequest().body(msg),
        Err(ApiError::Unauthorized(msg)) => HttpResponse::Unauthorized().body(msg),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config_repo = Arc::new(ConfigRepository::new());
    let controller = DeviceController::new(config_repo.clone());
    
    // Register some devices
    controller.register_device(
        "router1".to_string(),
        DeviceConfig {
            device_type: "cisco_ios".to_string(),
            host: "192.168.1.1".to_string(),
            username: "admin".to_string(),
            password: Some("cisco123".to_string()),
            port: Some(22),
            timeout: Some(std::time::Duration::from_secs(60)),
            secret: Some("enable_secret".to_string()),
            session_log: Some("logs/router1.log".to_string()),
        },
    );
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(controller.clone()))
            .route("/devices", web::get().to(list_devices))
            .route("/devices/{id}", web::get().to(get_device_info))
            .route("/devices/{id}/interfaces", web::get().to(get_interfaces))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
*/
