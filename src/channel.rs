use crate::error::NetsshError;
use log::debug;
use regex::Regex;
use ssh2::Channel as SSH2Channel;
use std::cell::RefCell;
use std::io::{self, Read, Write};
use std::time::Duration;

const MAX_BUFFER: usize = 65536; // 64KB, same as in Python's Netmiko

pub struct SSHChannel {
    remote_conn: RefCell<Option<SSH2Channel>>,
    encoding: String,
    base_prompt: Option<String>,
    prompt_regex: Option<Regex>,
}

// Implement Clone for SSHChannel
impl Clone for SSHChannel {
    fn clone(&self) -> Self {
        // We can't clone the SSH2Channel, so we create a new one with None
        Self {
            remote_conn: RefCell::new(None),
            encoding: self.encoding.clone(),
            base_prompt: self.base_prompt.clone(),
            prompt_regex: self.prompt_regex.clone(),
        }
    }
}

impl SSHChannel {
    pub fn new(conn: Option<SSH2Channel>) -> Self {
        Self {
            remote_conn: RefCell::new(conn),
            encoding: String::from("utf-8"),
            base_prompt: None,
            prompt_regex: None,
        }
    }
    
    pub fn set_base_prompt(&mut self, prompt: &str) {
        debug!(target: "SSHChannel::set_base_prompt", "Setting base prompt to: {}", prompt);
        self.base_prompt = Some(prompt.to_string());
        
        // Create a regex that matches the prompt followed by > or #
        let prompt_pattern = format!(r"{}[>#]", regex::escape(prompt));
        match Regex::new(&prompt_pattern) {
            Ok(re) => {
                debug!(target: "SSHChannel::set_base_prompt", "Created prompt regex: {}", prompt_pattern);
                self.prompt_regex = Some(re);
            },
            Err(e) => {
                debug!(target: "SSHChannel::set_base_prompt", "Failed to create prompt regex: {}", e);
            }
        }
    }
    
    pub fn get_base_prompt(&self) -> Option<&str> {
        self.base_prompt.as_deref()
    }

    pub fn write_channel(&self, out_data: &str) -> Result<(), NetsshError> {
        debug!(target: "SSHChannel::write_channel", "Writing to channel: {:?}", out_data);
        
        let mut remote_conn = self.remote_conn.borrow_mut();
        let channel = remote_conn.as_mut()
            .ok_or_else(|| NetsshError::WriteError("Attempt to write data, but there is no active channel.".to_string()))?;

        // Convert string to bytes and write to channel
        let bytes = out_data.as_bytes();
        channel.write_all(bytes)
            .map_err(|e| NetsshError::WriteError(format!("Failed to write to channel: {}", e)))?;

        // Flush the channel to ensure all data is sent
        channel.flush()
            .map_err(|e| NetsshError::WriteError(format!("Failed to flush channel: {}", e)))?;

        debug!(target: "SSHChannel::write_channel", "Successfully wrote to channel");
        Ok(())
    }

