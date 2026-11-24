//! ML-DSA signature verification behind a trait.
//!
//! The vault never implements ML-DSA itself. Verification is delegated to the
//! Soroban host via [`NativeHostVerifier`]. Unit tests use the same native host
//! functions through `soroban-sdk` testutils, not a mock.

use soroban_sdk::{Bytes, Env};

use crate::types::{PqPublicKey, PqSignature};

/// Verification failures that a verifier can report without trapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerifyError {
    /// Key and signature use different ML-DSA parameter sets.
    AlgorithmMismatch,
    /// The signature does not verify.
    InvalidSignature,
}

/// Verifies an ML-DSA signature over `message` under `key`.
///
/// Implementations must use the pure (non-pre-hashed) `ML-DSA.Verify` of
/// NIST FIPS 204 with an empty context string; domain separation is carried
/// inside `message` (see `qrypton_types::message`).
pub trait PqVerifier {
    fn verify(
        env: &Env,
        key: &PqPublicKey,
        message: &Bytes,
        signature: &PqSignature,
    ) -> Result<(), VerifyError>;
}

/// Verification through the native Soroban host functions of CAP-0087
/// (Protocol 30): `env.crypto().ml_dsa_{44,65,87}_verify(public_key, message,
/// signature, context)`, with the context left empty.
///
/// The host **traps** on an invalid or malformed signature instead of
/// returning `false`, so an invalid signature fails the whole invocation and
/// [`VerifyError::InvalidSignature`] is never returned by this verifier.
pub struct NativeHostVerifier;

impl PqVerifier for NativeHostVerifier {
    fn verify(
        env: &Env,
        key: &PqPublicKey,
        message: &Bytes,
        signature: &PqSignature,
    ) -> Result<(), VerifyError> {
        let ctx = Bytes::new(env);
        let crypto = env.crypto();
        match (key, signature) {
            (PqPublicKey::MlDsa44(pk), PqSignature::MlDsa44(sig)) => {
                crypto.ml_dsa_44_verify(pk, message, sig, &ctx)
            }
            (PqPublicKey::MlDsa65(pk), PqSignature::MlDsa65(sig)) => {
                crypto.ml_dsa_65_verify(pk, message, sig, &ctx)
            }
            (PqPublicKey::MlDsa87(pk), PqSignature::MlDsa87(sig)) => {
                crypto.ml_dsa_87_verify(pk, message, sig, &ctx)
            }
            _ => return Err(VerifyError::AlgorithmMismatch),
        }
        Ok(())
    }
}
