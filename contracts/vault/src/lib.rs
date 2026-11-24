//! # Qrypton Vault contract
//!
//! A Soroban custody vault governed by a post-quantum quorum.
//!
//! **Authority model**
//!
//! * The **PQ quorum** — at least `threshold` of the vault's registered ML-DSA
//!   (NIST FIPS 204) signers — is the root of authority.
//! * The **operator** is a classical Stellar address (`require_auth`; may be a
//!   multisig or contract account) that acts as a second factor for day-to-day
//!   operations: every withdrawal needs both the operator and the PQ quorum.
//! * A stolen operator key alone cannot move funds or change anything.
//! * A stolen quorum alone can only replace the operator, and only after a
//!   public time lock during which the institution can cancel the change or
//!   move funds to a pre-configured rescue vault.
//!
//! | Function                   | Operator | PQ quorum | Time lock |
//! |----------------------------|:--------:|:---------:|:---------:|
//! | `withdraw`                 | ✓        | ✓         |           |
//! | `set_policy` (tightening)  | ✓        | ✓         |           |
//! | `set_policy` (loosening)   | ✓        | ✓         | ✓         |
//! | `cancel_policy_change`     | ✓        | ✓         |           |
//! | `propose_key_rotation`     | ✓        | ✓         | ✓         |
//! | `cancel_key_rotation`      | ✓        | ✓         |           |
//! | `propose_operator_change`  |          | ✓         | ✓         |
//! | `cancel_operator_change`   |          | ✓         |           |
//! | `execute_*`                | anyone, after the time lock      | |
//!
//! On top of that the vault enforces an institutional policy (per-token limits,
//! recipient allowlist, rescue vault; changes that loosen it are time-locked),
//! single-use nonces and request expiry,
//! and emits an audit event for every state change.
//!
//! Key rotation, operator change and loosening policy changes share one
//! propose → wait → execute / cancel mechanism (`timelock` module).
//!
//! There is deliberately **no upgrade entry point**: the deployed code cannot
//! be replaced. Migration to a new version means withdrawing to a new vault
//! under the normal authorization rules.
//!
//! ML-DSA verification sits behind the [`verifier::PqVerifier`] trait and uses
//! the native Soroban host functions of CAP-0087 (Protocol 30).
//!
//! # Status: pre-alpha proof
//!
//! Implemented: constructor, `deposit`, `withdraw` (operator authorization,
//! replay protection, M-of-N ML-DSA, expiry, allowlist) and the basic views.
//!
//! **Not implemented yet in this proof:** token limits (the per-token "deny by
//! default" rule included), the rescue-vault exemption, `set_policy` and every
//! time-locked governance action (key rotation, operator change, policy
//! loosening). Those functions are `todo!()` and `withdraw` calls none of them.
//! Do not use this contract on mainnet.

#![no_std]
// Governance functions are still `todo!()`, so their parameters are unused.
// Remove once governance lands.
#![allow(unused_variables)]

mod error;
mod events;
mod message;
mod storage;
mod timelock;
mod types;
pub mod verifier;

#[cfg(test)]
mod test;

pub use error::Error;
pub use events::{
    Deposited, KeyRotationCancelled, KeyRotationExecuted, KeyRotationProposed,
    OperatorChangeCancelled, OperatorChangeExecuted, OperatorChangeProposed, PolicyChangeCancelled,
    PolicyChangeProposed, PolicyUpdated, WithdrawExecuted, WithdrawRejected,
};
pub use types::{
    CancelCause, GovernanceAuth, PendingOperatorChange, PendingPolicyChange, PendingRotation,
    Policy, PolicyChangeOutcome, PqApproval, PqPublicKey, PqSignature, RejectReason, TokenLimit,
    WithdrawOutcome, WithdrawRequest,
};

use soroban_sdk::{contract, contractimpl, panic_with_error, token, Address, Bytes, Env, Vec};

use verifier::{NativeHostVerifier, PqVerifier, VerifyError};

#[contract]
pub struct QryptonVault;

