use soroban_sdk::{panic_with_error, Address, BytesN, Env};

use crate::errors::CoreError;
use crate::events::INIT_EVENT;
use crate::{smt_root, storage};

pub struct Admin;

impl Admin {
    /// Initializes the contract with the contract owner.
    ///
    /// This function must be called exactly once during contract deployment.
    /// Only the owner can authorize this call. Prevents reinitialization.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `owner`: The address to be set as the contract owner. Must be authorized.
    ///
    /// ### Errors
    /// - `AlreadyInitialized`: If the contract has already been initialized.
    ///
    /// ### Events
    /// - Emits `INIT_EVENT` with the owner address.
    pub fn initialize(env: Env, owner: Address) {
        owner.require_auth();
        if storage::is_initialized(&env) {
            panic_with_error!(&env, CoreError::AlreadyInitialized);
        }
        storage::set_owner(&env, &owner);
        #[allow(deprecated)]
        env.events().publish((INIT_EVENT,), (owner,));
    }

    /// Retrieves the contract owner's address.
    ///
    /// ### Returns
    /// The address of the contract owner.
    ///
    /// ### Errors
    /// - `NotFound`: If the contract has not been initialized.
    pub fn get_contract_owner(env: Env) -> Address {
        storage::get_owner(&env).unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound))
    }

    /// Retrieves the current Sparse Merkle Tree (SMT) root hash.
    ///
    /// This root is used to validate zero-knowledge proofs during registration and transfers.
    ///
    /// ### Returns
    /// A 32-byte hash representing the current SMT root.
    ///
    /// ### Errors
    /// - `RootNotSet`: If the SMT root has not yet been set.
    pub fn get_smt_root(env: Env) -> BytesN<32> {
        smt_root::SmtRoot::get_root(env.clone())
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::RootNotSet))
    }
}