    pub fn read_buffer(&self, prompt_regex: Option<&Regex>) -> Result<String, NetsshError> {
        debug!(target: "SSHChannel::read_buffer", "Reading buffer from channel");
        
        let mut remote_conn = self.remote_conn.borrow_mut();
        let channel = remote_conn.as_mut()
            .ok_or_else(|| NetsshError::ReadError("Attempt to read, but there is no active channel.".to_string()))?;

        let mut buffer = vec![0; MAX_BUFFER];
        let mut output = String::new();
        
        // Check if data is available (similar to recv_ready in Python)
        debug!(target: "SSHChannel::read_buffer", "Checking if data is available to read");
        match channel.read(&mut buffer) {
            Ok(n) if n > 0 => {
                debug!(target: "SSHChannel::read_buffer", "Read {} bytes from channel", n);
                
                // Convert bytes to string using the specified encoding
                let chunk = String::from_utf8_lossy(&buffer[..n]).to_string();
                output.push_str(&chunk);
                
                // Check if we found the prompt
                if let Some(re) = prompt_regex {
                    if re.is_match(&output) {
                        debug!(target: "SSHChannel::read_buffer", "Found prompt in output");
                    }
                } else if let Some(ref re) = self.prompt_regex {
                    if re.is_match(&output) {
                        debug!(target: "SSHChannel::read_buffer", "Found prompt in output using default prompt regex");
                    }
                }
            },
            Ok(0) => {
                debug!(target: "SSHChannel::read_buffer", "Channel stream closed by remote device");
                return Err(NetsshError::ReadError("Channel stream closed by remote device.".to_string()));
            },
            Ok(_) => {
                debug!(target: "SSHChannel::read_buffer", "No data available to read");
            },
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                debug!(target: "SSHChannel::read_buffer", "Would block, no data available");
            },
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                debug!(target: "SSHChannel::read_buffer", "Timed out, no data available");
            },
            Err(e) => {
                debug!(target: "SSHChannel::read_buffer", "Error reading from channel: {}", e);
                return Err(NetsshError::IoError(e));
            }
        }

        debug!(target: "SSHChannel::read_buffer", "Read buffer result: {:?}", output);
        Ok(output)
    }

    pub fn read_channel(&self) -> Result<String, NetsshError> {
        debug!(target: "SSHChannel::read_channel", "Reading all available data from channel");
        
        // Check if we have a channel
        if self.is_none() {
            return Err(NetsshError::ReadError("Attempt to read, but there is no active channel.".to_string()));
        }

        let mut output = String::new();
        let prompt_regex = self.prompt_regex.as_ref();
        
        // Keep reading until no more data is available or we find the prompt
        loop {
            let new_output = self.read_buffer(prompt_regex)?;
            if new_output.is_empty() {
                break;
            }
            output.push_str(&new_output);
            
            // Check if we found the prompt
            if let Some(re) = prompt_regex {
                if re.is_match(&output) {
                    debug!(target: "SSHChannel::read_channel", "Found prompt, breaking read loop");
                    break;
                }
            }
        }

        debug!(target: "SSHChannel::read_channel", "Read channel result: {:?}", output);
        Ok(output)
    }
    
    pub fn read_until_prompt(&self, timeout: Option<Duration>, custom_prompt: Option<&Regex>) -> Result<String, NetsshError> {
        debug!(target: "SSHChannel::read_until_prompt", "Reading until prompt");
        
        // Check if we have a channel
        if self.is_none() {
            return Err(NetsshError::ReadError("Attempt to read, but there is no active channel.".to_string()));
        }
        
        // Check if we have a prompt regex
        let prompt_regex = if let Some(re) = custom_prompt {
            re
        } else {
            match &self.prompt_regex {
                Some(re) => re,
                None => return Err(NetsshError::ReadError("No prompt regex set".to_string())),
            }
        };

        let mut output = String::new();
        let start_time = std::time::Instant::now();
        let timeout = timeout.unwrap_or(Duration::from_secs(10));
        
        // Keep reading until we find the prompt or timeout
        while start_time.elapsed() < timeout {
            let mut buffer = vec![0; 1024];
            let mut remote_conn = self.remote_conn.borrow_mut();
            let channel = remote_conn.as_mut()
                .ok_or_else(|| NetsshError::ReadError("Attempt to read, but there is no active channel.".to_string()))?;
            
            match channel.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]).to_string();
                    debug!(target: "SSHChannel::read_until_prompt", "Read chunk: {:?}", chunk);
                    output.push_str(&chunk);
                    
                    // Check if we found the prompt
                    if prompt_regex.is_match(&output) {
                        debug!(target: "SSHChannel::read_until_prompt", "Found prompt, breaking read loop");
                        break;
                    }
                },
                Ok(0) => {
                    debug!(target: "SSHChannel::read_until_prompt", "Channel stream closed by remote device");
                    break;
                },
                Ok(_) => {
                    debug!(target: "SSHChannel::read_until_prompt", "No data available to read");
                    std::thread::sleep(Duration::from_millis(100));
                },
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                    debug!(target: "SSHChannel::read_until_prompt", "Timed out or would block, waiting...");
                    std::thread::sleep(Duration::from_millis(100));
                },
                Err(e) => {
                    debug!(target: "SSHChannel::read_until_prompt", "Error reading from channel: {}", e);
                    return Err(NetsshError::IoError(e));
                }
            }
        }
        
        if start_time.elapsed() >= timeout {
            debug!(target: "SSHChannel::read_until_prompt", "Timeout reached waiting for prompt");
            return Err(NetsshError::TimeoutError("Timeout waiting for prompt".to_string()));
        }

        debug!(target: "SSHChannel::read_until_prompt", "Read result: {:?}", output);
        Ok(output)
    }
    
    pub fn read_ignore(&self, prompt_regex: &Regex) -> Result<(), NetsshError> {
        debug!(target: "SSHChannel::read_ignore", "Reading and ignoring output until prompt");
        
        // Check if we have a channel
        if self.is_none() {
            return Err(NetsshError::ReadError("Attempt to read, but there is no active channel.".to_string()));
        }

        loop {
            let mut buffer = vec![0; 1024];
            let mut remote_conn = self.remote_conn.borrow_mut();
            let channel = remote_conn.as_mut()
                .ok_or_else(|| NetsshError::ReadError("Attempt to read, but there is no active channel.".to_string()))?;
            
            match channel.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]).to_string();
                    debug!(target: "SSHChannel::read_ignore", "Ignored: {:?}", chunk);
                    
                    // Check if we found the prompt
                    if prompt_regex.is_match(&chunk) {
                        debug!(target: "SSHChannel::read_ignore", "Found prompt. Ready for next command");
                        break;
                    }
                },
                Ok(_) => {
                    debug!(target: "SSHChannel::read_ignore", "Channel stream closed or no data");
                    break;
                },
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                    debug!(target: "SSHChannel::read_ignore", "Timed out... Assuming no data");
                    break;
                },
                Err(e) => {
                    debug!(target: "SSHChannel::read_ignore", "Ignored error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
    
    pub fn execute_raw(&self, command: &str, prompt_regex: &Regex) -> Result<(), NetsshError> {
        debug!(target: "SSHChannel::execute_raw", "Executing raw command: {}", command);
        
        // Write the command
        self.write_channel(command)?;
        self.write_channel("\n")?;
        
        // Read and ignore output until prompt
        self.read_ignore(prompt_regex)?;
        
        Ok(())
    }

    pub fn set_remote_conn(&self, conn: SSH2Channel) {
        *self.remote_conn.borrow_mut() = Some(conn);
    }

    pub fn get_remote_conn(&self) -> Option<&SSH2Channel> {
        if let Some(ref channel) = *self.remote_conn.borrow() {
            unsafe {
                // This is a bit of a hack to get around the borrow checker
                let ptr = channel as *const SSH2Channel;
                Some(&*ptr)
            }
        } else {
            None
        }
    }

    pub fn as_mut(&self) -> Option<&mut SSH2Channel> {
        if self.remote_conn.borrow().is_some() {
            unsafe {
                // This is a bit of a hack to get around the borrow checker
                let ptr = self.remote_conn.as_ptr();
                if let Some(ref mut channel) = *ptr {
                    Some(channel)
                } else {
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn is_none(&self) -> bool {
        self.remote_conn.borrow().is_none()
    }
    
    pub fn is_some(&self) -> bool {
        self.remote_conn.borrow().is_some()
    }

    pub fn set_encoding(&mut self, encoding: String) {
        self.encoding = encoding;
    }

    pub fn get_encoding(&self) -> &str {
        &self.encoding
    }
}

// For backward compatibility
pub struct Channel {
    inner: SSH2Channel,
}

impl Channel {
    pub fn new(channel: SSH2Channel) -> Self {
        Self { inner: channel }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, NetsshError> {
        Ok(self.inner.read(buf)?)
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, NetsshError> {
        Ok(self.inner.write(data)?)
    }

    pub fn write_all(&mut self, data: &[u8]) -> Result<(), NetsshError> {
        Ok(self.inner.write_all(data)?)
    }

    pub fn flush(&mut self) -> Result<(), NetsshError> {
        Ok(self.inner.flush()?)
    }
}

impl Read for Channel {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl Write for Channel {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.write(data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}
