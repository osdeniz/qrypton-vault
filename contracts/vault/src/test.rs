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
fn withdraw_to_rescue_vault_skips_limits_and_allowlist() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn operator_change_needs_only_quorum_and_is_timelocked() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn operator_cannot_cancel_operator_change_alone() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn only_one_operator_change_can_be_pending() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn executing_key_rotation_cancels_pending_operator_change() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn execute_operator_change_is_permissionless() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn governance_approval_is_bound_to_vault_network_and_action() {
    todo!("approval for one action/vault/network must not verify for another")
}

#[test]
#[ignore = "pre-alpha"]
fn tightening_policy_change_applies_immediately() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn loosening_policy_change_is_timelocked() {
    todo!("raise limit, add token, add allowlist entry, empty allowlist, change rescue vault")
}

#[test]
#[ignore = "pre-alpha"]
fn executing_key_rotation_cancels_pending_policy_change() {
    todo!()
}

#[test]
#[ignore = "pre-alpha"]
fn set_policy_requires_operator_and_current_threshold() {
    todo!()
}

#[test]
#[ignore = "requires a network with native ML-DSA host functions"]
fn native_verifier_accepts_fips204_known_answer() {
    todo!("FIPS 204 / ACVP known-answer vectors")
}
