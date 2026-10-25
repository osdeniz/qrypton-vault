//! Off-chain representation of a vault's policy.
//!
//! The policy is enforced by the contract; signers and the SDK use this
//! representation to pre-check a request before anyone signs it.

use alloc::vec::Vec;

use crate::request::AddressBytes;

/// Withdrawal limits for one token, in the token's smallest unit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenLimit {
    pub token: AddressBytes,
    /// Maximum amount for a single withdrawal.
    pub max_per_withdrawal: i128,
    /// Maximum cumulative amount within one rolling window (next phase).
    pub max_per_window: i128,
}

/// Institutional withdrawal policy, mirroring the contract's `Policy`.
///
/// Deny by default: a token without a [`TokenLimit`] entry cannot be withdrawn,
/// and only allowlisted recipients (plus the rescue vault) can receive funds
/// unless `open_recipients` is set.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Policy {
    pub limits: Vec<TokenLimit>,
    /// Length of the rolling window in seconds.
    pub window_seconds: u64,
    /// Recipients withdrawals may go to. Closed by default: an empty
    /// allowlist permits no recipient (only the rescue vault).
    pub allowlist: Vec<AddressBytes>,
    /// Lifts the allowlist and permits any recipient. Off by default.
    pub open_recipients: bool,
    /// Pre-allowlisted rescue vault; withdrawals to it skip amount limits and
    /// the allowlist (operator and PQ quorum are still required).
    pub rescue_vault: Option<AddressBytes>,
}
