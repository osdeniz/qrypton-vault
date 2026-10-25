//! Off-chain representation of a withdrawal request.

use alloc::vec::Vec;

use crate::message::MessageError;

/// An address as the XDR encoding of a Soroban `ScVal::Address`.
///
/// Kept as opaque bytes so this crate does not depend on an XDR library; the
/// SDK converts to and from strkeys (`G...`, `C...`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AddressBytes(pub Vec<u8>);

/// A withdrawal request, mirroring the contract's `WithdrawRequest`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WithdrawRequest {
    /// Recipient of the funds.
    pub to: AddressBytes,
    /// Token contract (Stellar Asset Contract or SEP-41 token).
    pub token: AddressBytes,
    /// Amount in the token's smallest unit. Must be positive.
    pub amount: i128,
    /// Single-use nonce. The vault rejects a nonce it has already consumed.
    pub nonce: u64,
    /// Ledger timestamp (UNIX seconds) after which the request is invalid.
    pub expires_at: u64,
}

/// A withdrawal request bound to a network and a vault: the exact payload
/// that signers approve.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WithdrawMessage {
    /// `SHA-256(network passphrase)`.
    pub network_id: [u8; 32],
    /// The vault contract address.
    pub vault: AddressBytes,
    /// The request being approved.
    pub request: WithdrawRequest,
}

impl WithdrawMessage {
    /// Encodes the message in the canonical `qrypton-vault/withdraw/v1` format
    /// described in [`crate::message`].
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!("canonical encoder; must match the contract byte-for-byte")
    }

    /// Decodes a canonical `qrypton-vault/withdraw/v1` message. Used by signers
    /// to display exactly what they are about to approve.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        let _ = bytes;
        todo!("canonical decoder; rejects trailing bytes and wrong domain tags")
    }
}
