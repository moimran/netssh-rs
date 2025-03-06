pub mod base_connection;
pub mod config;
pub mod error;
pub mod logging;
pub mod session_log;
pub mod vendors;

pub use base_connection::BaseConnection;
pub use config::{NetsshConfig, NetsshConfigBuilder};
pub use error::NetsshError;
pub use logging::init_logging as initialize_logging;
pub use vendors::cisco::{CiscoBaseConnection, CiscoXrSsh};
