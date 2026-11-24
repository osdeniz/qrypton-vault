//! Off-chain representation of a withdrawal request.

use alloc::vec::Vec;

use crate::message::{DomainTag, MessageError, Reader, Writer};

/// An address as the XDR encoding of a Soroban `ScVal::Address`.
///
/// Kept as opaque bytes so the core of this crate does not depend on an XDR
/// library; with the `stellar` feature, [`AddressBytes::from_strkey`] and
/// [`AddressBytes::to_strkey`] convert to and from `G...` / `C...` strkeys.
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
        let mut w = Writer::default();
        w.tag(DomainTag::WithdrawV1);
        w.raw(&self.network_id);
        w.addr(&self.vault);
        w.addr(&self.request.to);
        w.addr(&self.request.token);
        w.raw(&self.request.amount.to_be_bytes());
        w.raw(&self.request.nonce.to_be_bytes());
        w.raw(&self.request.expires_at.to_be_bytes());
        w.finish()
    }

    /// Decodes a canonical `qrypton-vault/withdraw/v1` message. Used by signers
    /// to display exactly what they are about to approve.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        let mut r = Reader::new(bytes);
        r.expect_tag(DomainTag::WithdrawV1)?;
        let network_id = r.array::<32>()?;
        let vault = r.addr()?;
        let to = r.addr()?;
        let token = r.addr()?;
        let amount = i128::from_be_bytes(r.array::<16>()?);
        let nonce = u64::from_be_bytes(r.array::<8>()?);
        let expires_at = u64::from_be_bytes(r.array::<8>()?);
        r.finish()?;
        Ok(Self {
            network_id,
            vault,
            request: WithdrawRequest {
                to,
                token,
                amount,
                nonce,
                expires_at,
            },
        })
    }
}
