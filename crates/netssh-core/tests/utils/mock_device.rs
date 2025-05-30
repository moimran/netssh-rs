use netssh_core::{
    device_connection::{DeviceInfo, NetworkDeviceConnection},
    error::NetsshError,
};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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
    /// Active client connections
    client_threads: Arc<Mutex<Vec<thread::JoinHandle<()>>>>,
}

/// Prompt style for the mock device
#[derive(Clone, Debug)]
#[allow(dead_code)] // Some variants are used in tests but not in this specific test run
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

#[allow(dead_code)] // Mock device methods are used in integration tests
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
            auth_credentials: Arc::new(Mutex::new(vec![(
                "admin".to_string(),
                "password".to_string(),
            )])),
            hostname: Arc::new(Mutex::new("MockDevice".to_string())),
            prompt_style: Arc::new(Mutex::new(PromptStyle::CiscoIOS)),
            client_threads: Arc::new(Mutex::new(Vec::new())),
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
        self.auth_credentials
            .lock()
            .unwrap()
            .push((username.to_string(), password.to_string()));
        self
    }

    /// Add a command response mapping
    pub fn add_command_response(&mut self, command: &str, response: &str) -> &mut Self {
        self.command_responses
            .lock()
            .unwrap()
            .insert(command.to_string(), response.to_string());
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
        let client_threads = self.client_threads.clone();

        // Start the server in a separate thread
        self.server_thread = Some(thread::spawn(move || {
            println!("Mock device server started on {}", addr);

            // Set a timeout on the listener
            listener.set_nonblocking(false).unwrap();
            listener.set_ttl(60).unwrap();

            let start_time = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(30); // 30 second timeout

            while !*stop_signal.lock().unwrap() {
                // Check if we've exceeded the timeout
                if start_time.elapsed() > timeout {
                    println!("Server thread timeout exceeded");
                    break;
                }

                // Accept new connections with a timeout
                match listener.accept() {
                    Ok((socket, addr)) => {
                        println!("New connection from {}", addr);

                        // Clone shared state for the client handler
                        let cmd_responses = command_responses.clone();
                        let auth_creds = auth_credentials.clone();
                        let host = hostname.clone();
                        let prompt = prompt_style.clone();
                        let dev_type = device_type.clone();
                        let stop = stop_signal.clone();

                        // Handle each client in a separate thread
                        let handle = thread::spawn(move || {
                            if let Err(e) = Self::handle_client(
                                socket,
                                cmd_responses,
                                auth_creds,
                                host,
                                prompt,
                                dev_type,
                                stop,
                            ) {
                                eprintln!("Error handling client: {}", e);
                            }
                        });

                        // Store the client thread handle
                        client_threads.lock().unwrap().push(handle);
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

        // Wait for all client threads to finish
        let mut client_threads = self.client_threads.lock().unwrap();
        while let Some(thread) = client_threads.pop() {
            if let Err(e) = thread.join() {
                eprintln!("Failed to join client thread: {:?}", e);
            }
        }

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
        mut socket: TcpStream,
        command_responses: Arc<Mutex<HashMap<String, String>>>,
        _auth_credentials: Arc<Mutex<Vec<(String, String)>>>,
        hostname: Arc<Mutex<String>>,
        prompt_style: Arc<Mutex<PromptStyle>>,
        device_type: String,
        stop_signal: Arc<Mutex<bool>>,
    ) -> Result<(), String> {
        println!("Handling new client connection");

        // Set a timeout for the client socket
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| e.to_string())?;
        socket
            .set_write_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| e.to_string())?;

        println!("Socket timeouts set");

        // Send initial banner based on device type
        let banner = match device_type.as_str() {
            "cisco_ios" => "Welcome to Cisco IOS (Mock)\r\n".to_string(),
            "juniper_junos" => "Welcome to Juniper JunOS (Mock)\r\n".to_string(),
            "cisco_asa" => "Welcome to Cisco ASA (Mock)\r\n".to_string(),
            _ => format!("Welcome to Mock Device ({})\r\n", device_type),
        };

        socket
            .write_all(banner.as_bytes())
            .map_err(|e| format!("Failed to write banner: {}", e))?;

        println!("Banner sent");

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
        socket
            .write_all(format!("{}\r\n", prompt).as_bytes())
            .map_err(|e| format!("Failed to write prompt: {}", e))?;
        socket
            .flush()
            .map_err(|e| format!("Failed to flush socket: {}", e))?;

        println!("Initial prompt sent");

        // Main command loop
        let mut buffer = [0u8; 1024];
        let mut line_buffer = String::new();

        while !*stop_signal.lock().unwrap() {
            println!("Waiting for command...");
            match socket.read(&mut buffer) {
                Ok(n) if n == 0 => {
                    // Client closed the connection
                    println!("Client closed the connection");
                    break;
                }
                Ok(n) => {
                    // Convert bytes to string and append to line buffer
                    if let Ok(data) = String::from_utf8(buffer[..n].to_vec()) {
                        println!("Received data: {:?}", data);
                        line_buffer.push_str(&data);

                        // Process any complete lines
                        while let Some(pos) = line_buffer.find('\n') {
                            let line = line_buffer[..pos].trim().to_string();
                            line_buffer = line_buffer[pos + 1..].to_string();

                            // Skip empty lines
                            if line.is_empty() {
                                continue;
                            }

                            println!("Processing command: {}", line);

                            // Check for exit command
                            if line.to_lowercase() == "exit" || line.to_lowercase() == "quit" {
                                println!("Exit command received");
                                socket.write_all(b"Goodbye!\r\n").ok();
                                socket.flush().ok();
                                return Ok(());
                            }

                            // Look up the response
                            let response = command_responses
                                .lock()
                                .unwrap()
                                .get(&line)
                                .cloned()
                                .unwrap_or_else(|| {
                                    format!("Command '{}' not recognized\r\n", line)
                                });

                            println!("Sending response: {:?}", response);

                            // Send the response and prompt
                            socket
                                .write_all(response.as_bytes())
                                .map_err(|e| format!("Failed to write response: {}", e))?;
                            socket
                                .write_all(b"\r\n")
                                .map_err(|e| format!("Failed to write line ending: {}", e))?;
                            socket
                                .write_all(format!("{}\r\n", prompt).as_bytes())
                                .map_err(|e| format!("Failed to write prompt: {}", e))?;
                            socket
                                .flush()
                                .map_err(|e| format!("Failed to flush socket: {}", e))?;

                            println!("Response and prompt sent");
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::TimedOut {
                        // Timeout is expected, continue checking stop signal
                        println!("Read timeout, checking stop signal");
                        continue;
                    }
                    eprintln!("Error reading from socket: {}", e);
                    break;
                }
            }
        }

        println!("Exiting command loop");

        // Send goodbye message if stopping
        if *stop_signal.lock().unwrap() {
            println!("Stop signal received, sending goodbye");
            socket.write_all(b"Server shutting down\r\n").ok();
            socket.flush().ok();
        }

        Ok(())
    }

    pub fn with_credentials(
        _device_type: &str,
        _hostname: &str,
        _auth_credentials: Arc<Mutex<Vec<(String, String)>>>,
    ) -> Self {
        let port = Self::find_available_port();
        Self::with_port(port)
    }
}

impl Drop for MockNetworkDevice {
    fn drop(&mut self) {
        if self.server_thread.is_some() {
            let _ = self.stop();
        }
    }
}

#[async_trait::async_trait]
impl NetworkDeviceConnection for MockNetworkDevice {
    fn connect(&mut self) -> Result<(), NetsshError> {
        // Start the mock device server if not already running
        if self.server_thread.is_none() {
            self.start()
                .map_err(|e| NetsshError::ConnectionError(e.to_string()))?;
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), NetsshError> {
        // Stop the mock device server
        self.stop()
            .map_err(|e| NetsshError::DisconnectError(e.to_string()))?;
        Ok(())
    }

    fn send_command(
        &mut self,
        command: &str,
        _expect_string: Option<&str>,
        _read_timeout: Option<f64>,
        _strip_prompt: Option<bool>,
        _strip_command: Option<bool>,
        _normalize: Option<bool>,
        _use_textfsm: Option<bool>,
        _use_ttp: Option<bool>,
    ) -> Result<String, NetsshError> {
        // Get the response from the command_responses map
        let response = self
            .command_responses
            .lock()
            .unwrap()
            .get(command)
            .cloned()
            .unwrap_or_else(|| format!("Command '{}' not recognized", command));
        Ok(response)
    }

    fn send_config_commands(&mut self, commands: &[&str]) -> Result<Vec<String>, NetsshError> {
        let mut responses = Vec::new();
        for command in commands {
            responses.push(self.send_command(command, None, None, None, None, None, None, None)?);
        }
        Ok(responses)
    }

    fn enter_config_mode(&mut self, _config_command: Option<&str>) -> Result<(), NetsshError> {
        let _ = self.send_command("configure terminal", None, None, None, None, None, None, None)?;
        Ok(())
    }

    fn exit_config_mode(&mut self, _exit_config: Option<&str>) -> Result<(), NetsshError> {
        let _ = self.send_command("end", None, None, None, None, None, None, None)?;
        Ok(())
    }

    fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        Ok(true)
    }

    fn get_device_info(&mut self) -> Result<DeviceInfo, NetsshError> {
        Ok(DeviceInfo {
            device_type: self.device_type.clone(),
            hostname: self.hostname.lock().unwrap().clone(),
            version: "1.0.0".to_string(),
            model: "Mock".to_string(),
            serial: "MOCK123".to_string(),
            uptime: "0 days".to_string(),
        })
    }

    fn get_device_type(&self) -> &str {
        &self.device_type
    }

    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        Ok(())
    }

    fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        Ok(())
    }

    fn set_terminal_width(&mut self, _width: u32) -> Result<(), NetsshError> {
        Ok(())
    }

    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        Ok(())
    }

    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        Ok(self.hostname.lock().unwrap().clone())
    }

    fn save_configuration(&mut self) -> Result<(), NetsshError> {
        Ok(())
    }

    fn send_config_set(
        &mut self,
        config_commands: Vec<String>,
        _exit_config_mode: Option<bool>,
        _read_timeout: Option<f64>,
        _strip_prompt: Option<bool>,
        _strip_command: Option<bool>,
        _config_mode_command: Option<&str>,
        _cmd_verify: Option<bool>,
        _enter_config_mode: Option<bool>,
        _error_pattern: Option<&str>,
        _terminator: Option<&str>,
        _bypass_commands: Option<&str>,
        _fast_cli: Option<bool>,
    ) -> Result<String, NetsshError> {
        let mut output = String::new();
        for command in config_commands {
            let response = self.send_command(command).execute()?;
            output.push_str(&response);
            output.push('\n');
        }
        Ok(output)
    }
}

#[allow(dead_code)] // Used in integration tests
pub fn setup_mock_device() -> MockNetworkDevice {
    let mut device = MockNetworkDevice::new();

    // Ensure the device is stopped
    device.stop().ok();

    // Set up command responses
    let mut responses = HashMap::new();
    responses.insert(
        "show version".to_string(),
        "Cisco IOS Software\nUptime: 10 days\n".to_string(),
    );
    responses.insert(
        "show interfaces".to_string(),
        "GigabitEthernet0/0\n192.168.1.1/24\n".to_string(),
    );
    device.add_command_response("show version", "Cisco IOS Software\nUptime: 10 days\n");
    device.add_command_response("show interfaces", "GigabitEthernet0/0\n192.168.1.1/24\n");

    // Set up authentication credentials
    let mut credentials = Vec::new();
    credentials.push(("test".to_string(), "test".to_string()));
    device.add_auth_credentials("test", "test");

    // Set up device info
    device.set_hostname("Router1");
    device.set_prompt_style(PromptStyle::CiscoIOS);

    device
}
