use soroban_sdk::{Address, Env};

use super::DataKey;

/// Requires that the current invoker is the contract owner. Use at the start of
/// all state-changing (write) functions. No state change without auth.
pub fn require_owner(env: &Env) {
    let owner: Address = env
        .storage()
        .instance()
        .get(&DataKey::Owner)
        .unwrap_or_else(|| panic!("Contract not initialized"));
    owner.require_auth();
}
