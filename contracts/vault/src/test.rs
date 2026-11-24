//! Contract tests.
//!
//! Signatures are produced off-chain by `qrypton-signer` (the `fips204` crate)
//! and verified on-chain by the Soroban host's native ML-DSA implementation
//! (CAP-0087, RustCrypto `ml-dsa` inside the host). The two are independent
//! FIPS 204 implementations, so a passing withdrawal is an interoperability
//! check, not a self-consistency check. No mock verifier is involved.
//!
//! Governance and limit tests stay `#[ignore]` until those features land.

#![cfg(test)]

extern crate std;

use qrypton_signer::backend::{KeyBackend, SoftwareKeystore};
use qrypton_types::{AddressBytes, MlDsaLevel};
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    token, vec, Address, BytesN, Env, Event, Vec,
};

use crate::{
    message, Error, Policy, PqApproval, PqPublicKey, PqSignature, QryptonVault, QryptonVaultClient,
    RejectReason, TokenLimit, WithdrawExecuted, WithdrawOutcome, WithdrawRejected, WithdrawRequest,
    MAX_SIGNERS,
};

const NOW: u64 = 1_000_000;
const DEPOSIT: i128 = 600;

struct Setup {
    env: Env,
    vault_id: Address,
    vault: QryptonVaultClient<'static>,
    token: token::Client<'static>,
    depositor: Address,
    dest: Address,
    keys: std::vec::Vec<SoftwareKeystore>,
}

fn keystore(i: usize, level: MlDsaLevel) -> SoftwareKeystore {
    SoftwareKeystore::from_seed(level, [i as u8 + 1; 32])
}

fn pq_public_key(env: &Env, ks: &SoftwareKeystore) -> PqPublicKey {
    let bytes = ks.public_key().unwrap().bytes;
    match ks.algorithm() {
        MlDsaLevel::MlDsa65 => {
            PqPublicKey::MlDsa65(BytesN::from_array(env, &bytes.try_into().unwrap()))
        }
        MlDsaLevel::MlDsa87 => {
            PqPublicKey::MlDsa87(BytesN::from_array(env, &bytes.try_into().unwrap()))
        }
        MlDsaLevel::MlDsa44 => {
            PqPublicKey::MlDsa44(BytesN::from_array(env, &bytes.try_into().unwrap()))
        }
    }
}

fn setup(n: usize, threshold: u32) -> Setup {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);

    let operator = Address::generate(&env);
    let depositor = Address::generate(&env);
    let dest = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(Address::generate(&env));
    token::StellarAssetClient::new(&env, &sac.address()).mint(&depositor, &1_000);

    let keys: std::vec::Vec<_> = (0..n).map(|i| keystore(i, MlDsaLevel::MlDsa65)).collect();
    let mut signers: Vec<PqPublicKey> = Vec::new(&env);
    for k in &keys {
        signers.push_back(pq_public_key(&env, k));
    }
    let policy = Policy {
        // Listing the token is what lets the vault accept deposits of it.
        limits: vec![
            &env,
            TokenLimit {
                token: sac.address(),
                max_per_withdrawal: 1_000,
                max_per_window: 1_000,
            },
        ],
        window_seconds: 0,
        allowlist: vec![&env, dest.clone()],
        open_recipients: false,
        rescue_vault: None,
    };
    let vault_id = env.register(QryptonVault, (operator, signers, threshold, policy));
    let vault = QryptonVaultClient::new(&env, &vault_id);
    let token = token::Client::new(&env, &sac.address());
    vault.deposit(&depositor, &token.address, &DEPOSIT);

    Setup {
        env,
        vault_id,
        vault,
        token,
        depositor,
        dest,
        keys,
    }
}

impl Setup {
    fn request(&self, amount: i128, nonce: u64) -> WithdrawRequest {
        WithdrawRequest {
            to: self.dest.clone(),
            token: self.token.address.clone(),
            amount,
            nonce,
            expires_at: NOW + 300,
        }
    }

