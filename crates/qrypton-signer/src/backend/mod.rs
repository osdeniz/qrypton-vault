//! Key backends: where an approver's ML-DSA private key lives and is used.

mod pkcs11;
mod software;

pub use pkcs11::{Pkcs11Config, Pkcs11Hsm};
pub use software::SoftwareKeystore;

use qrypton_types::{MlDsaLevel, PqPublicKey, PqSignature};

use crate::SignerError;

/// A source of ML-DSA signatures for one key.
///
/// Implementations never expose private key material. `sign` must produce a
/// pure (non-pre-hashed) FIPS 204 ML-DSA signature with an empty context over
/// exactly the bytes given.
pub trait KeyBackend {
    /// The parameter set of the key. ML-DSA-65 by default; ML-DSA-87 optional.
    fn algorithm(&self) -> MlDsaLevel;

    /// The encoded public key, as registered in the vault's signer set.
    fn public_key(&self) -> Result<PqPublicKey, SignerError>;

    /// Signs `message`, which must be a canonical Qrypton Vault message.
    fn sign(&self, message: &[u8]) -> Result<PqSignature, SignerError>;
}
