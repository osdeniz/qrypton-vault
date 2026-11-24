use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use fips204::traits::{KeyGen, SerDes, Signer};
use qrypton_types::{MlDsaLevel, PqPublicKey, PqSignature};
use rand_core::{OsRng, RngCore};
use zeroize::Zeroizing;

use super::KeyBackend;
use crate::SignerError;

const HEADER: &str = "qrypton-keystore v1";
const WARNING: &str =
    "# UNENCRYPTED DEVELOPMENT KEY. Do not use for real funds. Encryption at rest arrives in M2.";

/// File-based keystore for development and testnet use only.
///
/// **Unencrypted.** The file holds the 32-byte FIPS 204 key-generation seed
/// `ξ` in hex and is created with mode `0600`. Encryption at rest under an
/// operator passphrase arrives in M2; production keys belong in an HSM
/// ([`super::Pkcs11Hsm`]).
///
/// Signing uses the `fips204` crate: pure ML-DSA (`ML-DSA.Sign` with an empty
/// context), not HashML-DSA and not pre-standard Dilithium.
pub struct SoftwareKeystore {
    path: Option<PathBuf>,
    level: MlDsaLevel,
    seed: Zeroizing<[u8; 32]>,
}

impl SoftwareKeystore {
    /// Generates a new key of the given parameter set and writes it to `path`.
    /// Fails if `path` already exists.
    pub fn generate(path: &Path, level: MlDsaLevel) -> Result<Self, SignerError> {
        let mut seed = Zeroizing::new([0u8; 32]);
        OsRng
            .try_fill_bytes(seed.as_mut())
            .map_err(|e| SignerError::Backend(format!("OS RNG failed: {e}")))?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(path)?;
        let contents = Zeroizing::new(format!(
            "{HEADER}\n{WARNING}\nalgorithm: {}\nseed: {}\n",
            level.name(),
            hex::encode(seed.as_ref())
        ));
        if let Err(e) = file
            .write_all(contents.as_bytes())
            .and_then(|()| file.sync_all())
        {
            // Do not leave a truncated keystore behind that blocks a retry.
            drop(file);
            let _ = std::fs::remove_file(path);
            return Err(e.into());
        }
        Ok(Self {
            path: Some(path.to_path_buf()),
            level,
            seed,
        })
    }

    /// Opens an existing keystore file.
    ///
    /// On Unix, refuses a file that group or other users can access (like
    /// ssh does for private keys): the seed is stored unencrypted.
    pub fn open(path: &Path) -> Result<Self, SignerError> {
        // One handle for both the permission check and the read, so the file
        // cannot be swapped in between.
        let mut file = File::open(path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = file.metadata()?.permissions().mode() & 0o777;
            if mode & 0o077 != 0 {
                return Err(SignerError::Backend(format!(
                    "keystore {} is accessible by other users (mode {mode:o}); run `chmod 600` on it \
                     (on file systems without Unix permissions, keep the keystore elsewhere)",
                    path.display()
                )));
            }
        }
        let mut contents = Zeroizing::new(String::new());
        file.read_to_string(&mut contents)?;
        let mut lines = contents.lines();
        if lines.next() != Some(HEADER) {
            return Err(SignerError::Backend(
                "not a qrypton keystore v1 file".into(),
            ));
        }
        let mut level = None;
        let mut seed = None;
        for line in lines {
            if let Some(v) = line.strip_prefix("algorithm: ") {
                level = Some(parse_level(v)?);
            } else if let Some(v) = line.strip_prefix("seed: ") {
                let mut s = Zeroizing::new([0u8; 32]);
                hex::decode_to_slice(v.trim(), s.as_mut())
                    .map_err(|_| SignerError::Backend("malformed seed".into()))?;
                seed = Some(s);
            }
        }
        match (level, seed) {
            (Some(level), Some(seed)) => Ok(Self {
                path: Some(path.to_path_buf()),
                level,
                seed,
            }),
            _ => Err(SignerError::Backend("keystore is missing fields".into())),
        }
    }

    /// In-memory key from a seed. For tests; never written to disk.
    pub fn from_seed(level: MlDsaLevel, seed: [u8; 32]) -> Self {
        Self {
            path: None,
            level,
            seed: Zeroizing::new(seed),
        }
    }

    /// Location of the keystore file, if any.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

/// Parses a parameter-set name as written by [`MlDsaLevel::name`].
pub fn parse_level(name: &str) -> Result<MlDsaLevel, SignerError> {
    match name.trim() {
        "ml-dsa-44" => Ok(MlDsaLevel::MlDsa44),
        "ml-dsa-65" => Ok(MlDsaLevel::MlDsa65),
        "ml-dsa-87" => Ok(MlDsaLevel::MlDsa87),
        _ => Err(SignerError::UnsupportedAlgorithm),
    }
}

macro_rules! with_level {
    ($level:expr, $m:ident => $body:expr) => {
        match $level {
            MlDsaLevel::MlDsa44 => {
                use fips204::ml_dsa_44 as $m;
                $body
            }
            MlDsaLevel::MlDsa65 => {
                use fips204::ml_dsa_65 as $m;
                $body
            }
            MlDsaLevel::MlDsa87 => {
                use fips204::ml_dsa_87 as $m;
                $body
            }
        }
    };
}

impl KeyBackend for SoftwareKeystore {
    fn algorithm(&self) -> MlDsaLevel {
        self.level
    }

    fn public_key(&self) -> Result<PqPublicKey, SignerError> {
        let bytes = with_level!(self.level, m => {
            let (pk, _sk) = m::KG::keygen_from_seed(&self.seed);
            pk.into_bytes().to_vec()
        });
        Ok(PqPublicKey {
            level: self.level,
            bytes,
        })
    }

    fn sign(&self, message: &[u8]) -> Result<PqSignature, SignerError> {
        // Pure ML-DSA with an empty context string (FIPS 204 Algorithm 2).
        let bytes = with_level!(self.level, m => {
            let (_pk, sk) = m::KG::keygen_from_seed(&self.seed);
            sk.try_sign(message, &[])
                .map_err(|e| SignerError::Backend(e.into()))?
                .to_vec()
        });
        Ok(PqSignature {
            level: self.level,
            bytes,
        })
    }
}
