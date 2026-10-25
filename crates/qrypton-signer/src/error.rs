use thiserror::Error;

/// Errors returned by key backends and the approval flow.
#[derive(Debug, Error)]
pub enum SignerError {
    #[error("key backend error: {0}")]
    Backend(String),
    #[error("key not found: {0}")]
    KeyNotFound(String),
    #[error("unsupported parameter set for this backend")]
    UnsupportedAlgorithm,
    #[error("malformed request: {0}")]
    Message(#[from] qrypton_types::MessageError),
    #[error("request violates vault policy: {0}")]
    PolicyViolation(String),
    #[error("operator declined the request")]
    Declined,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
