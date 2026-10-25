use std::path::PathBuf;

use qrypton_types::{MlDsaLevel, PqPublicKey, PqSignature};

use super::KeyBackend;
use crate::SignerError;

/// Connection settings for a PKCS#11 token.
#[derive(Clone, Debug)]
pub struct Pkcs11Config {
    /// Path to the vendor PKCS#11 module (shared library).
    pub module: PathBuf,
    /// Token label identifying the slot.
    pub token_label: String,
    /// Label of the ML-DSA key object on the token.
    pub key_label: String,
}

/// Production backend: the ML-DSA private key is generated and used inside an
/// HSM and never leaves it. Requires an HSM whose firmware supports ML-DSA
/// through PKCS#11.
pub struct Pkcs11Hsm {
    config: Pkcs11Config,
    level: MlDsaLevel,
}

impl Pkcs11Hsm {
    /// Opens a session to the token and locates the key. The user PIN is
    /// requested interactively and is never stored.
    pub fn connect(config: Pkcs11Config) -> Result<Self, SignerError> {
        todo!()
    }

    /// The configuration this backend was opened with.
    pub fn config(&self) -> &Pkcs11Config {
        &self.config
    }
}

impl KeyBackend for Pkcs11Hsm {
    fn algorithm(&self) -> MlDsaLevel {
        self.level
    }

    fn public_key(&self) -> Result<PqPublicKey, SignerError> {
        todo!()
    }

    fn sign(&self, message: &[u8]) -> Result<PqSignature, SignerError> {
        todo!()
    }
}