#[contractimpl]
impl QryptonVault {
    /// Initializes the vault. Runs once, atomically with deployment, so the
    /// vault can never be observed (or front-run) in an uninitialized state.
    ///
    /// * `operator` — classical Stellar address that co-authorizes withdrawals
    ///   and day-to-day governance.
    /// * `pq_signers` — the initial ML-DSA signer set (N keys, no duplicates).
    /// * `threshold` — number of PQ approvals required (M), `1 <= M <= N`.
    /// * `policy` — initial withdrawal policy, including the rescue vault.
    ///
    /// Panics with [`Error::InvalidThreshold`] or [`Error::DuplicateSigner`]
    /// on invalid input. (Policy limits are stored but not validated or
    /// enforced in this proof.)
    pub fn __constructor(
        env: Env,
        operator: Address,
        pq_signers: Vec<PqPublicKey>,
        threshold: u32,
        policy: Policy,
    ) {
        if threshold == 0 || threshold > pq_signers.len() {
            panic_with_error!(&env, Error::InvalidThreshold);
        }
        for i in 0..pq_signers.len() {
            for j in (i + 1)..pq_signers.len() {
                if pq_signers.get_unchecked(i) == pq_signers.get_unchecked(j) {
                    panic_with_error!(&env, Error::DuplicateSigner);
                }
            }
        }
        storage::init(&env, &operator, &pq_signers, threshold, &policy);
    }

    /// Deposits `amount` of `token` from `from` into the vault.
    ///
    /// Requires `from.require_auth()`. Transfers via the token contract's
    /// SEP-41 `transfer` and emits [`Deposited`]. Deposits are not subject to
    /// PQ approval; only outflows are.
    pub fn deposit(env: Env, from: Address, token: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        storage::bump_instance(&env);
        token::TokenClient::new(&env, &token).transfer(
            &from,
            env.current_contract_address(),
            &amount,
        );
        Deposited {
            token,
            from,
            amount,
        }
        .publish(&env);
        Ok(())
    }

    /// Executes a withdrawal approved by at least `threshold` PQ signers.
    ///
    /// Order of checks — nothing changes state, and no event is emitted, until
    /// the PQ quorum has been verified:
    ///
    /// 1. **Operator** — `operator.require_auth()` (the classical signature).
    ///    Non-positive amounts fail with [`Error::InvalidAmount`].
    /// 2. **Replay** — if `request.nonce` was already consumed, fail with
    ///    [`Error::NonceAlreadyUsed`]: no event, no state change, no signature
    ///    verification. Approvals of past withdrawals are public on-chain; if a
    ///    replay were verified first and then recorded as a rejection, a stolen
    ///    operator key could replay them to emit unlimited rejection events
    ///    without the quorum.
    /// 3. **M-of-N ML-DSA** — rebuild the canonical `qrypton-vault/withdraw/v1`
    ///    message from `request`, the network id and this contract's address;
    ///    each approval must reference a distinct signer index of the current
    ///    set (so a key is counted at most once) and verify under the native
    ///    host function; at least `threshold` approvals are required. Any
    ///    failure fails the invocation and changes nothing, so an operator
    ///    without the quorum cannot consume a nonce.
    /// 4. **Expiry and allowlist** — if the ledger timestamp is past
    ///    `request.expires_at`, or the recipient is not allowlisted (closed by
    ///    default) and `open_recipients` is off, the quorum-approved request is
    ///    rejected: the nonce is consumed, [`WithdrawRejected`] is emitted and
    ///    [`WithdrawOutcome::Rejected`] is returned. No funds move.
    /// 5. **Transfer** — consume the nonce, transfer via the token contract,
    ///    emit [`WithdrawExecuted`].
    ///
    /// Not enforced in this proof: token limits and the rescue-vault exemption.
    pub fn withdraw(
        env: Env,
        request: WithdrawRequest,
        approvals: Vec<PqApproval>,
    ) -> Result<WithdrawOutcome, Error> {
        // 1. Operator.
        storage::operator(&env).require_auth();
        if request.amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        // 2. Replay: fail before verification, leaving no trace.
        if storage::is_nonce_used(&env, request.nonce) {
            return Err(Error::NonceAlreadyUsed);
        }

        // 3. M-of-N ML-DSA over the canonical message.
        let msg = message::withdraw_message(&env, &request);
        let signer_indices = verify_quorum::<NativeHostVerifier>(&env, &msg, &approvals)?;

        // From here on the request is approved by the quorum.
        storage::bump_instance(&env);
        storage::consume_nonce(&env, request.nonce);

        // 4. Expiry and allowlist: recorded rejection, no transfer.
        if let Some(reason) = rejection(&env, &request) {
            WithdrawRejected {
                token: request.token,
                to: request.to,
                amount: request.amount,
                nonce: request.nonce,
                reason,
            }
            .publish(&env);
            return Ok(WithdrawOutcome::Rejected(reason));
        }

        // 5. Transfer.
        token::TokenClient::new(&env, &request.token).transfer(
            &env.current_contract_address(),
            &request.to,
            &request.amount,
        );
        WithdrawExecuted {
            token: request.token,
            to: request.to,
            amount: request.amount,
            nonce: request.nonce,
            signer_indices,
        }
        .publish(&env);
        Ok(WithdrawOutcome::Executed)
    }