    /// The off-chain `qrypton-types` message for `req`, bound to `vault` and
    /// `network_id`, built from strkeys exactly as the signer CLI does.
    fn offchain_message(
        &self,
        req: &WithdrawRequest,
        vault: &Address,
        network_id: [u8; 32],
    ) -> qrypton_types::WithdrawMessage {
        qrypton_types::WithdrawMessage {
            network_id,
            vault: addr(vault),
            request: qrypton_types::WithdrawRequest {
                to: addr(&req.to),
                token: addr(&req.token),
                amount: req.amount,
                nonce: req.nonce,
                expires_at: req.expires_at,
            },
        }
    }

    /// Signs `req` with key `i` as `qrypton-signer` would.
    fn approve(&self, i: usize, req: &WithdrawRequest) -> PqApproval {
        let msg = self.offchain_message(req, &self.vault_id, self.network_id());
        self.sign_bytes(i, &msg.to_bytes())
    }

    fn sign_bytes(&self, i: usize, bytes: &[u8]) -> PqApproval {
        PqApproval {
            signer_index: i as u32,
            signature: pq_signature(&self.env, &self.keys[i], bytes),
        }
    }

    fn network_id(&self) -> [u8; 32] {
        self.env.ledger().network_id().to_array()
    }

    fn vault_balance(&self) -> i128 {
        self.token.balance(&self.vault_id)
    }

    fn vault_events(&self) -> std::vec::Vec<soroban_sdk::xdr::ContractEvent> {
        self.env
            .events()
            .all()
            .filter_by_contract(&self.vault_id)
            .events()
            .to_vec()
    }
}

fn pq_signature(env: &Env, ks: &SoftwareKeystore, bytes: &[u8]) -> PqSignature {
    let sig = ks.sign(bytes).unwrap().bytes;
    match ks.algorithm() {
        MlDsaLevel::MlDsa65 => {
            PqSignature::MlDsa65(BytesN::from_array(env, &sig.try_into().unwrap()))
        }
        MlDsaLevel::MlDsa87 => {
            PqSignature::MlDsa87(BytesN::from_array(env, &sig.try_into().unwrap()))
        }
        MlDsaLevel::MlDsa44 => {
            PqSignature::MlDsa44(BytesN::from_array(env, &sig.try_into().unwrap()))
        }
    }
}

/// Off-chain address encoding, via the strkey.
fn addr(a: &Address) -> AddressBytes {
    let s = a.to_string();
    let mut buf = [0u8; 56];
    s.copy_into_slice(&mut buf);
    AddressBytes::from_strkey(core::str::from_utf8(&buf).unwrap()).unwrap()
}

// ---------------------------------------------------------------------------
// Canonical message: on-chain == off-chain
// ---------------------------------------------------------------------------

#[test]
fn onchain_message_matches_offchain_encoder() {
    let s = setup(1, 1);
    // Account recipient and contract recipient.
    for to in [s.dest.clone(), s.token.address.clone()] {
        let mut req = s.request(123_456_789, u64::MAX - 1);
        req.to = to;
        let onchain = s
            .env
            .as_contract(&s.vault_id, || message::withdraw_message(&s.env, &req));
        let offchain = s
            .offchain_message(&req, &s.vault_id, s.network_id())
            .to_bytes();
        let mut onchain_bytes = std::vec![0u8; onchain.len() as usize];
        onchain.copy_into_slice(&mut onchain_bytes);
        assert_eq!(onchain_bytes, offchain);
    }
}

// ---------------------------------------------------------------------------
// Successful withdrawals
// ---------------------------------------------------------------------------

