//! Contract storage layout and TTL handling.

use soroban_sdk::{contracttype, Address, Env, Vec};

use crate::types::{Policy, PqPublicKey};

const DAY_IN_LEDGERS: u32 = 17_280;
/// Extend entries to ~30 days whenever they drop below ~7 days.
const BUMP_THRESHOLD: u32 = 7 * DAY_IN_LEDGERS;
const BUMP_TO: u32 = 30 * DAY_IN_LEDGERS;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Operator,
    Signers,
    Threshold,
    Policy,
    /// A consumed nonce (persistent, one entry per nonce).
    Nonce(u64),
}

pub(crate) fn init(
    env: &Env,
    operator: &Address,
    signers: &Vec<PqPublicKey>,
    threshold: u32,
    policy: &Policy,
) {
    let s = env.storage().instance();
    s.set(&DataKey::Operator, operator);
    s.set(&DataKey::Signers, signers);
    s.set(&DataKey::Threshold, &threshold);
    s.set(&DataKey::Policy, policy);
    bump_instance(env);
}

pub(crate) fn bump_instance(env: &Env) {
    env.storage().instance().extend_ttl(BUMP_THRESHOLD, BUMP_TO);
}

pub(crate) fn operator(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Operator).unwrap()
}

pub(crate) fn signers(env: &Env) -> Vec<PqPublicKey> {
    env.storage().instance().get(&DataKey::Signers).unwrap()
}

pub(crate) fn threshold(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::Threshold).unwrap()
}

pub(crate) fn policy(env: &Env) -> Policy {
    env.storage().instance().get(&DataKey::Policy).unwrap()
}

pub(crate) fn is_nonce_used(env: &Env, nonce: u64) -> bool {
    env.storage().persistent().has(&DataKey::Nonce(nonce))
}

pub(crate) fn consume_nonce(env: &Env, nonce: u64) {
    let key = DataKey::Nonce(nonce);
    env.storage().persistent().set(&key, &());
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TO);
}
