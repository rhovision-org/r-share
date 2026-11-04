//! Errors from the pure client layer.

/// A domain or encoding error in the client's pure layer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RshareError {
    #[error(transparent)]
    Rcash(#[from] rcash_core::CoreError),
    #[error("invalid terms: {0}")]
    Terms(#[from] TermsError),
    #[error("invalid slot request: {0}")]
    Slots(String),
    #[error("invalid operation id: {0}")]
    OpId(String),
    #[error("invalid manifest: {0}")]
    Manifest(String),
    #[error("JSON value cannot be represented in Rholang: {0}")]
    Json(String),
    #[error("term is {actual} bytes; the project budget is {limit} bytes")]
    TermTooLarge { actual: usize, limit: usize },
}

impl From<serde_json::Error> for RshareError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, RshareError>;

// Re-exported so callers can match a terms refusal without a second import.
pub use crate::terms::TermsError;
