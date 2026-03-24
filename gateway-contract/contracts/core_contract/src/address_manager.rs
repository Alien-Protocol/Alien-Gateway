use crate::registration::Registration;
use crate::types::PrivacyMode;
use soroban_sdk::{contracterror, contractevent, contracttype, panic_with_error, BytesN, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Privacy(BytesN<32>),
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivSet {
    pub username_hash: BytesN<32>,
    pub mode: u32,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AddressManagerError {
    UsernameNotRegistered = 1,
}

pub struct AddressManager;

impl AddressManager {
    /// Set privacy mode for a username hash.
    /// Requires authentication from the owner of the username hash.
    pub fn set_privacy_mode(env: Env, username_hash: BytesN<32>, mode: PrivacyMode) {
        // Retrieve the owner of the username hash
        let owner = Registration::get_owner(env.clone(), username_hash.clone())
            .unwrap_or_else(|| panic_with_error!(&env, AddressManagerError::UsernameNotRegistered));

        // Ensure the caller is the owner
        owner.require_auth();

        // Store the privacy mode in persistent storage
        let key = DataKey::Privacy(username_hash.clone());
        env.storage().persistent().set(&key, &mode);

        // Emit typed PrivSet event (using structured publish method)
        let mode_val: u32 = match mode {
            PrivacyMode::Normal => 0,
            PrivacyMode::Private => 1,
        };
        PrivSet {
            username_hash,
            mode: mode_val,
        }
        .publish(&env);
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
