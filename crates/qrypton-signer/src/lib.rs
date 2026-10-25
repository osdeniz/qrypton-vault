//! # qrypton-signer
//!
//! The approver side of Qrypton Vault. A signer holds one ML-DSA key and turns
//! a withdrawal request into a [`PqApproval`](qrypton_types::PqApproval) only
//! after showing the request to the operator and pre-checking it against the
//! vault policy.
//!
//! * [`backend`] — where the private key lives: [`backend::SoftwareKeystore`]
//!   for development, [`backend::Pkcs11Hsm`] for production.
//! * [`approval`] — the approval flow: display → policy pre-check → sign →
//!   approval.
//!
//! This crate does not implement ML-DSA. Signing is delegated to the key
//! backend: an audited ML-DSA library for the software keystore, the HSM
//! firmware for PKCS#11.
//!
//! Status: pre-alpha. Interfaces are defined; bodies are `todo!()`.

// Pre-alpha skeleton: bodies are `todo!()`, so parameters are not used yet.
// Remove once the implementation lands.
#![allow(unused_variables)]

pub mod approval;
pub mod backend;
mod error;

pub use approval::{approve_withdraw, ApprovalContext, Confirm};
pub use backend::KeyBackend;
pub use error::SignerError;
