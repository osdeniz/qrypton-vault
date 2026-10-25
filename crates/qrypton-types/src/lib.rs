//! Shared types for Qrypton Vault.
//!
//! This crate is the single source of truth for:
//!
//! * the post-quantum algorithm identifiers and their encoded sizes ([`algorithm`]),
//! * the canonical byte format of every message a PQ signer approves ([`message`]),
//! * the off-chain representation of withdrawal requests, approvals and policy
//!   ([`request`], [`governance`], [`approval`], [`policy`]).
//!
//! The crate is `no_std` so the Soroban contract can share constants and the
//! message layout with off-chain code. Owned types require the `alloc` feature.
//!
//! Status: pre-alpha. Types and layouts are defined; encoders are not implemented.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod algorithm;
pub mod message;

#[cfg(feature = "alloc")]
pub mod approval;
#[cfg(feature = "alloc")]
pub mod governance;
#[cfg(feature = "alloc")]
pub mod policy;
#[cfg(feature = "alloc")]
pub mod request;

pub use algorithm::MlDsaLevel;
pub use message::{DomainTag, MessageError};

#[cfg(feature = "alloc")]
pub use approval::{PqApproval, PqPublicKey, PqSignature};
#[cfg(feature = "alloc")]
pub use governance::{GovernanceAction, GovernanceMessage};
#[cfg(feature = "alloc")]
pub use policy::{Policy, TokenLimit};
#[cfg(feature = "alloc")]
pub use request::{AddressBytes, WithdrawMessage, WithdrawRequest};
