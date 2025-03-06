use netssh_rs::{
    initialize_logging,
    BaseConnection,
    NetsshError,
};
use std::env;
use std::time::Duration;

fn main() -> Result<(), NetsshError> {
    // Initialize logging with both debug and session logging enabled
    initialize_logging(true, true)?;

    // Get environment variables
    let host = env::var("DEVICE_HOST").expect("DEVICE_HOST not set");
    let username = env::var("DEVICE_USER").expect("DEVICE_USER not set");
    let password = env::var("DEVICE_PASS").expect("DEVICE_PASS not set");

    // Create and configure connection
    let mut connection = BaseConnection::new()?;
    connection.set_session_log(String::from("logs/basic_connection.log"))?;

    // Connect to device
    connection.connect(
        &host,
        &username,
        Some(&password),
        Some(22),
        Some(Duration::from_secs(10)),
    )?;

    // Send a command and print output
    connection.write_channel("terminal length 0\n")?;
    let output = connection.read_channel()?;
    println!("Command output: {}", output);

    Ok(())
}
