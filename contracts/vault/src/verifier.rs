//! ML-DSA signature verification behind a trait.
//!
//! The vault never implements ML-DSA itself. Verification is delegated to the
//! Soroban host via [`NativeHostVerifier`]; [`MockVerifier`] exists only for
//! unit tests of the vault's control flow.

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

/// Verification through the native Soroban host functions introduced by
/// Stage 1 of Stellar's Quantum Preparedness Plan (CAP-0087, Protocol 30).
///
/// **The host API is not final on the network yet.** At the time of writing:
///
/// * CAP-0087 specifies `verify_sig_ml_dsa_{44,65,87}(public_key, msg,
///   signature, context)` returning `Void` and trapping on failure; final cost
///   calibration is still marked TBD.
/// * Protocol 30 is not yet enabled on testnet or mainnet (validator votes
///   are scheduled but not final).
/// * The latest stable `soroban-sdk` (29.0.1, pinned in this workspace) has no
///   ML-DSA functions; `soroban-sdk` 30.0.0-rc.1 exposes
///   `env.crypto().ml_dsa_{44,65,87}_verify(public_key, message, signature, context)`
///   with fixed-size `BytesN` keys/signatures and a `context: &Bytes` (≤ 255
///   bytes), and requires stellar-cli 30+, which is not released yet.
///
/// Because the host traps rather than returning `false`, an invalid signature
/// fails the whole invocation; `VerifyError::InvalidSignature` is therefore
/// unreachable with this verifier. The implementation will be written against
/// the released 30.x SDK once Protocol 30 is live on testnet.
pub struct NativeHostVerifier;

impl PqVerifier for NativeHostVerifier {
    fn verify(
        env: &Env,
        key: &PqPublicKey,
        message: &Bytes,
        signature: &PqSignature,
    ) -> Result<(), VerifyError> {
        // 1. Match (key, signature) variants; mismatched levels → AlgorithmMismatch.
        // 2. Call the corresponding host function with an empty context.
        todo!("bind to the native ML-DSA host function once released")
    }
}

/// Test-only verifier. Never compiled into the deployed contract.
///
/// Intended behaviour: accept a signature iff it was produced by the matching
/// test helper for `(key, message)`, so control-flow tests can exercise
/// thresholds, duplicates and mismatches without real ML-DSA.
#[cfg(test)]
pub struct MockVerifier;

#[cfg(test)]
impl PqVerifier for MockVerifier {
    fn verify(
        env: &Env,
        key: &PqPublicKey,
        message: &Bytes,
        signature: &PqSignature,
    ) -> Result<(), VerifyError> {
        todo!()
    }
}