#[test]
fn withdraw_1_of_1_fips204_signature_verified_by_host() {
    let s = setup(1, 1);
    let req = s.request(250, 1);
    let approvals = vec![&s.env, s.approve(0, &req)];

    s.env.cost_estimate().budget().reset_default();
    let outcome = s.vault.withdraw(&req, &approvals);
    let events = s.vault_events();
    let auths = s.env.auths();
    let budget = s.env.cost_estimate().budget();
    std::println!(
        "1-of-1 ML-DSA-65 withdraw (native test build): cpu_insns={} mem_bytes={}",
        budget.cpu_instruction_cost(),
        budget.memory_bytes_cost()
    );

    assert_eq!(outcome, WithdrawOutcome::Executed);
    assert_eq!(s.token.balance(&s.dest), 250);
    assert_eq!(s.vault_balance(), DEPOSIT - 250);
    assert!(s.vault.is_nonce_used(&1));
    assert_eq!(
        events,
        std::vec![WithdrawExecuted {
            token: s.token.address.clone(),
            to: s.dest.clone(),
            amount: 250,
            nonce: 1,
            signer_indices: vec![&s.env, 0],
        }
        .to_xdr(&s.env, &s.vault_id)]
    );
    // The classical factor was required: the operator's authorization was used.
    assert_eq!(auths.len(), 1);
}

#[test]
fn withdraw_2_of_3_executes() {
    let s = setup(3, 2);
    let req = s.request(100, 7);
    let approvals = vec![&s.env, s.approve(2, &req), s.approve(0, &req)];
    assert_eq!(
        s.vault.withdraw(&req, &approvals),
        WithdrawOutcome::Executed
    );
    assert_eq!(s.token.balance(&s.dest), 100);
}

#[test]
fn deposit_transfers_into_vault() {
    let s = setup(1, 1);
    assert_eq!(s.vault_balance(), DEPOSIT);
    assert_eq!(s.token.balance(&s.depositor), 1_000 - DEPOSIT);
}

#[test]
fn invalid_approval_before_threshold_fails() {
    // Order matters: the host traps on the first invalid signature it checks.
    let s = setup(3, 2);
    let req = s.request(100, 1);
    let mut bad = s.approve(2, &req);
    if let PqSignature::MlDsa65(sig) = &bad.signature {
        let mut bytes = sig.to_array();
        bytes[100] ^= 0x01;
        bad.signature = PqSignature::MlDsa65(BytesN::from_array(&s.env, &bytes));
    }
    let approvals = vec![&s.env, bad, s.approve(0, &req), s.approve(1, &req)];
    assert!(s.vault.try_withdraw(&req, &approvals).is_err());
    assert_untouched(&s, 1, s.vault_events());
}

#[test]
fn deposit_rejects_token_not_listed_in_policy() {
    let s = setup(1, 1);
    let other = s
        .env
        .register_stellar_asset_contract_v2(Address::generate(&s.env));
    token::StellarAssetClient::new(&s.env, &other.address()).mint(&s.depositor, &100);
    assert_eq!(
        s.vault.try_deposit(&s.depositor, &other.address(), &100),
        Err(Ok(Error::TokenNotAccepted))
    );
    assert!(s.vault_events().is_empty());
}

#[test]
fn surplus_approval_beyond_threshold_is_ignored() {
    let s = setup(3, 2);
    let req = s.request(100, 1);
    let mut bad = s.approve(2, &req);
    if let PqSignature::MlDsa65(sig) = &bad.signature {
        let mut bytes = sig.to_array();
        bytes[100] ^= 0x01;
        bad.signature = PqSignature::MlDsa65(BytesN::from_array(&s.env, &bytes));
    }
    let approvals = vec![&s.env, s.approve(0, &req), s.approve(1, &req), bad];

    s.env.cost_estimate().budget().reset_default();
    assert_eq!(
        s.vault.withdraw(&req, &approvals),
        WithdrawOutcome::Executed
    );
    let verifications = s
        .env
        .cost_estimate()
        .budget()
        .tracker(soroban_sdk::xdr::ContractCostType::VerifyMlDsa65Sig)
        .iterations;
    assert_eq!(verifications, 2, "verification stops at the threshold");
    assert_eq!(s.token.balance(&s.dest), 100);
}

// ---------------------------------------------------------------------------
// Failures: nothing changes, no event
// ---------------------------------------------------------------------------

/// Asserts that a failed call emitted no vault event and left balances and
/// the nonce as they were. `events` must be captured right after the call:
/// `env.events().all()` only covers the most recent invocation.
fn assert_untouched(s: &Setup, nonce: u64, events: std::vec::Vec<soroban_sdk::xdr::ContractEvent>) {
    assert!(events.is_empty());
    assert_eq!(s.vault_balance(), DEPOSIT);
    assert!(!s.vault.is_nonce_used(&nonce));
}

