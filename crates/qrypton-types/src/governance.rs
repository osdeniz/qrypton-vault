//! Off-chain representation of governance messages.

use alloc::vec::Vec;

use crate::approval::PqPublicKey;
use crate::message::{DomainTag, MessageError};
use crate::policy::Policy;
use crate::request::AddressBytes;

/// A governance action awaiting PQ approval.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GovernanceAction {
    /// Propose replacing the classical operator (time-locked).
    SetOperator { new_operator: AddressBytes },
    /// Propose replacing the PQ signer set and threshold (time-locked).
    RotateKeys {
        new_signers: Vec<PqPublicKey>,
        new_threshold: u32,
    },
    /// Replace the withdrawal policy.
    SetPolicy { policy: Policy },
    /// Cancel the pending operator change identified by its proposal hash.
    CancelOperatorChange { proposal_hash: [u8; 32] },
    /// Cancel the pending key rotation identified by its proposal hash.
    CancelKeyRotation { proposal_hash: [u8; 32] },
    /// Cancel the pending policy change identified by its proposal hash.
    CancelPolicyChange { proposal_hash: [u8; 32] },
}

impl GovernanceAction {
    /// The domain tag this action is signed under.
    pub const fn domain_tag(&self) -> DomainTag {
        match self {
            GovernanceAction::SetOperator { .. } => DomainTag::SetOperatorV1,
            GovernanceAction::RotateKeys { .. } => DomainTag::RotateKeysV1,
            GovernanceAction::SetPolicy { .. } => DomainTag::SetPolicyV1,
            GovernanceAction::CancelOperatorChange { .. } => DomainTag::CancelOperatorChangeV1,
            GovernanceAction::CancelKeyRotation { .. } => DomainTag::CancelKeyRotationV1,
            GovernanceAction::CancelPolicyChange { .. } => DomainTag::CancelPolicyChangeV1,
        }
    }
}

/// A governance action bound to a network, a vault, a nonce and an expiry:
/// the exact payload that signers approve.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GovernanceMessage {
    /// `SHA-256(network passphrase)`.
    pub network_id: [u8; 32],
    /// The vault contract address.
    pub vault: AddressBytes,
    pub action: GovernanceAction,
    /// Single-use nonce, shared with withdrawals.
    pub nonce: u64,
    /// Ledger timestamp (UNIX seconds) after which the approval is invalid.
    pub expires_at: u64,
}

impl GovernanceMessage {
    /// Encodes the message in the canonical format described in
    /// [`crate::message`].
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!("canonical encoder; must match the contract byte-for-byte")
    }

    /// Decodes a canonical governance message so signers can display it.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        let _ = bytes;
        todo!()
    }
}
