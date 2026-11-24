//! Canonical format of the messages that PQ signers approve.
//!
//! Every approval is an ML-DSA signature over a byte string built from a
//! *domain-separation tag*, the network, the vault contract and the fields of
//! the action being approved. Binding all of these prevents an approval from
//! being replayed on another network, against another vault, or for another
//! kind of action.
//!
//! # Withdrawal message (`qrypton-vault/withdraw/v1`)
//!
//! All integers are big-endian. `addr` is an address field encoded as
//! `u32 length || bytes`, where `bytes` is the XDR encoding of the Soroban
//! `ScVal::Address` for that address (exactly what `Address::to_xdr` returns
//! on-chain). Length prefixes make the encoding unambiguous for both account
//! (`G...`) and contract (`C...`) addresses.
//!
//! | Field           | Encoding                                              |
//! |-----------------|-------------------------------------------------------|
//! | domain tag      | `u8 length \|\| "qrypton-vault/withdraw/v1"`          |
//! | network id      | 32 bytes, `SHA-256(network passphrase)`               |
//! | vault           | `addr` — the vault contract address                   |
//! | to              | `addr` — the recipient                                |
//! | token           | `addr` — the Stellar Asset / SEP-41 token contract    |
//! | amount          | `i128`, 16 bytes, two's complement                    |
//! | nonce           | `u64`, 8 bytes                                        |
//! | expires_at      | `u64`, 8 bytes, ledger timestamp (UNIX seconds)       |
//!
//! # Governance messages
//!
//! Governance actions are approved over the same envelope, with an
//! action-specific payload and their own domain tag. Like withdrawals, every
//! governance message is bound to the network, the vault, a single-use nonce
//! and an expiry.
//!
//! | Field           | Encoding                                              |
//! |-----------------|-------------------------------------------------------|
//! | domain tag      | `u8 length \|\| tag` (see [`DomainTag`])             |
//! | network id      | 32 bytes, `SHA-256(network passphrase)`               |
//! | vault           | `addr` — the vault contract address                   |
//! | payload         | action-specific, see below                            |
//! | nonce           | `u64`, 8 bytes                                        |
//! | expires_at      | `u64`, 8 bytes, ledger timestamp (UNIX seconds)       |
//!
//! | Action                  | Tag                                         | Payload                                        |
//! |-------------------------|---------------------------------------------|------------------------------------------------|
//! | propose operator change | `qrypton-vault/set-operator/v1`             | `addr` — the new operator                      |
//! | propose key rotation    | `qrypton-vault/rotate-keys/v1`              | 32 bytes, SHA-256 of the new signer set and threshold |
//! | set policy              | `qrypton-vault/set-policy/v1`               | 32 bytes, SHA-256 of the XDR-encoded policy    |
//! | cancel operator change  | `qrypton-vault/cancel-operator-change/v1`   | 32 bytes, hash of the pending proposal         |
//! | cancel key rotation     | `qrypton-vault/cancel-key-rotation/v1`      | 32 bytes, hash of the pending proposal         |
//! | cancel policy change    | `qrypton-vault/cancel-policy-change/v1`     | 32 bytes, hash of the pending proposal         |
//!
//! Cancellations commit to the hash of the specific pending proposal, so an
//! approval to cancel one proposal cannot cancel a later one.
//!
//! Nonces are single-use across all actions of a vault: a nonce consumed by a
//! withdrawal cannot be reused by a governance action and vice versa.
//!
//! # Canonical serialisation
//!
//! The encoding is fixed-order, fixed-width for scalars and length-prefixed for
//! variable fields, with no optional fields and no padding, so there is exactly
//! one valid byte string per request. The contract rebuilds the message from the
//! request it receives and never accepts a pre-built message from the caller.
//! On-chain and off-chain encoders must produce byte-identical output; this is
//! enforced by shared test vectors (see `tests/vectors.rs`).
//!
//! The network id is used rather than the raw passphrase because it is what the
//! Soroban environment exposes (`env.ledger().network_id()`) and what Stellar
//! itself uses to bind transaction signatures to a network.
//!
//! The FIPS 204 `ctx` parameter is currently left empty; domain separation is
//! carried inside the message so it is independent of signer/HSM support for
//! `ctx`. Whether to additionally set `ctx` is an open design question.

/// Domain-separation tags. Each signed action has its own tag and version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomainTag {
    /// Approval of a withdrawal from the vault.
    WithdrawV1,
    /// Approval of a proposed operator change (time-locked).
    SetOperatorV1,
    /// Approval of a proposed signer-set rotation (time-locked).
    RotateKeysV1,
    /// Approval of a policy change.
    SetPolicyV1,
    /// Approval of cancelling a pending operator change.
    CancelOperatorChangeV1,
    /// Approval of cancelling a pending signer-set rotation.
    CancelKeyRotationV1,
    /// Approval of cancelling a pending (loosening) policy change.
    CancelPolicyChangeV1,
}

