use soroban_sdk::{contracttype, BytesN, Env};

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
