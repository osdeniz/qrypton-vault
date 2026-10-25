//! Audit events. Every state change emits exactly one event so that
//! institutions can reconcile the vault from the event stream alone.

use soroban_sdk::{contractevent, Address, BytesN};

use crate::types::{CancelCause, RejectReason};

/// Funds were deposited into the vault.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deposited {
    #[topic]
    pub token: Address,
    #[topic]
    pub from: Address,
    pub amount: i128,
}

/// A withdrawal passed all checks and was transferred.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawExecuted {
    #[topic]
    pub token: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
    pub nonce: u64,
    /// Indices of the signers whose approvals were counted.
    pub signer_indices: soroban_sdk::Vec<u32>,
}

/// A withdrawal was rejected by expiry, nonce or policy checks.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawRejected {
    #[topic]
    pub token: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
    pub nonce: u64,
    pub reason: RejectReason,
}

/// A new signer set was proposed and is time-locked.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyRotationProposed {
    /// SHA-256 over the canonical encoding of the proposed signer set.
    pub signers_hash: BytesN<32>,
    pub new_threshold: u32,
    pub executable_at: u64,
}

/// The pending signer set became active.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyRotationExecuted {
    pub signers_hash: BytesN<32>,
    pub new_threshold: u32,
}

/// The pending rotation was cancelled.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyRotationCancelled {
    pub signers_hash: BytesN<32>,
}

/// The withdrawal policy was replaced.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyUpdated {
    /// SHA-256 over the XDR encoding of the new policy.
    pub policy_hash: BytesN<32>,
}

/// A new operator was proposed by the PQ quorum and is time-locked.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperatorChangeProposed {
    #[topic]
    pub new_operator: Address,
    pub executable_at: u64,
}

/// The pending operator became active.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperatorChangeExecuted {
    pub old_operator: Address,
    #[topic]
    pub new_operator: Address,
}

/// The pending operator change was cancelled.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperatorChangeCancelled {
    #[topic]
    pub new_operator: Address,
    pub cause: CancelCause,
}
