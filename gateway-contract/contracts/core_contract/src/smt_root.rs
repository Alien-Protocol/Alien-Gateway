use soroban_sdk::{BytesN, Env};

use crate::events::ROOT_UPDATED;
use crate::storage::DataKey;

pub struct SmtRoot;

impl SmtRoot {
    /// Updates the SMT root internally (not exposed as a public contract function).
    ///
    /// This internal helper is called during verified proof submission flows (registration and transfer).
    /// It atomically updates the root and emits the update event for indexers.
    /// Should only be called from verified proof contexts.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `new_root`: The 32-byte new SMT root to set.
    ///
    /// ### Events
    /// - Emits `ROOT_UPDATED` event with (old_root, new_root).
    #[allow(dead_code)]
    pub fn update_root(env: &Env, new_root: BytesN<32>) {
        let old_root: Option<BytesN<32>> = env.storage().instance().get(&DataKey::SmtRoot);

        env.storage().instance().set(&DataKey::SmtRoot, &new_root);

        #[allow(deprecated)]
        env.events().publish((ROOT_UPDATED,), (old_root, new_root));
    }

    /// Retrieves the current SMT root, or None if not yet set.
    ///
    /// Returns the latest Sparse Merkle Tree root hash used for validating ZK proofs.
    /// This is a read-only query operation with no authentication requirement.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    ///
    /// ### Returns
    /// - `Some(BytesN<32>)` with the current SMT root.
    /// - `None` if the root has not been initialized yet.
    pub fn get_root(env: Env) -> Option<BytesN<32>> {
        env.storage().instance().get(&DataKey::SmtRoot)
    }
}
