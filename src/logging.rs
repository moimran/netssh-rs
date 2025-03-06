use log::LevelFilter;
use std::fs::{File, OpenOptions};
use std::io::Write;
use crate::error::NetsshError;
use chrono::Local;

struct MultiWriter {
    debug_file: File,
}

impl MultiWriter {
    fn new(debug_path: &str) -> std::io::Result<Self> {
        let debug_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(debug_path)?;

        Ok(MultiWriter { debug_file })
    }
}

impl Write for MultiWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Write to stdout
        std::io::stdout().write_all(buf)?;
        // Write to debug file
        self.debug_file.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()?;
        self.debug_file.flush()?;
        Ok(())
    }
}

pub fn init_logging(
    debug_enabled: bool,
    _session_logging_enabled: bool, // This is now handled by BaseConnection
) -> Result<(), NetsshError> {
    // Create logs directory if it doesn't exist
    std::fs::create_dir_all("logs")
        .map_err(|e| NetsshError::IoError(e))?;

    // Set up environment for env_logger
    if debug_enabled {
        std::env::set_var("RUST_LOG", "debug");
    } else {
        std::env::set_var("RUST_LOG", "info");
    }

    // Create a custom logger builder
    let mut builder = env_logger::Builder::from_default_env();
    
    // Set the log level
    builder.filter_level(if debug_enabled { LevelFilter::Debug } else { LevelFilter::Info });

    // Create the debug writer
    let writer = MultiWriter::new("logs/debug.log")
        .map_err(|e| NetsshError::IoError(e))?;

    // Set the writer
    builder.target(env_logger::Target::Pipe(Box::new(writer)));

    // Set format with timestamp, file, module path, and target
    builder.format(|buf, record| {
        writeln!(
            buf,
            "{} [{}] [{}:{}] [{}::{}] {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.module_path().unwrap_or("unknown"),
            record.target(),
            record.args()
        )
    });

    // Initialize the logger
    builder.init();

    Ok(())
}
