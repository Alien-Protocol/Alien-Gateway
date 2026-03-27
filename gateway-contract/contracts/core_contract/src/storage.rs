use soroban_sdk::{contracttype, Address, BytesN, Env};

use crate::types::PrivacyMode;

/// Storage keys for the Core contract's persistent and instance storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Key for resolver data, indexed by commitment.
    Resolver(BytesN<32>),
    /// Key for the SMT root in instance storage.
    SmtRoot,
    /// Key for the primary Stellar address linked to a username hash.
    StellarAddress(BytesN<32>),
    /// Key for the user's selected privacy mode.
    PrivacyMode(BytesN<32>),
    /// Key for the contract owner set during initialization (instance storage).
    Owner,
    /// Key for a shielded address commitment, indexed by username hash.
    ShieldedAddress(BytesN<32>),
}

pub fn set_privacy_mode(env: &Env, username_hash: &BytesN<32>, mode: &PrivacyMode) {
    env.storage()
        .persistent()
        .set(&DataKey::PrivacyMode(username_hash.clone()), mode);
}

pub fn get_privacy_mode(env: &Env, username_hash: &BytesN<32>) -> PrivacyMode {
    env.storage()
        .persistent()
        .get::<DataKey, PrivacyMode>(&DataKey::PrivacyMode(username_hash.clone()))
        .unwrap_or(PrivacyMode::Normal)
}

pub fn set_owner(env: &Env, owner: &Address) {
    env.storage().instance().set(&DataKey::Owner, owner);
}

pub fn get_owner(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Owner)
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Owner)
}

pub fn set_shielded_address(env: &Env, username_hash: &BytesN<32>, commitment: &BytesN<32>) {
    env.storage()
        .persistent()
        .set(&DataKey::ShieldedAddress(username_hash.clone()), commitment);
}

pub fn get_shielded_address(env: &Env, username_hash: &BytesN<32>) -> Option<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&DataKey::ShieldedAddress(username_hash.clone()))
}

pub fn has_shielded_address(env: &Env, username_hash: &BytesN<32>) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::ShieldedAddress(username_hash.clone()))
}
