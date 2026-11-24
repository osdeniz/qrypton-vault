//! Off-chain Stellar helpers (feature `stellar`): strkey ⇄ XDR address
//! conversion and network ids.

use core::str::FromStr;

use sha2::{Digest, Sha256};
use stellar_xdr::{Limits, ReadXdr, ScAddress, ScVal, WriteXdr};

use crate::message::MessageError;
use crate::request::AddressBytes;

impl AddressBytes {
    /// Encodes a `G...` or `C...` strkey as XDR `ScVal::Address`, byte-for-byte
    /// what `Address::to_xdr` produces inside a Soroban contract.
    pub fn from_strkey(strkey: &str) -> Result<Self, MessageError> {
        let addr = ScAddress::from_str(strkey).map_err(|_| MessageError::InvalidAddress)?;
        ScVal::Address(addr)
            .to_xdr(Limits::none())
            .map(AddressBytes)
            .map_err(|_| MessageError::InvalidAddress)
    }

    /// Decodes back to a strkey for display.
    pub fn to_strkey(&self) -> Result<String, MessageError> {
        match ScVal::from_xdr(&self.0, Limits::none()) {
            Ok(ScVal::Address(addr)) => Ok(addr.to_string()),
            _ => Err(MessageError::InvalidAddress),
        }
    }
}

/// `SHA-256(network passphrase)`, the network id Soroban exposes as
/// `env.ledger().network_id()`.
pub fn network_id(passphrase: &str) -> [u8; 32] {
    Sha256::digest(passphrase.as_bytes()).into()
}
