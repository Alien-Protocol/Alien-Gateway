use soroban_sdk::{contracttype, Address, BytesN, Env, Symbol};
use crate::types::PrivacyMode;
use crate::registration::Registration;
use crate::events::PRIV_SET;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Privacy(BytesN<32>),
}

pub struct AddressManager;

impl AddressManager {
    /// Set privacy mode for a username hash.
    /// Requires authentication from the owner of the username hash.
    pub fn set_privacy_mode(env: Env, username_hash: BytesN<32>, mode: PrivacyMode) {
        // Retrieve the owner of the username hash
        let owner = Registration::get_owner(env.clone(), username_hash.clone())
            .expect("Username not registered");
        
        // Ensure the caller is the owner
        owner.require_auth();

        // Store the privacy mode in persistent storage
        let key = DataKey::Privacy(username_hash.clone());
        env.storage().persistent().set(&key, &mode);

        // Emit PRIV_SET event
        let mode_val: u32 = match mode {
            PrivacyMode::Normal => 0,
            PrivacyMode::Private => 1,
        };
        env.events().publish((PRIV_SET,), (username_hash, mode_val));
    }

    /// Get privacy mode for a username hash.
    /// Defaults to Normal if not set.
    pub fn get_privacy_mode(env: Env, username_hash: BytesN<32>) -> PrivacyMode {
        let key = DataKey::Privacy(username_hash);
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(PrivacyMode::Normal)
    }
}
