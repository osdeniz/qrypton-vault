//! # qrypton-sdk
//!
//! The integration surface for institutions using Qrypton Vault:
//!
//! 1. [`VaultClient::build_withdraw_request`] — build a request and the exact
//!    canonical message signers will approve.
//! 2. [`collect_approvals`] — gather ML-DSA approvals from M-of-N approvers
//!    (local signers, HSM-backed services, or remote approval endpoints).
//! 3. [`VaultClient::submit_withdraw`] — assemble, simulate and submit the
//!    Soroban transaction through Stellar RPC.
//! 4. [`VaultClient::watch_events`] — follow the vault's audit events for
//!    reconciliation and alerting.
//!
//! Status: pre-alpha. Signatures and documentation are final candidates;
//! bodies are `todo!()`.

// Pre-alpha skeleton: bodies are `todo!()`, so parameters are not used yet.
// Remove once the implementation lands.
#![allow(unused_variables)]

mod error;
mod events;

pub use error::SdkError;
pub use events::{EventCursor, EventSubscription, VaultEvent};

use qrypton_types::{PqApproval, WithdrawMessage, WithdrawRequest};

/// Stellar network the vault is deployed on.
#[derive(Clone, Debug)]
pub struct NetworkConfig {
    /// Stellar RPC endpoint.
    pub rpc_url: String,
    /// Network passphrase, e.g. the public testnet passphrase.
    pub network_passphrase: String,
}

/// Client for one deployed vault.
#[derive(Clone, Debug)]
pub struct VaultClient {
    network: NetworkConfig,
    /// Vault contract address (`C...`).
    vault_id: String,
}

/// Parameters for a new withdrawal.
#[derive(Clone, Debug)]
pub struct WithdrawParams {
    /// Recipient strkey (`G...` or `C...`).
    pub to: String,
    /// Token contract strkey (`C...`).
    pub token: String,
    /// Amount in the token's smallest unit.
    pub amount: i128,
    /// Validity window in seconds from the current ledger time.
    pub ttl_seconds: u64,
}

/// Result of a submitted withdrawal.
#[derive(Clone, Debug)]
pub enum SubmitResult {
    /// Funds moved; contains the transaction hash.
    Executed { tx_hash: String },
    /// The vault rejected the request by policy; no funds moved.
    Rejected { tx_hash: String, reason: String },
}

impl VaultClient {
    /// Creates a client for the vault at `vault_id` on `network`.
    pub fn new(network: NetworkConfig, vault_id: impl Into<String>) -> Self {
        Self {
            network,
            vault_id: vault_id.into(),
        }
    }

    /// The network this client talks to.
    pub fn network(&self) -> &NetworkConfig {
        &self.network
    }

    /// The vault contract address.
    pub fn vault_id(&self) -> &str {
        &self.vault_id
    }

    /// Builds a withdrawal request with a fresh nonce and an expiry derived
    /// from the latest ledger time, and binds it to this network and vault.
    ///
    /// The returned [`WithdrawMessage`] is what approvers decode, display and
    /// sign; its canonical bytes are `WithdrawMessage::to_bytes()`.
    pub async fn build_withdraw_request(
        &self,
        params: WithdrawParams,
    ) -> Result<WithdrawMessage, SdkError> {
        todo!()
    }

    /// Simulates, assembles and submits the `withdraw` invocation via Stellar
    /// RPC, then waits for the transaction result.
    ///
    /// `source` signs the classical part: it must be (or be authorized by) the
    /// vault's admin address. Approvals are passed through unchanged; the
    /// contract performs all verification.
    pub async fn submit_withdraw(
        &self,
        request: &WithdrawRequest,
        approvals: Vec<PqApproval>,
        source: &dyn ClassicalSigner,
    ) -> Result<SubmitResult, SdkError> {
        todo!()
    }

    /// Streams the vault's audit events from `cursor` onward (Stellar RPC
    /// `getEvents`), decoded into [`VaultEvent`]s.
    pub async fn watch_events(&self, cursor: EventCursor) -> Result<EventSubscription, SdkError> {
        todo!()
    }
}

/// A source of ML-DSA approvals: a local `qrypton-signer`, an HSM-backed
/// service, or a remote approval endpoint.
pub trait ApprovalSource {
    /// Human-readable identifier, used in logs and errors.
    fn id(&self) -> &str;

    /// Requests an approval for `message`. May wait for a human decision.
    fn request_approval(
        &self,
        message: &WithdrawMessage,
    ) -> impl std::future::Future<Output = Result<PqApproval, SdkError>> + Send;
}

/// Signs the classical Stellar transaction envelope (Ed25519 today).
pub trait ClassicalSigner {
    /// Stellar account strkey (`G...`).
    fn account_id(&self) -> &str;

    /// Signs the transaction hash for `network_passphrase`.
    fn sign_tx_hash(&self, tx_hash: &[u8; 32]) -> Result<[u8; 64], SdkError>;
}

/// Requests approvals from `sources` concurrently and returns as soon as
/// `threshold` distinct approvals are collected.
///
/// Approvals are checked locally for distinct signer indices and matching
/// parameter sets; signature validity is left to the contract. Fails with
/// [`SdkError::ThresholdNotReached`] if too many sources decline or time out.
pub async fn collect_approvals<S: ApprovalSource>(
    message: &WithdrawMessage,
    sources: &[S],
    threshold: u32,
) -> Result<Vec<PqApproval>, SdkError> {
    todo!()
}
