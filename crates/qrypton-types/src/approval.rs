//! Post-quantum keys, signatures and approvals (off-chain representation).

use alloc::vec::Vec;

use crate::algorithm::MlDsaLevel;

/// An encoded ML-DSA public key together with its parameter set.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PqPublicKey {
    pub level: MlDsaLevel,
    /// FIPS 204 encoded verifying key; length is `level.public_key_len()`.
    pub bytes: Vec<u8>,
}

/// An encoded ML-DSA signature together with its parameter set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PqSignature {
    pub level: MlDsaLevel,
    /// FIPS 204 encoded signature; length is `level.signature_len()`.
    pub bytes: Vec<u8>,
}

/// One signer's approval of a message.
///
/// `signer_index` refers to the position of the signer's key in the vault's
/// current signer set. The contract checks that indices are unique and that
/// the signature's level matches the registered key's level.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PqApproval {
    pub signer_index: u32,
    pub signature: PqSignature,
}
