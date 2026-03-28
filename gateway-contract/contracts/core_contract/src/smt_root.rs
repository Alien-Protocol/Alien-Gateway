use soroban_sdk::{BytesN, Env};

use crate::events::ROOT_UPDATED;
use crate::storage::DataKey;

pub struct SmtRoot;

impl SmtRoot {
    /// Update the SMT root internally (not exposed as a public contract function).
    /// This should only be called from within verified proof submission flow.
    /// Emits ROOT_UPD event with (old_root, new_root).
    #[allow(dead_code)]
    pub fn update_root(env: &Env, new_root: BytesN<32>) {
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
