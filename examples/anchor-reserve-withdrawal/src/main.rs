//! # Scenario: anchor reserve withdrawal
//!
//! An anchor keeps its USDC reserve in a Qrypton Vault. A customer redeems,
//! and the anchor's treasury must move funds from the reserve to its
//! operational hot wallet.
//!
//! * Signer set: 3 ML-DSA-65 approvers (treasury, compliance, risk), each on
//!   its own HSM. Threshold: 2.
//! * Policy: per-withdrawal and rolling-window limits for USDC; the hot wallet
//!   is the only allowlisted recipient.
//! * Classical authorization: the anchor's operator account.
//!
//! Flow:
//! 1. Treasury builds the withdrawal request with the SDK.
//! 2. The request is sent to the three approvers; each reviews and signs.
//! 3. As soon as 2 approvals arrive, the SDK submits the withdrawal.
//! 4. The anchor's reconciliation service observes `WithdrawExecuted`.
//!
//! Skeleton only: compiles, but every step panics with `todo!()`.

use qrypton_sdk::{NetworkConfig, VaultClient, WithdrawParams};

const RESERVE_VAULT: &str = "C...RESERVE_VAULT";
const USDC_TOKEN: &str = "C...USDC_SAC";
const HOT_WALLET: &str = "G...ANCHOR_HOT_WALLET";
const THRESHOLD: u32 = 2;

#[tokio::main]
async fn main() {
    let network = NetworkConfig {
        rpc_url: "https://soroban-testnet.stellar.org".into(),
        network_passphrase: "Test SDF Network ; September 2015".into(),
    };
    let vault = VaultClient::new(network, RESERVE_VAULT);

    // 1. Build the request: 25,000 USDC (7 decimals) to the hot wallet, valid 15 minutes.
    let _message = vault
        .build_withdraw_request(WithdrawParams {
            to: HOT_WALLET.into(),
            token: USDC_TOKEN.into(),
            amount: 25_000 * 10_i128.pow(7),
            ttl_seconds: 15 * 60,
        })
        .await;

    // 2. Collect 2-of-3 approvals from the treasury, compliance and risk approvers:
    //    qrypton_sdk::collect_approvals(&message, &approvers, THRESHOLD).await
    // 3. Submit with the anchor operator as classical signer:
    //    vault.submit_withdraw(&message.request, approvals, &operator).await
    // 4. Watch for WithdrawExecuted / WithdrawRejected:
    //    vault.watch_events(EventCursor::Ledger(start)).await
    let _ = THRESHOLD;
    todo!("wire approvers and the classical signer")
}