#[test]
fn corrupted_signature_fails() {
    let s = setup(1, 1);
    let req = s.request(250, 1);
    let mut approval = s.approve(0, &req);
    if let PqSignature::MlDsa65(sig) = &approval.signature {
        let mut bytes = sig.to_array();
        bytes[100] ^= 0x01;
        approval.signature = PqSignature::MlDsa65(BytesN::from_array(&s.env, &bytes));
    }
    assert!(s.vault.try_withdraw(&req, &vec![&s.env, approval]).is_err());
    assert_untouched(&s, 1, s.vault_events());
}

#[test]
fn signature_for_a_different_amount_fails() {
    let s = setup(1, 1);
    let approval = s.approve(0, &s.request(1, 1));
    assert!(s
        .vault
        .try_withdraw(&s.request(500, 1), &vec![&s.env, approval])
        .is_err());
    assert_untouched(&s, 1, s.vault_events());
}

#[test]
fn approval_for_another_vault_or_network_fails() {
    let s = setup(1, 1);
    let req = s.request(250, 1);
    let other_vault = Address::generate(&s.env);
    let for_other_vault = s.offchain_message(&req, &other_vault, s.network_id());
    let for_other_network = s.offchain_message(&req, &s.vault_id, [9; 32]);
    for msg in [for_other_vault, for_other_network] {
        let approval = s.sign_bytes(0, &msg.to_bytes());
        assert!(s.vault.try_withdraw(&req, &vec![&s.env, approval]).is_err());
        assert_untouched(&s, 1, s.vault_events());
    }
}

#[test]
fn withdraw_rejects_quorum_without_operator() {
    let s = setup(1, 1);
    let req = s.request(250, 1);
    let approvals = vec![&s.env, s.approve(0, &req)];
    s.env.set_auths(&[]); // no classical authorization at all
    assert!(s.vault.try_withdraw(&req, &approvals).is_err());
    assert_untouched(&s, 1, s.vault_events());
}

#[test]
fn withdraw_rejects_operator_without_quorum_and_keeps_nonce_usable() {
    let s = setup(3, 2);
    let req = s.request(100, 5);

    // Operator alone: no approvals, then fewer than the threshold.
    let none: Vec<PqApproval> = Vec::new(&s.env);
    assert_eq!(
        s.vault.try_withdraw(&req, &none),
        Err(Ok(Error::InsufficientApprovals))
    );
    assert_untouched(&s, 5, s.vault_events());
    assert_eq!(
        s.vault
            .try_withdraw(&req, &vec![&s.env, s.approve(1, &req)]),
        Err(Ok(Error::InsufficientApprovals))
    );
    assert_untouched(&s, 5, s.vault_events());

    // The same nonce still works for a properly approved request.
    let approvals = vec![&s.env, s.approve(0, &req), s.approve(1, &req)];
    assert_eq!(
        s.vault.withdraw(&req, &approvals),
        WithdrawOutcome::Executed
    );
}

#[test]
fn duplicate_approval_is_counted_once() {
    let s = setup(3, 2);
    let req = s.request(100, 1);
    let a = s.approve(0, &req);
    assert_eq!(
        s.vault.try_withdraw(&req, &vec![&s.env, a.clone(), a]),
        Err(Ok(Error::DuplicateApproval))
    );
    assert_untouched(&s, 1, s.vault_events());
}

#[test]
fn unknown_signer_index_fails() {
    let s = setup(1, 1);
    let req = s.request(100, 1);
    let mut a = s.approve(0, &req);
    a.signer_index = 3;
    assert_eq!(
        s.vault.try_withdraw(&req, &vec![&s.env, a]),
        Err(Ok(Error::UnknownSigner))
    );
}

