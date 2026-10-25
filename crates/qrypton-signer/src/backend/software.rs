use std::path::{Path, PathBuf};

use qrypton_types::{MlDsaLevel, PqPublicKey, PqSignature};

use super::KeyBackend;
use crate::SignerError;

/// File-based keystore for development and testnet use only.
///
/// The private key is stored encrypted at rest under an operator passphrase.
/// Not intended for production custody; use [`super::Pkcs11Hsm`] instead.
pub struct SoftwareKeystore {
    path: PathBuf,
    level: MlDsaLevel,
}

impl SoftwareKeystore {
    /// Generates a new key of the given parameter set and writes it to `path`.
    /// Fails if `path` already exists.
    pub fn generate(path: &Path, level: MlDsaLevel) -> Result<Self, SignerError> {
        todo!()
    }

    /// Opens an existing keystore file. The passphrase is requested when the
    /// key is first used.
    pub fn open(path: &Path) -> Result<Self, SignerError> {
        todo!()
    }

    /// Location of the keystore file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl KeyBackend for SoftwareKeystore {
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