impl DomainTag {
    /// The tag bytes, without the length prefix.
    pub const fn as_bytes(self) -> &'static [u8] {
        match self {
            DomainTag::WithdrawV1 => b"qrypton-vault/withdraw/v1",
            DomainTag::SetOperatorV1 => b"qrypton-vault/set-operator/v1",
            DomainTag::RotateKeysV1 => b"qrypton-vault/rotate-keys/v1",
            DomainTag::SetPolicyV1 => b"qrypton-vault/set-policy/v1",
            DomainTag::CancelOperatorChangeV1 => b"qrypton-vault/cancel-operator-change/v1",
            DomainTag::CancelKeyRotationV1 => b"qrypton-vault/cancel-key-rotation/v1",
            DomainTag::CancelPolicyChangeV1 => b"qrypton-vault/cancel-policy-change/v1",
        }
    }
}

/// Withdrawal domain tag, exposed as a constant for the contract.
pub const WITHDRAW_V1_TAG: &[u8] = DomainTag::WithdrawV1.as_bytes();

/// Length of the network id (SHA-256 output).
pub const NETWORK_ID_LEN: usize = 32;

/// Errors produced while encoding or decoding a signed message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageError {
    /// An address field is not a valid XDR `ScVal::Address`.
    InvalidAddress,
    /// The input ended before all fields were read.
    Truncated,
    /// The domain tag does not match the expected action.
    WrongDomain,
    /// Bytes remained after the last field.
    TrailingBytes,
}

impl core::fmt::Display for MessageError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            MessageError::InvalidAddress => "invalid address encoding",
            MessageError::Truncated => "message truncated",
            MessageError::WrongDomain => "unexpected domain tag",
            MessageError::TrailingBytes => "trailing bytes after message",
        };
        f.write_str(s)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MessageError {}

/// Byte writer for the canonical encoding.
#[cfg(feature = "alloc")]
#[derive(Default)]
pub(crate) struct Writer {
    out: alloc::vec::Vec<u8>,
}

#[cfg(feature = "alloc")]
impl Writer {
    /// `u8 length || tag`.
    pub(crate) fn tag(&mut self, tag: DomainTag) {
        let tag = tag.as_bytes();
        self.out.push(tag.len() as u8);
        self.out.extend_from_slice(tag);
    }

    /// `u32 length || XDR(ScVal::Address)`.
    pub(crate) fn addr(&mut self, addr: &crate::request::AddressBytes) {
        self.out
            .extend_from_slice(&(addr.0.len() as u32).to_be_bytes());
        self.out.extend_from_slice(&addr.0);
    }

    pub(crate) fn raw(&mut self, bytes: &[u8]) {
        self.out.extend_from_slice(bytes);
    }

    pub(crate) fn finish(self) -> alloc::vec::Vec<u8> {
        self.out
    }
}

/// Byte reader for the canonical encoding.
#[cfg(feature = "alloc")]
pub(crate) struct Reader<'a> {
    rest: &'a [u8],
}

#[cfg(feature = "alloc")]
impl<'a> Reader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { rest: bytes }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], MessageError> {
        if self.rest.len() < n {
            return Err(MessageError::Truncated);
        }
        let (head, tail) = self.rest.split_at(n);
        self.rest = tail;
        Ok(head)
    }

    pub(crate) fn array<const N: usize>(&mut self) -> Result<[u8; N], MessageError> {
        let mut out = [0u8; N];
        out.copy_from_slice(self.take(N)?);
        Ok(out)
    }

    pub(crate) fn expect_tag(&mut self, tag: DomainTag) -> Result<(), MessageError> {
        let len = self.array::<1>()?[0] as usize;
        if self.take(len)? != tag.as_bytes() {
            return Err(MessageError::WrongDomain);
        }
        Ok(())
    }

    pub(crate) fn addr(&mut self) -> Result<crate::request::AddressBytes, MessageError> {
        let len = u32::from_be_bytes(self.array::<4>()?) as usize;
        if len == 0 {
            return Err(MessageError::InvalidAddress);
        }
        Ok(crate::request::AddressBytes(self.take(len)?.to_vec()))
    }

    pub(crate) fn finish(self) -> Result<(), MessageError> {
        if self.rest.is_empty() {
            Ok(())
        } else {
            Err(MessageError::TrailingBytes)
        }
    }
}
