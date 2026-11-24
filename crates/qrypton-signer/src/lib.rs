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
//! backend: in development, an established open-source Rust ML-DSA library
//! that has not yet been independently audited; in production, the HSM
//! firmware via PKCS#11.
//!
//! Status: pre-alpha. The software keystore, approval flow and CLI work; the
//! PKCS#11 backend is still `todo!()`.

// The PKCS#11 backend is still `todo!()`, so its parameters are unused.
// Remove once it lands.
#![allow(unused_variables)]

pub mod approval;
pub mod backend;
mod error;

pub use approval::{approve_withdraw, precheck_policy, ApprovalContext, Confirm};
pub use backend::KeyBackend;
pub use error::SignerError;
