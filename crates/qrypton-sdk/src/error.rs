use thiserror::Error;

/// Errors returned by the SDK.
#[derive(Debug, Error)]
pub enum SdkError {
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("invalid address: {0}")]
    InvalidAddress(String),
    #[error("transaction simulation failed: {0}")]
    Simulation(String),
    #[error("transaction failed: {0}")]
    Transaction(String),
    #[error("approval threshold not reached: {collected} of {threshold}")]
    ThresholdNotReached { collected: u32, threshold: u32 },
    #[error("approval source {0} declined")]
    Declined(String),
    #[error(transparent)]
    Message(#[from] qrypton_types::MessageError),
}
