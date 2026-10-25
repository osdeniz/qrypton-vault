//! Contract test skeletons. All tests are ignored until the corresponding
//! functionality is implemented.

#![cfg(test)]

extern crate std;

#[test]
#[ignore = "pre-alpha"]
fn constructor_rejects_threshold_above_signer_count() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn deposit_transfers_and_emits_event() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn withdraw_requires_operator_auth() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn withdraw_rejects_expired_request() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn withdraw_rejects_reused_nonce() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn withdraw_enforces_per_withdrawal_and_window_limits() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn withdraw_enforces_allowlist() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn withdraw_requires_threshold_distinct_approvals() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn withdraw_rejects_algorithm_mismatch() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn withdraw_message_matches_offchain_test_vectors() {
    todo!("compare against crates/qrypton-types test vectors")
}

#[test]
#[ignore = "pre-alpha"]
fn key_rotation_is_timelocked_and_cancellable() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn set_policy_requires_current_threshold() {
    todo!()
}

#[test]
#[ignore = "requires a network with native ML-DSA host functions"]
fn native_verifier_accepts_fips204_known_answer() {
    todo!("FIPS 204 / ACVP known-answer vectors")
}
