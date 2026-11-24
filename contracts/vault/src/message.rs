//! On-chain construction of the canonical signed messages.
//!
//! Must produce exactly the bytes of `qrypton_types::WithdrawMessage::to_bytes`
//! for the same request; `test.rs` checks this against the off-chain encoder.

use soroban_sdk::{xdr::ToXdr, Address, Bytes, Env};

use crate::types::WithdrawRequest;

/// `qrypton-vault/withdraw/v1` message for `request`, bound to the current
/// network and to this vault.
pub(crate) fn withdraw_message(env: &Env, request: &WithdrawRequest) -> Bytes {
    let tag = qrypton_types::message::WITHDRAW_V1_TAG;
    let mut m = Bytes::new(env);
    m.push_back(tag.len() as u8);
    m.extend_from_slice(tag);
    m.append(&env.ledger().network_id().into());
    put_address(env, &mut m, &env.current_contract_address());
    put_address(env, &mut m, &request.to);
    put_address(env, &mut m, &request.token);
    m.extend_from_array(&request.amount.to_be_bytes());
    m.extend_from_array(&request.nonce.to_be_bytes());
    m.extend_from_array(&request.expires_at.to_be_bytes());
    m
}

/// `u32 length || XDR(ScVal::Address)`.
fn put_address(env: &Env, m: &mut Bytes, address: &Address) {
    let xdr = address.clone().to_xdr(env);
    m.extend_from_array(&xdr.len().to_be_bytes());
    m.append(&xdr);
}
