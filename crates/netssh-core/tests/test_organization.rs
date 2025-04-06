mod utils;

#[cfg(test)]
mod tests {
    use super::*;

    // Import directly from the tests/utils/mock_device.rs file
    mod mock_device_mod {
        include!("utils/mock_device.rs");
    }
    use mock_device_mod::{MockNetworkDevice, PromptStyle};

    use std::sync::{Arc, Mutex};

    // Create a mock device for testing
    fn setup_mock_device() -> MockNetworkDevice {
        let mut device = MockNetworkDevice::new();

        device
            .set_device_type("generic")
            .set_hostname("test-device")
            .set_prompt_style(PromptStyle::Custom("test-device# ".to_string()))
            .add_auth_credentials("admin", "admin");

        device.add_command_response("show version", "Test Device Version 1.0\ntest-device# ");

        device
    }

    #[test]
    fn test_organization() {
        // This test simply demonstrates how to work with the reorganized test structure
        // It doesn't execute any actual logic but shows the import patterns
        let device = setup_mock_device();
        assert_eq!(device.port() > 0, true);

        // Show successful organization
        println!("Test organization is working properly!");
    }
}
