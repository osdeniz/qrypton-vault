//! Canonical-encoding test vectors shared with the contract.
//!
//! The same vectors are checked on-chain in `contracts/vault/src/test.rs`, so a
//! divergence between the off-chain and on-chain encoders fails CI.

#[test]
#[ignore = "pre-alpha: encoder not implemented"]
fn withdraw_v1_vector_roundtrip() {
    todo!("load vectors, encode, compare bytes, decode, compare struct")
}

#[test]
#[ignore = "pre-alpha: encoder not implemented"]
fn withdraw_v1_rejects_trailing_bytes() {
    todo!()
}

#[test]
#[ignore = "pre-alpha: encoder not implemented"]
fn domain_tags_are_distinct() {
    todo!()
}
