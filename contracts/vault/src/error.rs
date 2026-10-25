use soroban_sdk::contracterror;

/// Errors that fail the invocation (all effects are rolled back).
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Threshold is zero or larger than the signer set.
    InvalidThreshold = 1,
    /// The same public key appears more than once in a signer set.
    DuplicateSigner = 2,
    /// The policy is internally inconsistent (e.g. negative limits).
    InvalidPolicy = 3,
    /// Amount is zero or negative.
    InvalidAmount = 4,
    /// Fewer valid approvals than the threshold.
    InsufficientApprovals = 5,
    /// An approval references a signer index outside the current set.
    UnknownSigner = 6,
    /// Two approvals reference the same signer index.
    DuplicateApproval = 7,
    /// Signature parameter set does not match the registered key.
    AlgorithmMismatch = 8,
    /// A governance authorization has expired or reuses a nonce.
    StaleGovernanceAuth = 9,
    /// No rotation is pending.
    NoPendingRotation = 10,
    /// A rotation is already pending.
    RotationAlreadyPending = 11,
    /// The rotation time lock has not elapsed yet.
    RotationTimelocked = 12,
    /// No operator change is pending.
    NoPendingOperatorChange = 13,
    /// An operator change is already pending.
    OperatorChangeAlreadyPending = 14,
    /// The operator-change time lock has not elapsed yet.
    OperatorChangeTimelocked = 15,
}
