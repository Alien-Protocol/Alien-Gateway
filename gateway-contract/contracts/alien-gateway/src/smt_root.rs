use soroban_sdk::{contracttype, symbol_short, BytesN, Env, Symbol};

use crate::contract_core;

// Storage Keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    SmtRoot,
}

// Event
const ROOT_UPDATED: Symbol = symbol_short!("ROOT_UPD");

pub struct SmtRoot;

impl SmtRoot {
    /// Update the SMT root. Caller must be the contract owner.
    /// Emits ROOT_UPD event with (old_root, new_root).
    pub fn update_root(env: Env, new_root: BytesN<32>) {
        contract_core::auth::require_owner(&env);

        let old_root: Option<BytesN<32>> = env.storage().instance().get(&DataKey::SmtRoot);

        env.storage().instance().set(&DataKey::SmtRoot, &new_root);

        #[allow(deprecated)]
        env.events().publish((ROOT_UPDATED,), (old_root, new_root));
    }

    /// Return the current SMT root, or None if not yet set.
    pub fn get_root(env: Env) -> Option<BytesN<32>> {
        env.storage().instance().get(&DataKey::SmtRoot)
    }
}
