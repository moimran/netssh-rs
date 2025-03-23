use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{Read, Write};
use std::time::Duration;
use ssh2::{Session, Channel};
use rand::{Rng, thread_rng};

pub struct MockNetworkDevice {
    /// Port the mock device will listen on
    port: u16,
    /// Server thread handle
    server_thread: Option<thread::JoinHandle<()>>,
    /// Flag to signal the server to stop
    stop_signal: Arc<Mutex<bool>>,
    /// Device type (cisco_ios, juniper_junos, etc.)
    device_type: String,
    /// Command responses
    command_responses: Arc<Mutex<HashMap<String, String>>>,
    /// Auth credentials
    auth_credentials: Arc<Mutex<Vec<(String, String)>>>,
    /// Hostname of the device
    hostname: Arc<Mutex<String>>,
    /// Prompt style
    prompt_style: Arc<Mutex<PromptStyle>>,
}

/// Prompt style for the mock device
#[derive(Clone, Debug)]
pub enum PromptStyle {
    /// Cisco IOS style: hostname>
    CiscoIOS,
    /// Juniper Junos style: user@hostname>
    JuniperJunos,
    /// Cisco ASA style: hostname#
    CiscoASA,
    /// Custom style with format string
    Custom(String),
}

impl MockNetworkDevice {
    /// Create a new mock device on a random available port
    pub fn new() -> Self {
        let port = Self::find_available_port();
        Self::with_port(port)
    }
    
    /// Create a new mock device on a specific port
    pub fn with_port(port: u16) -> Self {
        MockNetworkDevice {
            port,
            server_thread: None,
            stop_signal: Arc::new(Mutex::new(false)),
            device_type: "cisco_ios".to_string(),
            command_responses: Arc::new(Mutex::new(HashMap::new())),
            auth_credentials: Arc::new(Mutex::new(vec![("admin".to_string(), "password".to_string())])),
            hostname: Arc::new(Mutex::new("MockDevice".to_string())),
            prompt_style: Arc::new(Mutex::new(PromptStyle::CiscoIOS)),
        }
    }
    
    /// Find an available TCP port
    fn find_available_port() -> u16 {
        let mut attempts = 0;
        while attempts < 10 {
            // Try a random port in the ephemeral range
            let port = thread_rng().gen_range(49152..65535);
            
            // Try to bind to it
            if TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok() {
                return port;
            }
            
            attempts += 1;
        }
        
        // Default fallback
        33000
    }
    
    /// Set the device type
    pub fn set_device_type(&mut self, device_type: &str) -> &mut Self {
        self.device_type = device_type.to_string();
        self
    }
    
    /// Set the hostname
    pub fn set_hostname(&mut self, hostname: &str) -> &mut Self {
        *self.hostname.lock().unwrap() = hostname.to_string();
        self
    }
    
    /// Set the prompt style
    pub fn set_prompt_style(&mut self, style: PromptStyle) -> &mut Self {
        *self.prompt_style.lock().unwrap() = style;
        self
    }
    
    /// Add auth credentials (username, password)
    pub fn add_auth_credentials(&mut self, username: &str, password: &str) -> &mut Self {
        self.auth_credentials.lock().unwrap().push((username.to_string(), password.to_string()));
        self
    }
    
    /// Add a command response mapping
    pub fn add_command_response(&mut self, command: &str, response: &str) -> &mut Self {
        self.command_responses.lock().unwrap().insert(command.to_string(), response.to_string());
        self
    }
    
    /// Get the port the mock device is listening on
    pub fn port(&self) -> u16 {
        self.port
    }
    
