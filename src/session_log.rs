use crate::error::NetsshError;
use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::Write;

pub struct SessionLog {
    file: Option<File>,
    enabled: bool,
}

impl SessionLog {
    pub fn new() -> Self {
        SessionLog {
            file: None,
            enabled: false,
        }
    }

    pub fn enable(&mut self, path: &str) -> Result<(), NetsshError> {
        // Create logs directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| NetsshError::IoError(e))?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)
            .map_err(|e| NetsshError::IoError(e))?;

        // Write session start header
        writeln!(file, "{}", "=".repeat(80)).map_err(|e| NetsshError::IoError(e))?;
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        writeln!(file, "SESSION START: {}", timestamp).map_err(|e| NetsshError::IoError(e))?;
        writeln!(file, "{}", "=".repeat(80)).map_err(|e| NetsshError::IoError(e))?;
        file.flush().map_err(|e| NetsshError::IoError(e))?;

        self.file = Some(file);
        self.enabled = true;
        Ok(())
    }

    pub fn disable(&mut self) {
        if self.enabled {
            if let Some(mut file) = self.file.take() {
                // Try to write the session end header
                let _ = writeln!(file, "\n{}", "=".repeat(80));
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let _ = writeln!(file, "SESSION END: {}", timestamp);
                let _ = writeln!(file, "{}", "=".repeat(80));
            }
            self.enabled = false;
        }
    }

    pub fn log_command(&mut self, command: &str, output: &str) -> Result<(), NetsshError> {
        if let Some(file) = self.file.as_mut() {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

            // Write command with timestamp
            writeln!(file, "\n{}", "-".repeat(80)).map_err(|e| NetsshError::IoError(e))?;
            writeln!(file, "Command Executed [{}]", timestamp)
                .map_err(|e| NetsshError::IoError(e))?;
            writeln!(file, "{}", "-".repeat(80)).map_err(|e| NetsshError::IoError(e))?;
            writeln!(file, "Input:").map_err(|e| NetsshError::IoError(e))?;
            writeln!(file, "{}", command).map_err(|e| NetsshError::IoError(e))?;

            // Write output
            writeln!(file, "\nOutput:").map_err(|e| NetsshError::IoError(e))?;
            writeln!(file, "{}", output.trim()).map_err(|e| NetsshError::IoError(e))?;

            // Write footer
            writeln!(file, "{}", "-".repeat(80)).map_err(|e| NetsshError::IoError(e))?;

            file.flush().map_err(|e| NetsshError::IoError(e))?;
        }
        Ok(())
    }

    pub fn write_raw(&mut self, data: &[u8]) -> Result<(), NetsshError> {
        if let Some(file) = self.file.as_mut() {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

            // Write raw data with timestamp
            writeln!(file, "\n{}", "-".repeat(80)).map_err(|e| NetsshError::IoError(e))?;
            writeln!(file, "Raw Data Written [{}]", timestamp)
                .map_err(|e| NetsshError::IoError(e))?;
            writeln!(file, "{}", "-".repeat(80)).map_err(|e| NetsshError::IoError(e))?;

            // Write the raw data as both hex and UTF-8 (if valid)
            writeln!(file, "Hex: {:02X?}", data).map_err(|e| NetsshError::IoError(e))?;
            if let Ok(text) = String::from_utf8(data.to_vec()) {
                writeln!(file, "Text: {}", text).map_err(|e| NetsshError::IoError(e))?;
            }

            writeln!(file, "{}", "-".repeat(80)).map_err(|e| NetsshError::IoError(e))?;

            file.flush().map_err(|e| NetsshError::IoError(e))?;
        }
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.enabled
    }

    pub fn write(&mut self, data: &str) -> Result<(), NetsshError> {
        self.write_raw(data.as_bytes())
    }
}

impl Drop for SessionLog {
    fn drop(&mut self) {
        self.disable();
    }
}
