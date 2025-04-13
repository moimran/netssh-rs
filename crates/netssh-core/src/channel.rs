use crate::error::NetsshError;
use regex::Regex;
use ssh2::Channel as SSH2Channel;
use std::cell::RefCell;
use std::io::{Read, Write};
use std::time::Duration;
use tracing::debug;

// Optimal buffer size based on typical network device response sizes
const DEFAULT_BUFFER_SIZE: usize = 16384; // 16KB

pub struct SSHChannel {
    remote_conn: RefCell<Option<SSH2Channel>>,
    encoding: String,
    base_prompt: Option<String>,
    prompt_regex: Option<Regex>,
    // Add a reusable buffer to avoid allocations
    read_buffer: RefCell<Vec<u8>>,
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
            read_buffer: RefCell::new(Vec::with_capacity(DEFAULT_BUFFER_SIZE)),
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
            read_buffer: RefCell::new(Vec::with_capacity(DEFAULT_BUFFER_SIZE)),
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
            }
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
        let channel = remote_conn.as_mut().ok_or_else(|| {
            NetsshError::WriteError(
                "Attempt to write data, but there is no active channel.".to_string(),
            )
        })?;

        // Convert string to bytes and write to channel
        let bytes = out_data.as_bytes();
        channel
            .write_all(bytes)
            .map_err(|e| NetsshError::WriteError(format!("Failed to write to channel: {}", e)))?;

        // Flush the channel to ensure all data is sent
        channel
            .flush()
            .map_err(|e| NetsshError::WriteError(format!("Failed to flush channel: {}", e)))?;

        debug!(target: "SSHChannel::write_channel", "Successfully wrote to channel");
        Ok(())
    }

    pub fn read_buffer(&self, prompt_regex: Option<&Regex>) -> Result<String, NetsshError> {
        debug!(target: "SSHChannel::read_buffer", "Reading buffer from channel");

        let mut remote_conn = self.remote_conn.borrow_mut();
        let channel = remote_conn.as_mut().ok_or_else(|| {
            NetsshError::ReadError("Attempt to read, but there is no active channel.".to_string())
        })?;

        // Reuse the existing buffer instead of allocating a new one
        let mut buffer = self.read_buffer.borrow_mut();

        // Ensure buffer has enough capacity, but don't reallocate if already adequate
        let current_capacity = buffer.capacity();
        if current_capacity < DEFAULT_BUFFER_SIZE {
            buffer.reserve(DEFAULT_BUFFER_SIZE - current_capacity);
        }

        // Clear but preserve capacity
        buffer.clear();

        // Resize to capacity for reading
        let capacity = buffer.capacity();
        buffer.resize(capacity, 0);

        let mut output = String::with_capacity(DEFAULT_BUFFER_SIZE);

        // Check if data is available
        debug!(target: "SSHChannel::read_buffer", "Checking if data is available to read");
        match channel.read(&mut buffer) {
            Ok(n) if n > 0 => {
                debug!(target: "SSHChannel::read_buffer", "Read {} bytes from channel", n);

                // Convert only the valid bytes (0..n) to a string to avoid UTF-8 validation on unused parts
                let chunk = match std::str::from_utf8(&buffer[..n]) {
                    Ok(s) => s.to_string(),
                    Err(e) => {
                        debug!(target: "SSHChannel::read_buffer", "UTF-8 conversion error: {}", e);
                        // Fallback to lossy conversion only when needed
                        String::from_utf8_lossy(&buffer[..n]).to_string()
                    }
                };

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
            }
            Ok(0) => {
                debug!(target: "SSHChannel::read_buffer", "Channel stream closed by remote device");
                return Err(NetsshError::ReadError(
                    "Channel stream closed by remote device.".to_string(),
                ));
            }
            Ok(_) => {
                debug!(target: "SSHChannel::read_buffer", "No data available to read");
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                debug!(target: "SSHChannel::read_buffer", "Would block, no data available");
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                debug!(target: "SSHChannel::read_buffer", "Timed out, no data available");
            }
            Err(e) => {
                debug!(target: "SSHChannel::read_buffer", "Error reading from channel: {}", e);
                return Err(NetsshError::IoError(e));
            }
        }

        debug!(target: "SSHChannel::read_buffer", "Read buffer result length: {}", output.len());
        debug!(target: "SSHChannel::read_buffer", "Read buffer result: {}", output);
        Ok(output)
    }

    pub fn read_channel(&self) -> Result<String, NetsshError> {
        debug!(target: "SSHChannel::read_channel", "Reading all available data from channel");

        // Check if we have a channel
        if self.is_none() {
            return Err(NetsshError::ReadError(
                "Attempt to read, but there is no active channel.".to_string(),
            ));
        }

        let mut remote_conn = self.remote_conn.borrow_mut();
        let channel = remote_conn.as_mut().unwrap(); // Safe because we checked is_none()

        let mut output = String::new();
        let mut read_something = false;
        let mut buffer = vec![0; 8192];

        // Read while there's data and we haven't found a prompt
        while !channel.eof() {
            match channel.read(&mut buffer) {
                Ok(n) => {
                    if n > 0 {
                        read_something = true;
                        // Optimize UTF-8 validation by only processing bytes we've read
                        match std::str::from_utf8(&buffer[..n]) {
                            Ok(s) => output.push_str(s),
                            Err(_) => output.push_str(&String::from_utf8_lossy(&buffer[..n])),
                        }

                        debug!(target: "SSHChannel::read_channel", "Read bytes from channel loop output: {}", output);

                        if output.contains("assword:"){
                            debug!(target: "SSHChannel::read_channel", "Found prompt/terminator, exiting read loop");
                            break;
                        }

                        // If we have a prompt or terminating character, break early
                        if output.contains(">") || output.contains("#") {
                            debug!(target: "SSHChannel::read_channel", "Found prompt/terminator, exiting read loop");
                            break;
                        }
                    } else {
                        // No data available or connection closed
                        debug!(target: "SSHChannel::read_channel", "No data available or channel closed");
                        if !read_something {
                            return Ok(String::new());
                        }
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    debug!(target: "SSHChannel::read_channel", "No more data available right now, read_something: {}", read_something);
                    if !read_something {
                        return Ok(String::new());
                    }
                    // Short sleep before checking eof() again
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    debug!(target: "SSHChannel::read_channel", "Error reading from channel: {}", e);
                    break;
                }
            }
        }

        debug!(target: "SSHChannel::read_channel", "Read complete, output length: {}", output.len());
        Ok(output)
    }

    pub fn read_channel_until_pattern(&self, pattern: &str) -> Result<String, NetsshError> {
        debug!(target: "SSHChannel::read_channel", "Reading all available data from channel");

        // Check if we have a channel
        if self.is_none() {
            return Err(NetsshError::ReadError(
                "Attempt to read, but there is no active channel.".to_string(),
            ));
        }

        let mut remote_conn = self.remote_conn.borrow_mut();
        let channel = remote_conn.as_mut().unwrap(); // Safe because we checked is_none()

        let mut output = String::new();
        let mut read_something = false;
        let mut buffer = vec![0; 8192];

        // Read while there's data and we haven't found a prompt
        while !channel.eof() {
            match channel.read(&mut buffer) {
                Ok(n) => {
                    if n > 0 {
                        read_something = true;
                        // Optimize UTF-8 validation by only processing bytes we've read
                        match std::str::from_utf8(&buffer[..n]) {
                            Ok(s) => output.push_str(s),
                            Err(_) => output.push_str(&String::from_utf8_lossy(&buffer[..n])),
                        }

                        debug!(target: "SSHChannel::read_channel_until_pattern", "Read bytes from channel loop output: {}", output);

                        if output.contains(pattern) {
                            debug!(target: "SSHChannel::read_channel_until_pattern", "Found pattern, exiting read loop");
                            break;
                        }

                        // If we have a prompt or terminating character, break early
                        if output.contains(">") || output.contains("#") {
                            debug!(target: "SSHChannel::read_channel_until_pattern", "Found prompt/terminator, exiting read loop");
                            break;
                        }
                    } else {
                        // No data available or connection closed
                        debug!(target: "SSHChannel::read_channel_until_pattern", "No data available or channel closed");
                        if !read_something {
                            return Ok(String::new());
                        }
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    debug!(target: "SSHChannel::read_channel_until_pattern", "No more data available right now, read_something: {}", read_something);
                    if !read_something {
                        return Ok(String::new());
                    }
                    // Short sleep before checking eof() again
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    debug!(target: "SSHChannel::read_channel_until_pattern", "Error reading from channel: {}", e);
                    break;
                }
            }
        }

        debug!(target: "SSHChannel::read_channel_until_pattern", "Read complete, output length: {}", output.len());
        Ok(output)
    }

    pub fn read_until_prompt(
        &self,
        timeout: Option<Duration>,
        custom_prompt: Option<&Regex>,
    ) -> Result<String, NetsshError> {
        debug!(target: "SSHChannel::read_until_prompt", "Reading until prompt");

        // Check if we have a channel
        if self.is_none() {
            return Err(NetsshError::ReadError(
                "Attempt to read, but there is no active channel.".to_string(),
            ));
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
            let new_output = self.read_buffer(Some(prompt_regex))?;
            if new_output.is_empty() {
                break;
            }
            output.push_str(&new_output);

            // Check if we found the prompt
            if prompt_regex.is_match(&output) {
                debug!(target: "SSHChannel::read_until_prompt", "Found prompt, breaking read loop");
                break;
            }
        }

        if start_time.elapsed() >= timeout {
            debug!(target: "SSHChannel::read_until_prompt", "Timeout reached waiting for prompt");
            return Err(NetsshError::TimeoutError(
                "Timeout waiting for prompt".to_string(),
            ));
        }

        debug!(target: "SSHChannel::read_until_prompt", "Read result: {:?}", output);
        Ok(output)
    }

    pub fn read_ignore(&self, prompt_regex: &Regex) -> Result<(), NetsshError> {
        debug!(target: "SSHChannel::read_ignore", "Reading and ignoring output until prompt");

        // Check if we have a channel
        if self.is_none() {
            return Err(NetsshError::ReadError(
                "Attempt to read, but there is no active channel.".to_string(),
            ));
        }

        loop {
            let mut buffer = vec![0; 1024];
            let mut remote_conn = self.remote_conn.borrow_mut();
            let channel = remote_conn.as_mut().ok_or_else(|| {
                NetsshError::ReadError(
                    "Attempt to read, but there is no active channel.".to_string(),
                )
            })?;

            match channel.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]).to_string();
                    debug!(target: "SSHChannel::read_ignore", "Ignored: {:?}", chunk);

                    // Check if we found the prompt
                    if prompt_regex.is_match(&chunk) {
                        debug!(target: "SSHChannel::read_ignore", "Found prompt. Ready for next command");
                        break;
                    }
                }
                Ok(_) => {
                    debug!(target: "SSHChannel::read_ignore", "Channel stream closed or no data");
                    break;
                }
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    debug!(target: "SSHChannel::read_ignore", "Timed out... Assuming no data");
                    break;
                }
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

    /// Close the SSH channel
    pub fn close(&self) -> Result<(), NetsshError> {
        debug!(target: "SSHChannel::close", "Closing channel");

        if let Some(mut channel) = self.remote_conn.borrow_mut().take() {
            // Send EOF to indicate we're done sending data
            if let Err(e) = channel.send_eof() {
                debug!(target: "SSHChannel::close", "Error sending EOF: {}", e);
            }

            // Close the channel
            if let Err(e) = channel.close() {
                debug!(target: "SSHChannel::close", "Error closing channel: {}", e);
                return Err(NetsshError::ChannelError(format!(
                    "Failed to close channel: {}",
                    e
                )));
            }

            // Wait for channel to close
            if let Err(e) = channel.wait_close() {
                debug!(target: "SSHChannel::close", "Error waiting for channel to close: {}", e);
                return Err(NetsshError::ChannelError(format!(
                    "Failed to wait for channel close: {}",
                    e
                )));
            }

            debug!(target: "SSHChannel::close", "Channel closed successfully");
        } else {
            debug!(target: "SSHChannel::close", "No active channel to close");
        }

        Ok(())
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
        self.read(buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl Write for Channel {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.write(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}