#[test]
fn withdraw_rejects_algorithm_mismatch() {
    let s = setup(1, 1);
    let req = s.request(100, 1);
    let msg = s.offchain_message(&req, &s.vault_id, s.network_id());
    let wrong = PqApproval {
        signer_index: 0,
        signature: pq_signature(&s.env, &keystore(0, MlDsaLevel::MlDsa87), &msg.to_bytes()),
    };
    assert_eq!(
        s.vault.try_withdraw(&req, &vec![&s.env, wrong]),
        Err(Ok(Error::AlgorithmMismatch))
    );
}

#[test]
fn replayed_approvals_emit_no_event_and_change_nothing() {
    let s = setup(1, 1);
    let req = s.request(250, 1);
    let approvals = vec![&s.env, s.approve(0, &req)];
    assert_eq!(
        s.vault.withdraw(&req, &approvals),
        WithdrawOutcome::Executed
    );
    let balance = s.vault_balance();

    // The approvals are public now; replaying them is an error, not a
    // recorded rejection, and is decided before any signature verification.
    s.env.cost_estimate().budget().reset_default();
    assert_eq!(
        s.vault.try_withdraw(&req, &approvals),
        Err(Ok(Error::NonceAlreadyUsed))
    );
    let events = s.vault_events();
    let tracker = s
        .env
        .cost_estimate()
        .budget()
        .tracker(soroban_sdk::xdr::ContractCostType::VerifyMlDsa65Sig);
    assert_eq!(tracker.iterations, 0, "no ML-DSA verification on replay");
    assert!(events.is_empty());
    assert_eq!(s.vault_balance(), balance);
}

// ---------------------------------------------------------------------------
// Quorum-approved but rejected: event, nonce consumed, no funds moved
// ---------------------------------------------------------------------------

#[test]
fn expired_request_is_rejected_with_event() {
    let s = setup(1, 1);
    let req = s.request(250, 1);
    let approvals = vec![&s.env, s.approve(0, &req)];
    s.env.ledger().set_timestamp(req.expires_at + 1);

    assert_eq!(
        s.vault.withdraw(&req, &approvals),
        WithdrawOutcome::Rejected(RejectReason::Expired)
    );
    assert_eq!(
        s.vault_events(), // still the withdraw invocation's events
        std::vec![WithdrawRejected {
            token: s.token.address.clone(),
            to: s.dest.clone(),
            amount: 250,
            nonce: 1,
            reason: RejectReason::Expired,
        }
        .to_xdr(&s.env, &s.vault_id)]
    );
    assert_eq!(s.vault_balance(), DEPOSIT);
    assert!(s.vault.is_nonce_used(&1));
}

#[test]
fn recipient_outside_allowlist_is_rejected_with_event() {
    let s = setup(1, 1);
    let mut req = s.request(250, 1);
    req.to = Address::generate(&s.env);
    let approvals = vec![&s.env, s.approve(0, &req)];

    assert_eq!(
        s.vault.withdraw(&req, &approvals),
        WithdrawOutcome::Rejected(RejectReason::RecipientNotAllowed)
    );
    assert_eq!(s.vault_events().len(), 1);
    assert_eq!(s.vault_balance(), DEPOSIT);
    assert!(s.vault.is_nonce_used(&1));
}

// ---------------------------------------------------------------------------
// Constructor
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn constructor_rejects_threshold_above_signer_count() {
    let env = Env::default();
    let signers = vec![&env, pq_public_key(&env, &keystore(0, MlDsaLevel::MlDsa65))];
    let policy = Policy {
        limits: vec![&env],
        window_seconds: 0,
        allowlist: vec![&env],
        open_recipients: false,
        rescue_vault: None,
    };
    env.register(
        QryptonVault,
        (Address::generate(&env), signers, 2_u32, policy),
    );
}

