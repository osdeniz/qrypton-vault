use soroban_sdk::{contracttype, Address, BytesN, Vec};

/// An ML-DSA public key (FIPS 204 encoding). The variant fixes the parameter
/// set and the encoded length.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PqPublicKey {
    MlDsa44(BytesN<1312>),
    /// Default parameter set for vault signers.
    MlDsa65(BytesN<1952>),
    MlDsa87(BytesN<2592>),
}

/// An ML-DSA signature (FIPS 204 encoding).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PqSignature {
    MlDsa44(BytesN<2420>),
    MlDsa65(BytesN<3309>),
    MlDsa87(BytesN<4627>),
}

/// One PQ signer's approval. `signer_index` is the position of the signer's
/// key in the current signer set.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PqApproval {
    pub signer_index: u32,
    pub signature: PqSignature,
}

/// A withdrawal request. Signers approve the canonical encoding of this
/// request bound to the network and the vault (see `qrypton_types::message`).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawRequest {
    pub to: Address,
    pub token: Address,
    pub amount: i128,
    pub nonce: u64,
    /// Ledger timestamp (UNIX seconds) after which the request is invalid.
    pub expires_at: u64,
}

/// PQ authorization for a governance action (policy change, rotation).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceAuth {
    pub nonce: u64,
    pub expires_at: u64,
    pub approvals: Vec<PqApproval>,
}

/// Withdrawal limits for one token.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenLimit {
    pub token: Address,
    pub max_per_withdrawal: i128,
    pub max_per_window: i128,
}

/// Institutional withdrawal policy. Deny by default: a token without a
/// [`TokenLimit`] cannot be withdrawn.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Policy {
    pub limits: Vec<TokenLimit>,
    pub window_seconds: u64,
    /// If non-empty, withdrawals may only go to these addresses.
    pub allowlist: Vec<Address>,
}

/// A proposed signer-set rotation awaiting its time lock.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingRotation {
    pub new_signers: Vec<PqPublicKey>,
    pub new_threshold: u32,
    /// Ledger timestamp from which `execute_key_rotation` succeeds.
    pub executable_at: u64,
}

/// Why a withdrawal was rejected without failing the invocation.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum RejectReason {
    Expired = 1,
    NonceReused = 2,
    TokenNotAllowed = 3,
    PerWithdrawalLimit = 4,
    WindowLimit = 5,
    RecipientNotAllowed = 6,
}

/// Result of a withdrawal attempt.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WithdrawOutcome {
    Executed,
    Rejected(RejectReason),
}
