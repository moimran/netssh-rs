use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetsshError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("SSH error: {0}")]
    SshError(#[from] ssh2::Error),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Command error: {0}")]
    CommandError(String),

    #[error("Read error: {0}")]
    ReadError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("System time error: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),

    #[error("Write error: {0}")]
    WriteError(String),

    #[error("Pattern match error: {0}")]
    PatternError(String),

    #[error("Device error: {0}")]
    DeviceError(String),

    #[error("Unsupported device: {0}")]
    UnsupportedDevice(String),

    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Logger error: {0}")]
    LogError(String),

    #[error("SSH error: {0}")]
    SshErrorNew(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Channel error: {0}")]
    ChannelError(String),

    #[error("Prompt error: {0}")]
    PromptError(String),

    #[error("Disconnect error: {0}")]
    DisconnectError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Session log error: {0}")]
    SessionLogError(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Connection error: failed to connect to {addr}: {source}")]
    ConnectionFailed {
        addr: String,
        #[source]
        source: io::Error,
    },

    #[error("SSH handshake failed: {source}")]
    SshHandshakeFailed {
        #[source]
        source: ssh2::Error,
    },

    #[error("Authentication failed for user {username}: {source}")]
    AuthenticationFailed {
        username: String,
        #[source]
        source: ssh2::Error,
    },

    #[error("Channel operation failed: {message}")]
    ChannelFailed {
        message: String,
        #[source]
        source: Option<ssh2::Error>,
    },

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Timeout occurred while {action}")]
    Timeout { action: String },

    #[error("Operation error: {0}")]
    OperationError(String),
}

// Helper methods for error context
impl NetsshError {
    pub fn connection_failed(addr: impl Into<String>, err: io::Error) -> Self {
        Self::ConnectionFailed {
            addr: addr.into(),
            source: err,
        }
    }

    pub fn ssh_handshake_failed(err: ssh2::Error) -> Self {
        Self::SshHandshakeFailed { source: err }
    }

    pub fn authentication_failed(username: impl Into<String>, err: ssh2::Error) -> Self {
        Self::AuthenticationFailed {
            username: username.into(),
            source: err,
        }
    }

    pub fn channel_failed(message: impl Into<String>, source: Option<ssh2::Error>) -> Self {
        Self::ChannelFailed {
            message: message.into(),
            source,
        }
    }

    pub fn timeout(action: impl Into<String>) -> Self {
        Self::Timeout {
            action: action.into(),
        }
    }
}
