use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

mod utils;
use crate::utils::mock_device::{MockNetworkDevice, PromptStyle};

#[cfg(test)]
mod tests {
    use super::*;
    use netssh_core::{device_connection::NetworkDeviceConnection, error::NetsshError};
    use std::thread;

    fn read_until_prompt(stream: &mut TcpStream, prompt: &str) -> Result<String, std::io::Error> {
        let mut buffer = [0u8; 1024];
        let mut output = String::new();
        let mut tries = 0;
        const MAX_TRIES: usize = 10;

        while tries < MAX_TRIES {
            match stream.read(&mut buffer) {
                Ok(0) => break, // Connection closed
                Ok(n) => {
                    output.push_str(&String::from_utf8_lossy(&buffer[..n]));
                    if output.contains(prompt) {
                        return Ok(output);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    tries += 1;
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        if !output.contains(prompt) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!("Prompt '{}' not found in output: {:?}", prompt, output),
            ));
        }

        Ok(output)
    }

    fn setup_mock_device() -> MockNetworkDevice {
        let mut device = MockNetworkDevice::new();

        // Configure the mock device
        device.set_hostname("TestRouter")
              .set_prompt_style(PromptStyle::CiscoIOS)
              .add_auth_credentials("admin", "password")
              .add_command_response("show version", 
                "Cisco IOS Software, Version 15.2(4)M\nUptime: 10 days\nProcessor: test")
              .add_command_response("show interfaces", 
                "GigabitEthernet0/0\n  Hardware: Ethernet, address: 0000.0000.0001\n  Internet address: 192.168.1.1/24")
              .add_command_response("configure terminal", 
                "Enter configuration commands, one per line. End with CNTL/Z.")
              .add_command_response("hostname NewRouter", 
                "");

        // Start the mock device server
        device.start().expect("Failed to start mock device");

        device
    }

    #[test]
    fn test_basic_device_operations() -> Result<(), std::io::Error> {
        // Set up a timeout for the entire test
        let test_timeout = Duration::from_secs(30);
        let test_start = std::time::Instant::now();

        // Create a new mock device instance
        let mut device = MockNetworkDevice::new();

        // Set up command responses
        device.add_command_response("show version", "Cisco IOS Software\nUptime: 10 days\n");
        device.add_command_response("show interfaces", "GigabitEthernet0/0\n192.168.1.1/24\n");

        // Set up authentication credentials
        device.add_auth_credentials("test", "test");

        // Set up device info
        device.set_hostname("Router1");
        device.set_prompt_style(PromptStyle::CiscoIOS);

        // Get the port and start the device
        let port = device.port();
        device.start().unwrap();

        // Connect to the device using raw TCP
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        // Read the initial banner and prompt
        let initial_output = read_until_prompt(&mut stream, "Router1>")?;
        assert!(
            initial_output.contains("Welcome to Cisco IOS"),
            "Expected welcome banner"
        );
        assert!(initial_output.contains("Router1>"), "Expected prompt");

        // Send a test command
        stream.write_all(b"show version\n")?;
        let output = read_until_prompt(&mut stream, "Router1>")?;
        assert!(
            output.contains("Cisco IOS Software"),
            "Expected version info"
        );
        assert!(output.contains("Uptime: 10 days"), "Expected uptime info");

        // Send another test command
        stream.write_all(b"show interfaces\n")?;
        let output = read_until_prompt(&mut stream, "Router1>")?;
        assert!(
            output.contains("GigabitEthernet0/0"),
            "Expected interface info"
        );
        assert!(output.contains("192.168.1.1/24"), "Expected IP info");

        // Send exit command
        stream.write_all(b"exit\n")?;
        let mut buffer = [0u8; 1024];
        let n = stream.read(&mut buffer)?;
        let output = String::from_utf8_lossy(&buffer[..n]);
        assert!(output.contains("Goodbye!"), "Expected goodbye message");

        // Check if we've exceeded the timeout
        if test_start.elapsed() > test_timeout {
            panic!(
                "Test exceeded timeout of {} seconds",
                test_timeout.as_secs()
            );
        }

        // Stop the mock device
        device.stop().unwrap();

        Ok(())
    }

    #[test]
    fn test_connection_timeout() -> Result<(), std::io::Error> {
        // Try to connect to an invalid address with a short timeout
        let result = TcpStream::connect_timeout(
            &"192.0.2.1:22".parse().unwrap(),
            Duration::from_millis(100),
        );

        assert!(result.is_err());
        Ok(())
    }
}
