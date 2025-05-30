//! # Job Scheduler
//!
//! A powerful Rust-based job scheduler application with SSH execution capabilities,
//! built using Apalis and netssh-rs.
//!
//! ## Features
//!
//! - **Multiple Job Types**: Ad-hoc, one-time scheduled, and recurring jobs with cron expressions
//! - **SSH Command Execution**: Execute commands on remote network devices using netssh-rs
//! - **SQLite Storage**: Lightweight, file-based storage with easy migration to other backends
//! - **Web Dashboard**: Built-in web UI for monitoring and managing jobs
//! - **REST API**: Complete API for job management and monitoring
//! - **Device Support**: Cisco IOS, IOS-XE, NX-OS, ASA, XR, Arista EOS, Juniper JUNOS
//! - **Extensible Architecture**: Easy to add new job types and storage backends
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use scheduler::{Config, SqliteStorage, Storage};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration
//!     let config = Config::from_env().unwrap_or_default();
//!
//!     // Initialize storage
//!     let storage = Arc::new(SqliteStorage::new(&config.database.url).await?);
//!     storage.initialize().await?;
//!
//!     // Your application logic here
//!
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod board;
pub mod config;
pub mod error;
pub mod jobs;
pub mod logging;
pub mod scheduler;
pub mod storage;

pub use config::Config;
pub use error::{Result, SchedulerError};
pub use storage::{SqliteStorage, Storage};