#[test]
fn constructor_accepts_max_signers() {
    let env = Env::default();
    let mut signers: Vec<PqPublicKey> = Vec::new(&env);
    for i in 0..MAX_SIGNERS as usize {
        signers.push_back(pq_public_key(&env, &keystore(i, MlDsaLevel::MlDsa65)));
    }
    let policy = Policy {
        limits: vec![&env],
        window_seconds: 0,
        allowlist: vec![&env],
        open_recipients: false,
        rescue_vault: None,
    };
    let id = env.register(
        QryptonVault,
        (Address::generate(&env), signers, MAX_SIGNERS, policy),
    );
    assert_eq!(
        QryptonVaultClient::new(&env, &id).signers().len(),
        MAX_SIGNERS
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #21)")]
fn constructor_rejects_too_many_signers() {
    let env = Env::default();
    let mut signers: Vec<PqPublicKey> = Vec::new(&env);
    for i in 0..=MAX_SIGNERS as usize {
        signers.push_back(pq_public_key(&env, &keystore(i, MlDsaLevel::MlDsa65)));
    }
    let policy = Policy {
        limits: vec![&env],
        window_seconds: 0,
        allowlist: vec![&env],
        open_recipients: false,
        rescue_vault: None,
    };
    env.register(
        QryptonVault,
        (Address::generate(&env), signers, 1_u32, policy),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn constructor_rejects_duplicate_signers() {
    let env = Env::default();
    let k = pq_public_key(&env, &keystore(0, MlDsaLevel::MlDsa65));
    let policy = Policy {
        limits: vec![&env],
        window_seconds: 0,
        allowlist: vec![&env],
        open_recipients: false,
        rescue_vault: None,
    };
    env.register(
        QryptonVault,
        (
            Address::generate(&env),
            vec![&env, k.clone(), k],
            1_u32,
            policy,
        ),
    );
}

// ---------------------------------------------------------------------------
// Not implemented in this proof
// ---------------------------------------------------------------------------

#[test]
#[ignore = "token limits arrive after the testnet proof"]
fn withdraw_enforces_per_withdrawal_and_window_limits() {
    todo!()
}

#[test]
#[ignore = "rescue vault arrives after the testnet proof"]
fn empty_allowlist_admits_only_rescue_vault() {
    todo!()
}

#[test]
#[ignore = "rescue vault arrives after the testnet proof"]
fn withdraw_to_rescue_vault_skips_limits_and_allowlist() {
    todo!()
}

#[test]
#[ignore = "governance arrives after the testnet proof"]
fn key_rotation_is_timelocked_and_cancellable() {
    todo!()
}

#[test]
#[ignore = "governance arrives after the testnet proof"]
fn operator_change_needs_only_quorum_and_is_timelocked() {
    todo!()
}

#[test]
#[ignore = "governance arrives after the testnet proof"]
fn operator_cannot_cancel_operator_change_alone() {
    todo!()
}

#[test]
#[ignore = "governance arrives after the testnet proof"]
fn only_one_operator_change_can_be_pending() {
    todo!()
}

#[test]
#[ignore = "governance arrives after the testnet proof"]
fn executing_key_rotation_cancels_pending_operator_change() {
    todo!()
}

#[test]
#[ignore = "governance arrives after the testnet proof"]
fn execute_operator_change_is_permissionless() {
    todo!()
}

#[test]
#[ignore = "governance arrives after the testnet proof"]
fn governance_approval_is_bound_to_vault_network_and_action() {
    todo!()
}

#[test]
#[ignore = "governance arrives after the testnet proof"]
fn tightening_policy_change_applies_immediately() {
    todo!()
}

#[test]
#[ignore = "governance arrives after the testnet proof"]
fn loosening_policy_change_is_timelocked() {
    todo!("raise limit, add token, add allowlist entry, open recipients, change rescue vault")
}

#[test]
#[ignore = "governance arrives after the testnet proof"]
fn executing_key_rotation_cancels_pending_policy_change() {
    todo!()
}

#[test]
#[ignore = "governance arrives after the testnet proof"]
fn loosening_cannot_execute_before_timelock() {
    todo!()
}

#[test]
#[ignore = "governance arrives after the testnet proof"]
fn stale_cancel_approval_cannot_cancel_newer_proposal() {
    todo!()
}

#[test]
#[ignore = "governance arrives after the testnet proof"]
fn set_policy_requires_operator_and_current_threshold() {
    todo!()
}
