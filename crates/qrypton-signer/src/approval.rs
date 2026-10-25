//! The approval flow.
//!
//! An approval is produced in four steps, and a signer never signs bytes it
//! has not decoded and shown to the operator:
//!
//! 1. **Display** — decode the canonical message and present network, vault,
//!    recipient, token, amount, nonce and expiry in human-readable form.
//! 2. **Policy pre-check** — check the request against the vault's current
//!    policy (limits, allowlist, expiry) so obviously invalid requests are
//!    refused before any signature exists. The contract re-checks everything;
//!    this step only avoids signing requests that would be rejected.
//! 3. **Sign** — sign the canonical bytes with the [`KeyBackend`].
//! 4. **Approval** — wrap the signature with the signer's index into a
//!    [`PqApproval`].

use qrypton_types::{Policy, PqApproval, WithdrawMessage};

use crate::{KeyBackend, SignerError};

/// Operator confirmation hook (terminal prompt, approval UI, ticketing system).
pub trait Confirm {
    /// Shows the decoded request and returns `true` only on explicit approval.
    fn confirm(&self, message: &WithdrawMessage) -> Result<bool, SignerError>;
}

/// What the signer knows about the vault when approving.
pub struct ApprovalContext<'a> {
    /// The vault's current policy, as read from the chain.
    pub policy: &'a Policy,
    /// This signer's index in the vault's current signer set.
    pub signer_index: u32,
    /// Current ledger timestamp used for the expiry pre-check.
    pub now: u64,
}

/// Runs the full approval flow for one withdrawal message.
pub fn approve_withdraw(
    backend: &dyn KeyBackend,
    confirm: &dyn Confirm,
    ctx: &ApprovalContext<'_>,
    message: &WithdrawMessage,
) -> Result<PqApproval, SignerError> {
    // 1. confirm.confirm(message)        → Declined if false
    // 2. precheck_policy(ctx, message)   → PolicyViolation
    // 3. backend.sign(&message.to_bytes())
    // 4. PqApproval { signer_index, signature }
    todo!()
}

/// Off-chain mirror of the contract's expiry and policy checks.
pub fn precheck_policy(
    ctx: &ApprovalContext<'_>,
    message: &WithdrawMessage,
) -> Result<(), SignerError> {
    todo!()
}
