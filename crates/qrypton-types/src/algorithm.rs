//! Post-quantum signature algorithm identifiers.
//!
//! Qrypton Vault uses ML-DSA as standardised in NIST FIPS 204. Sizes below are
//! the FIPS 204 encoded sizes and must match what the Soroban host expects.

/// ML-DSA parameter set (NIST FIPS 204).
///
/// ML-DSA-65 is the default for vault signers. ML-DSA-87 is available for
/// institutions that require NIST security category 5. ML-DSA-44 is listed for
/// completeness and interoperability; it is not recommended for custody keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(u32)]
pub enum MlDsaLevel {
    /// NIST security category 2.
    MlDsa44 = 44,
    /// NIST security category 3. Default.
    #[default]
    MlDsa65 = 65,
    /// NIST security category 5.
    MlDsa87 = 87,
}

impl MlDsaLevel {
    /// Encoded public (verifying) key length in bytes.
    pub const fn public_key_len(self) -> usize {
        match self {
            MlDsaLevel::MlDsa44 => 1312,
            MlDsaLevel::MlDsa65 => 1952,
            MlDsaLevel::MlDsa87 => 2592,
        }
    }

    /// Encoded signature length in bytes.
    pub const fn signature_len(self) -> usize {
        match self {
            MlDsaLevel::MlDsa44 => 2420,
            MlDsaLevel::MlDsa65 => 3309,
            MlDsaLevel::MlDsa87 => 4627,
        }
    }

    /// Stable lowercase name, e.g. `"ml-dsa-65"`. Used in CLI flags and key files.
    pub const fn name(self) -> &'static str {
        match self {
            MlDsaLevel::MlDsa44 => "ml-dsa-44",
            MlDsaLevel::MlDsa65 => "ml-dsa-65",
            MlDsaLevel::MlDsa87 => "ml-dsa-87",
        }
    }
}