    // ---------------------------------------------------------------------
    // Operator change — the recovery path (PQ quorum only, time-locked)
    // ---------------------------------------------------------------------

    /// Proposes replacing the operator.
    ///
    /// Requires **only** current-threshold PQ approvals over the
    /// `qrypton-vault/set-operator/v1` message; the current operator's
    /// signature is not required, so a lost or compromised classical key cannot
    /// lock the vault. At most one operator change can be pending
    /// ([`Error::OperatorChangeAlreadyPending`]). The change becomes executable
    /// after a public time lock. Emits [`OperatorChangeProposed`].
    pub fn propose_operator_change(
        env: Env,
        new_operator: Address,
        auth: GovernanceAuth,
    ) -> Result<(), Error> {
        todo!()
    }

    /// Activates the pending operator once its time lock has elapsed.
    ///
    /// Permissionless: anyone may call it, because the change was approved by
    /// the quorum at proposal time and the waiting period has passed. Emits
    /// [`OperatorChangeExecuted`].
    pub fn execute_operator_change(env: Env) -> Result<(), Error> {
        todo!()
    }

    /// Cancels the pending operator change.
    ///
    /// Requires **only** current-threshold PQ approvals over the
    /// `qrypton-vault/cancel-operator-change/v1` message, which commits to the
    /// hash of the pending proposal. The operator cannot cancel on its own, so
    /// a stolen classical key cannot block recovery. Emits
    /// [`OperatorChangeCancelled`] with [`CancelCause::Quorum`].
    pub fn cancel_operator_change(env: Env, auth: GovernanceAuth) -> Result<(), Error> {
        todo!()
    }

    // ---------------------------------------------------------------------
    // Signer-set rotation (operator + PQ quorum, time-locked)
    // ---------------------------------------------------------------------

    /// Proposes replacing the PQ signer set and threshold.
    ///
    /// Requires `operator.require_auth()` and current-threshold PQ approvals
    /// over the `qrypton-vault/rotate-keys/v1` message. At most one rotation can
    /// be pending. The proposal becomes executable only after a time lock,
    /// giving the institution a window to detect and cancel an unauthorized
    /// rotation. Emits [`KeyRotationProposed`].
    pub fn propose_key_rotation(
        env: Env,
        new_signers: Vec<PqPublicKey>,
        new_threshold: u32,
        auth: GovernanceAuth,
    ) -> Result<(), Error> {
        todo!()
    }

    /// Activates the pending signer set once its time lock has elapsed.
    ///
    /// Permissionless: anyone may call it after the time lock, because the
    /// rotation was already approved at proposal time. Emits
    /// [`KeyRotationExecuted`].
    ///
    /// Executing a rotation **cancels any pending operator change and any
    /// pending policy change** (emitting [`OperatorChangeCancelled`] /
    /// [`PolicyChangeCancelled`] with [`CancelCause::KeyRotation`]): those
    /// proposals were approved by keys that may just have been removed, and
    /// must not be executable afterwards.
    pub fn execute_key_rotation(env: Env) -> Result<(), Error> {
        todo!()
    }

    /// Cancels a pending rotation before it executes.
    ///
    /// Requires `operator.require_auth()` and current-threshold PQ approvals
    /// over the `qrypton-vault/cancel-key-rotation/v1` message, which commits
    /// to the hash of the pending proposal. Emits [`KeyRotationCancelled`].
    pub fn cancel_key_rotation(env: Env, auth: GovernanceAuth) -> Result<(), Error> {
        todo!()
    }

    // ---------------------------------------------------------------------
    // Policy (operator + PQ quorum; loosening is time-locked)
    // ---------------------------------------------------------------------

