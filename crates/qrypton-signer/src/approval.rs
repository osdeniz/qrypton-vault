//! The approval flow.
//!
//! An approval is produced in four steps, and a signer never signs bytes it
//! has not decoded and shown to the operator:
//!
//! 1. **Display** — present network, vault, recipient, token, amount, nonce and
//!    expiry in human-readable form ([`Confirm`]).
//! 2. **Pre-check** — refuse expired requests and, when the vault policy is
//!    known, requests the policy would reject. The contract re-checks
//!    everything; this step only avoids signing requests that would fail.
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
    /// The vault's current policy, if known. Reading it from the chain is SDK
    /// work (M3); without it only the expiry is pre-checked.
    pub policy: Option<&'a Policy>,
    /// This signer's index in the vault's current signer set.
    pub signer_index: u32,
    /// Current time (UNIX seconds) used for the expiry pre-check.
    pub now: u64,
}

/// Runs the full approval flow for one withdrawal message.
pub fn approve_withdraw(
    backend: &dyn KeyBackend,
    confirm: &dyn Confirm,
    ctx: &ApprovalContext<'_>,
    message: &WithdrawMessage,
) -> Result<PqApproval, SignerError> {
    if !confirm.confirm(message)? {
        return Err(SignerError::Declined);
    }
    precheck_policy(ctx, message)?;
    let signature = backend.sign(&message.to_bytes())?;
    Ok(PqApproval {
        signer_index: ctx.signer_index,
        signature,
    })
}

/// Off-chain mirror of the contract's expiry and allowlist checks.
pub fn precheck_policy(
    ctx: &ApprovalContext<'_>,
    message: &WithdrawMessage,
) -> Result<(), SignerError> {
    let request = &message.request;
    if request.amount <= 0 {
        return Err(SignerError::PolicyViolation(
            "amount must be positive".into(),
        ));
    }
    if ctx.now > request.expires_at {
        return Err(SignerError::PolicyViolation("request has expired".into()));
    }
    if let Some(policy) = ctx.policy {
        if !policy.open_recipients && !policy.allowlist.contains(&request.to) {
            return Err(SignerError::PolicyViolation(
                "recipient is not on the vault allowlist".into(),
            ));
        }
    }
    Ok(())
}
