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

/// PQ authorization for a governance action.
///
/// `approvals` are ML-DSA signatures over the canonical governance message
/// for the action, which binds the action's own domain tag
/// (`qrypton-vault/set-operator/v1`, `.../rotate-keys/v1`, `.../set-policy/v1`,
/// `.../cancel-operator-change/v1`, `.../cancel-key-rotation/v1`,
/// `.../cancel-policy-change/v1`), the network
/// id, this vault's address, the action payload, `nonce` and `expires_at`
/// (see `qrypton_types::message`). The nonce space is shared with withdrawals.
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
    /// Rolling-window cap (next phase; not enforced in the first phase).
    pub max_per_window: i128,
}

/// Institutional withdrawal policy. Deny by default: a token without a
/// [`TokenLimit`] cannot be withdrawn.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Policy {
    pub limits: Vec<TokenLimit>,
    /// Rolling-window length for `max_per_window` (next phase; not enforced in
    /// the first phase).
    pub window_seconds: u64,
    /// If non-empty, withdrawals may only go to these addresses.
    pub allowlist: Vec<Address>,
    /// Pre-allowlisted rescue vault (typically another Qrypton Vault controlled
    /// by the same institution, with its own, separate signer set and operator,
    /// preferably on a separate HSM). Withdrawals to it still need the operator and
    /// the PQ quorum, but skip amount limits and the allowlist, so all funds
    /// can be moved to safety within an operator-change time lock.
    pub rescue_vault: Option<Address>,
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

/// A proposed operator change awaiting its time lock.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingOperatorChange {
    pub new_operator: Address,
    /// Ledger timestamp from which `execute_operator_change` succeeds.
    pub executable_at: u64,
}

/// Why a pending proposal (operator or policy change) was cancelled.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CancelCause {
    /// Cancelled by the PQ quorum.
    Quorum = 1,
    /// Invalidated because a signer-set rotation executed.
    KeyRotation = 2,
}

/// A loosening policy change awaiting its time lock.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingPolicyChange {
    pub policy: Policy,
    /// Ledger timestamp from which `execute_policy_change` succeeds.
    pub executable_at: u64,
}

/// Result of `set_policy`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyChangeOutcome {
    /// Tightening change, applied immediately.
    Applied,
    /// Loosening change, executable from the given ledger timestamp.
    Scheduled(u64),
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
