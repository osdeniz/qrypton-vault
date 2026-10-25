//! Decoded vault audit events.

use crate::SdkError;

/// Position in the event stream (Stellar RPC paging token or start ledger).
#[derive(Clone, Debug)]
pub enum EventCursor {
    /// Start at the given ledger sequence.
    Ledger(u32),
    /// Resume after a previously returned paging token.
    Token(String),
}

/// A vault event, decoded from its Soroban representation.
#[derive(Clone, Debug)]
pub enum VaultEvent {
    Deposited {
        token: String,
        from: String,
        amount: i128,
    },
    WithdrawExecuted {
        token: String,
        to: String,
        amount: i128,
        nonce: u64,
        signer_indices: Vec<u32>,
    },
    WithdrawRejected {
        token: String,
        to: String,
        amount: i128,
        nonce: u64,
        reason: String,
    },
    KeyRotationProposed {
        signers_hash: [u8; 32],
        new_threshold: u32,
        executable_at: u64,
    },
    KeyRotationExecuted {
        signers_hash: [u8; 32],
        new_threshold: u32,
    },
    KeyRotationCancelled {
        signers_hash: [u8; 32],
    },
    PolicyUpdated {
        policy_hash: [u8; 32],
    },
    OperatorChangeProposed {
        new_operator: String,
        executable_at: u64,
    },
    OperatorChangeExecuted {
        old_operator: String,
        new_operator: String,
    },
    OperatorChangeCancelled {
        new_operator: String,
        cause: String,
    },
}

/// A live subscription to vault events.
pub struct EventSubscription {
    cursor: EventCursor,
}

impl EventSubscription {
    /// Waits for the next event. Returns `None` when the subscription is closed.
    pub async fn next(&mut self) -> Option<Result<(VaultEvent, EventCursor), SdkError>> {
        todo!()
    }

    /// Cursor to resume from after a restart.
    pub fn cursor(&self) -> &EventCursor {
        &self.cursor
    }
}
