use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextFSMError {
    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("FSM error: {0}")]
    FSMError(String),

    #[error("Usage error: {0}")]
    UsageError(String),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Fancy Regex error: {0}")]
    FancyRegexError(#[from] fancy_regex::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

// Internal FSM action exceptions (not errors)
#[derive(Debug)]
pub enum FSMAction {
    SkipRecord,
    SkipValue,
}

pub type Result<T> = std::result::Result<T, TextFSMError>;