    /// Start the mock device server
    pub fn start(&mut self) -> Result<(), String> {
        if self.server_thread.is_some() {
            return Err("Server already running".to_string());
        }
        
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = match TcpListener::bind(&addr) {
            Ok(l) => l,
            Err(e) => return Err(format!("Failed to bind to {}: {}", addr, e)),
        };
        
        // Clone the shared state for the server thread
        let stop_signal = self.stop_signal.clone();
        let command_responses = self.command_responses.clone();
        let auth_credentials = self.auth_credentials.clone();
        let hostname = self.hostname.clone();
        let prompt_style = self.prompt_style.clone();
        let device_type = self.device_type.clone();
        
        // Start the server in a separate thread
        self.server_thread = Some(thread::spawn(move || {
            println!("Mock device server started on {}", addr);
            
            // Set a timeout on the listener to periodically check the stop signal
            listener.set_nonblocking(true).unwrap();
            
            while !*stop_signal.lock().unwrap() {
                match listener.accept() {
                    Ok((socket, addr)) => {
                        println!("New connection from {}", addr);
                        
                        // Clone shared state for the client handler
                        let cmd_responses = command_responses.clone();
                        let auth_creds = auth_credentials.clone();
                        let host = hostname.clone();
                        let prompt = prompt_style.clone();
                        let dev_type = device_type.clone();
                        
                        // Handle each client in a separate thread
                        thread::spawn(move || {
                            if let Err(e) = Self::handle_client(socket, cmd_responses, auth_creds, host, prompt, dev_type) {
                                eprintln!("Error handling client: {}", e);
                            }
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No new connections, sleep briefly and check for stop signal
                        thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                        break;
                    }
                }
            }
            
            println!("Mock device server stopped");
        }));
        
        // Give the server a moment to start
        thread::sleep(Duration::from_millis(100));
        
        Ok(())
    }
    
    /// Stop the mock device server
    pub fn stop(&mut self) -> Result<(), String> {
        if self.server_thread.is_none() {
            return Err("Server not running".to_string());
        }
        
        // Signal the server to stop
        *self.stop_signal.lock().unwrap() = true;
        
        // Wait for the server thread to finish
        if let Some(thread) = self.server_thread.take() {
            if let Err(e) = thread.join() {
                return Err(format!("Failed to join server thread: {:?}", e));
            }
        }
        
        // Reset the stop signal for next time
        *self.stop_signal.lock().unwrap() = false;
        
        Ok(())
    }
    
    /// Handle a client connection
    fn handle_client(
        socket: TcpStream,
        command_responses: Arc<Mutex<HashMap<String, String>>>,
        auth_credentials: Arc<Mutex<Vec<(String, String)>>>,
        hostname: Arc<Mutex<String>>,
        prompt_style: Arc<Mutex<PromptStyle>>,
        device_type: String,
    ) -> Result<(), String> {
        println!("Handling new client connection");
        
        // Create a new SSH session for this client
        let mut session = Session::new().map_err(|e| format!("Failed to create SSH session: {}", e))?;
        session.set_tcp_stream(socket);
        
        // Complete the SSH handshake
        session.handshake().map_err(|e| format!("SSH handshake failed: {}", e))?;
        
        // Set up server-side authentication with the mock credentials
        session.userauth_publickey_memory("mock_server", include_str!("public_key.pem"), include_str!("private_key.pem"), None)
            .map_err(|e| format!("Failed to set up server authentication: {}", e))?;
        
        println!("SSH handshake completed, waiting for auth");
        
        // Handle authentication attempts
        let credentials = auth_credentials.lock().unwrap().clone();
        session.auth_methods("").map_err(|e| format!("Failed to get auth methods: {}", e))?;
        
        // Wait for a channel
        let mut channel = session.accept().map_err(|e| format!("Failed to accept connection: {}", e))?;
        
        println!("Channel established, sending banner");
        
        // Send initial banner based on device type
        let banner = match device_type.as_str() {
            "cisco_ios" => "Welcome to Cisco IOS (Mock)\r\n".to_string(),
            "juniper_junos" => "Welcome to Juniper JunOS (Mock)\r\n".to_string(),
            "cisco_asa" => "Welcome to Cisco ASA (Mock)\r\n".to_string(),
            _ => format!("Welcome to Mock Device ({})\r\n", device_type),
        };
        
        channel.write_all(banner.as_bytes()).map_err(|e| format!("Failed to write banner: {}", e))?;
        
        // Get the hostname and prompt style
        let hostname = hostname.lock().unwrap().clone();
        let prompt_style = prompt_style.lock().unwrap().clone();
        
        // Generate the prompt string
        let prompt = match prompt_style {
            PromptStyle::CiscoIOS => format!("{}>", hostname),
            PromptStyle::JuniperJunos => format!("admin@{}> ", hostname),
            PromptStyle::CiscoASA => format!("{}# ", hostname),
            PromptStyle::Custom(fmt) => fmt.replace("{hostname}", &hostname),
        };
        
        // Write the initial prompt
        channel.write_all(prompt.as_bytes()).map_err(|e| format!("Failed to write prompt: {}", e))?;
        channel.flush().map_err(|e| format!("Failed to flush channel: {}", e))?;
        
        // Main command loop
        let mut command_buffer = Vec::new();
        let mut buffer = [0u8; 1024];
        
        loop {
            match channel.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    // Append to command buffer
                    command_buffer.extend_from_slice(&buffer[..n]);
                    
                    // Check if we have a complete command (ends with newline)
                    if command_buffer.contains(&b'\n') || command_buffer.contains(&b'\r') {
                        // Convert to string
                        let command_str = String::from_utf8_lossy(&command_buffer).trim().to_string();
                        command_buffer.clear();
                        
                        println!("Received command: {}", command_str);
                        
                        // Check for exit command
                        if command_str.to_lowercase() == "exit" || command_str.to_lowercase() == "quit" {
                            channel.write_all(b"Goodbye!\r\n").ok();
                            channel.flush().ok();
                            break;
                        }
                        
                        // Look up the response
                        let response = command_responses.lock().unwrap()
                            .get(&command_str)
                            .cloned()
                            .unwrap_or_else(|| format!("Command '{}' not recognized\r\n", command_str));
                        
                        // Send the response and prompt
                        channel.write_all(response.as_bytes()).map_err(|e| format!("Failed to write response: {}", e))?;
                        channel.write_all(b"\r\n").map_err(|e| format!("Failed to write line ending: {}", e))?;
                        channel.write_all(prompt.as_bytes()).map_err(|e| format!("Failed to write prompt: {}", e))?;
                        channel.flush().map_err(|e| format!("Failed to flush channel: {}", e))?;
                    }
                }
                Ok(0) => {
                    // Client closed the connection
                    println!("Client closed the connection");
                    break;
                }
                Err(e) => {
                    eprintln!("Error reading from channel: {}", e);
                    break;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}

impl Drop for MockNetworkDevice {
    fn drop(&mut self) {
        if self.server_thread.is_some() {
            let _ = self.stop();
        }
    }
} 