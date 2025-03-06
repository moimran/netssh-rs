use std::collections::HashMap;
use std::io::{Read, Write};
use mockall::mock;

// Mock SSH session
mock! {
    pub Session {
        pub fn handshake(&self) -> std::io::Result<()>;
        pub fn userauth_password(&self, username: &str, password: &str) -> std::io::Result<()>;
        pub fn channel_session(&self) -> std::io::Result<Channel>;
    }
}

// Mock SSH channel
mock! {
    pub Channel {
        pub fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
        pub fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
        pub fn flush(&mut self) -> std::io::Result<()>;
        pub fn eof(&self) -> bool;
        pub fn request_pty(&mut self, term: &str) -> std::io::Result<()>;
        pub fn shell(&mut self) -> std::io::Result<()>;
    }
}

pub struct MockDevice {
    responses: HashMap<String, String>,
    prompt: String,
}

impl MockDevice {
    pub fn new(device_type: DeviceType) -> Self {
        let mut device = Self {
            responses: HashMap::new(),
            prompt: String::new(),
        };
        device.setup_device(device_type);
        device
    }

    fn setup_device(&mut self, device_type: DeviceType) {
        match device_type {
            DeviceType::CiscoXr => {
                self.prompt = "RP/0/RP0/CPU0:ios#".to_string();
                self.responses.insert(
                    "show version".to_string(),
                    format!("Cisco IOS XR Software, Version 7.3.2\nCopyright (c) 2013-2021 by Cisco Systems, Inc.\n\n{}", self.prompt)
                );
                self.responses.insert(
                    "configure terminal".to_string(),
                    format!("Enter configuration commands, one per line. End with CNTL/Z.\n{}", self.prompt)
                );
                self.responses.insert(
                    "end".to_string(),
                    format!("End of configuration\n{}", self.prompt)
                );
            }
        }
    }

    pub fn create_mocked_session(&self) -> MockSession {
        let mut session = MockSession::new();
        let device = self.clone();

        // Mock handshake
        session.expect_handshake()
            .returning(|| Ok(()));

        // Mock authentication
        session.expect_userauth_password()
            .returning(|username, password| {
                if username == "valid_user" && password == "valid_pass" {
                    Ok(())
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Authentication failed"))
                }
            });

        // Mock channel creation
        session.expect_channel_session()
            .returning(move || {
                let mut channel = MockChannel::new();
                let device = device.clone();

                // Mock read
                channel.expect_read()
                    .returning(move |buf| {
                        // Simulate response based on last command
                        let response = device.get_response();
                        let bytes = response.as_bytes();
                        let len = std::cmp::min(buf.len(), bytes.len());
                        buf[..len].copy_from_slice(&bytes[..len]);
                        Ok(len)
                    });

                // Mock write
                channel.expect_write()
                    .returning(|buf| Ok(buf.len()));

                // Mock flush
                channel.expect_flush()
                    .returning(|| Ok(()));

                // Mock eof
                channel.expect_eof()
                    .returning(|| false);

                // Mock pty request
                channel.expect_request_pty()
                    .returning(|_| Ok(()));

                // Mock shell
                channel.expect_shell()
                    .returning(|| Ok(()));

                Ok(channel)
            });

        session
    }

    fn get_response(&self, command: &str) -> String {
        self.responses.get(command)
            .cloned()
            .unwrap_or_else(|| format!("Unknown command: {}\n{}", command, self.prompt))
    }
}

#[derive(Clone)]
pub enum DeviceType {
    CiscoXr,
}
