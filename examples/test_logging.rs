use netssh_rs::initialize_logging;
use log::{debug, info, warn, error, LevelFilter};
use std::fs;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Clean up old log files
    let _ = fs::remove_file("logs/debug.log");
    let _ = fs::remove_file("logs/session.log");

    println!("Initializing logging with debug and session logging enabled...");
    initialize_logging(true, true)?;

    // Verify that debug logging is enabled
    if log::max_level() >= LevelFilter::Debug {
        println!("Debug logging is enabled (max_level = {:?})", log::max_level());
    } else {
        println!("Warning: Debug logging is not enabled (max_level = {:?})", log::max_level());
    }

    println!("\nSending test messages at different log levels...");
    error!("This is an error message");
    warn!("This is a warning message");
    info!("This is an info message");
    debug!("This is a debug message");
    debug!("Another debug message with some details: value={}", 42);

    // Give the logger a moment to flush
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Check debug.log
    println!("\nChecking debug.log file...");
    let mut debug_content = String::new();
    std::fs::File::open("logs/debug.log")?.read_to_string(&mut debug_content)?;
    println!("\nContents of debug.log:");
    println!("{}", debug_content);

    // Check session.log
    println!("\nChecking session.log file...");
    let mut session_content = String::new();
    std::fs::File::open("logs/session.log")?.read_to_string(&mut session_content)?;
    println!("\nContents of session.log (should not contain DEBUG messages):");
    println!("{}", session_content);

    Ok(())
}
