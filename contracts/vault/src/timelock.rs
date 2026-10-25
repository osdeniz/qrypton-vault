//! Shared time-lock mechanism: **propose → wait → execute**, or **cancel**.
//!
//! Signer-set rotation, operator change and loosening policy changes are all
//! implemented on top of this one module instead of three bespoke copies.
//! Each kind of proposal has at most one pending instance. Authorization (which
//! approvals a proposal or cancellation needs) is checked by the caller in
//! `lib.rs`; this module only stores, times and releases proposals.
//!
//! Every pending proposal is identified by the SHA-256 hash of its payload.
//! Cancellation approvals commit to that hash, so an approval to cancel one
//! proposal can never cancel a later one.
//!
//! Wait durations are deployment parameters and are not documented here.

// Pre-alpha skeleton: callers in `lib.rs` are still `todo!()`.
#![allow(dead_code)]

use soroban_sdk::{contracttype, BytesN, Env, IntoVal, TryFromVal, Val};

use crate::Error;

/// The kinds of time-locked proposals.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProposalKind {
    /// Replace the operator (quorum only).
    OperatorChange = 1,
    /// Replace the PQ signer set and threshold (operator + quorum).
    KeyRotation = 2,
    /// Loosen the withdrawal policy (operator + quorum).
    PolicyChange = 3,
}

/// Bookkeeping stored next to every pending proposal, whatever its kind.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalMeta {
    /// SHA-256 of the proposal payload; cancellations must reference it.
    pub proposal_hash: BytesN<32>,
    /// Ledger timestamp from which the proposal can be executed.
    pub executable_at: u64,
}

/// Stores `payload` as the pending proposal of `kind`, executable after the
/// kind's wait duration.
///
/// Fails with the kind's "already pending" error if one is pending.
pub(crate) fn propose<T>(env: &Env, kind: ProposalKind, payload: &T) -> Result<ProposalMeta, Error>
where
    T: IntoVal<Env, Val>,
{
    todo!()
}

/// Removes and returns the pending proposal of `kind` if its wait has elapsed.
///
/// Fails with the kind's "no pending" or "time-locked" error otherwise.
pub(crate) fn take_executable<T>(env: &Env, kind: ProposalKind) -> Result<(T, ProposalMeta), Error>
where
    T: TryFromVal<Env, Val>,
{
    todo!()
}

/// Removes the pending proposal of `kind` if its hash equals `proposal_hash`.
pub(crate) fn cancel(
    env: &Env,
    kind: ProposalKind,
    proposal_hash: &BytesN<32>,
) -> Result<ProposalMeta, Error> {
    todo!()
}

/// Called when a key rotation executes: drops any pending operator change and
/// policy change, since they were approved by the previous signer set.
/// Returns the metadata of what was dropped so the caller can emit events.
pub(crate) fn cancel_on_key_rotation(env: &Env) -> (Option<ProposalMeta>, Option<ProposalMeta>) {
    todo!()
}

/// The pending proposal of `kind`, if any.
pub(crate) fn pending<T>(env: &Env, kind: ProposalKind) -> Option<(T, ProposalMeta)>
where
    T: TryFromVal<Env, Val>,
{
    todo!()
}

/// Wait duration for `kind`, in seconds (deployment parameter).
fn delay(env: &Env, kind: ProposalKind) -> u64 {
    todo!()
}