    /// Replaces the withdrawal policy.
    ///
    /// Requires `operator.require_auth()` and current-threshold PQ approvals
    /// over the `qrypton-vault/set-policy/v1` message, which commits to the
    /// hash of the new policy. Requiring the operator here means a stolen
    /// quorum cannot remove the rescue vault or change limits during an
    /// operator-change time lock.
    ///
    /// The new policy is compared with the current one:
    ///
    /// * **Tightening only** — every change lowers a limit, removes a token,
    ///   removes an allowlist entry (removing the last one closes the vault to
    ///   all recipients but the rescue vault) or turns `open_recipients` off,
    ///   and the rescue vault is unchanged. Applied
    ///   immediately; emits [`PolicyUpdated`] and returns
    ///   [`PolicyChangeOutcome::Applied`].
    /// * **Anything else is loosening** — raising a limit, adding a token or
    ///   allowlist entry, turning `open_recipients` on, or setting, changing or
    ///   removing the rescue vault. Scheduled behind a time lock; emits
    ///   [`PolicyChangeProposed`] and returns [`PolicyChangeOutcome::Scheduled`].
    ///   At most one policy change can be pending
    ///   ([`Error::PolicyChangeAlreadyPending`]).
    ///
    /// Without this, an attacker holding both the operator and the quorum could
    /// raise a limit or allowlist their own address and withdraw in the same
    /// minute; with it, loosening is publicly visible before it takes effect.
    pub fn set_policy(
        env: Env,
        policy: Policy,
        auth: GovernanceAuth,
    ) -> Result<PolicyChangeOutcome, Error> {
        todo!()
    }

    /// Applies the pending (loosening) policy change once its time lock has
    /// elapsed.
    ///
    /// Permissionless: the change was approved at proposal time. Emits
    /// [`PolicyUpdated`].
    pub fn execute_policy_change(env: Env) -> Result<(), Error> {
        todo!()
    }

    /// Cancels the pending policy change.
    ///
    /// Requires `operator.require_auth()` and current-threshold PQ approvals
    /// over the `qrypton-vault/cancel-policy-change/v1` message, which commits
    /// to the hash of the pending proposal. Emits [`PolicyChangeCancelled`]
    /// with [`CancelCause::Quorum`].
    pub fn cancel_policy_change(env: Env, auth: GovernanceAuth) -> Result<(), Error> {
        todo!()
    }

    // ---------------------------------------------------------------------
    // Read-only views
    // ---------------------------------------------------------------------

    /// The current operator address.
    pub fn operator(env: Env) -> Address {
        storage::operator(&env)
    }

    /// The current PQ signer set.
    pub fn signers(env: Env) -> Vec<PqPublicKey> {
        storage::signers(&env)
    }

    /// The current approval threshold (M).
    pub fn threshold(env: Env) -> u32 {
        storage::threshold(&env)
    }

    /// The current withdrawal policy.
    pub fn policy(env: Env) -> Policy {
        storage::policy(&env)
    }

    /// The pending operator change, if any.
    pub fn pending_operator_change(env: Env) -> Option<PendingOperatorChange> {
        todo!()
    }

    /// The pending (loosening) policy change, if any.
    pub fn pending_policy_change(env: Env) -> Option<PendingPolicyChange> {
        todo!()
    }

    /// The pending signer-set rotation, if any.
    pub fn pending_rotation(env: Env) -> Option<PendingRotation> {
        todo!()
    }

    /// Whether `nonce` has already been consumed.
    pub fn is_nonce_used(env: Env, nonce: u64) -> bool {
        storage::is_nonce_used(&env, nonce)
    }
}

/// Verifies that `approvals` contain at least `threshold` valid signatures over
/// `msg` from distinct members of the current signer set. Returns the signer
/// indices that were counted.
fn verify_quorum<V: PqVerifier>(
    env: &Env,
    msg: &Bytes,
    approvals: &Vec<PqApproval>,
) -> Result<Vec<u32>, Error> {
    let signers = storage::signers(env);
    let threshold = storage::threshold(env);
    let mut counted: Vec<u32> = Vec::new(env);
    for approval in approvals.iter() {
        let index = approval.signer_index;
        if index >= signers.len() {
            return Err(Error::UnknownSigner);
        }
        if counted.contains(index) {
            return Err(Error::DuplicateApproval);
        }
        let key = signers.get_unchecked(index);
        V::verify(env, &key, msg, &approval.signature).map_err(|e| match e {
            VerifyError::AlgorithmMismatch => Error::AlgorithmMismatch,
            VerifyError::InvalidSignature => Error::InvalidSignature,
        })?;
        counted.push_back(index);
    }
    if counted.len() < threshold {
        return Err(Error::InsufficientApprovals);
    }
    Ok(counted)
}

/// Expiry and allowlist checks for a quorum-approved request.
fn rejection(env: &Env, request: &WithdrawRequest) -> Option<RejectReason> {
    if env.ledger().timestamp() > request.expires_at {
        return Some(RejectReason::Expired);
    }
    let policy = storage::policy(env);
    if !policy.open_recipients && !policy.allowlist.contains(&request.to) {
        return Some(RejectReason::RecipientNotAllowed);
    }
    None
}
