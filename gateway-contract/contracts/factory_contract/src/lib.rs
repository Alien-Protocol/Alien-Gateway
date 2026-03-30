#![no_std]

mod errors;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, panic_with_error, Address, BytesN, Env};

use crate::errors::FactoryError;
use crate::events::emit_username_deployed;
use crate::storage::{
    get_auction_contract, get_core_contract, get_username, has_username, set_auction_contract,
    set_core_contract, set_username,
};
use crate::types::UsernameRecord;

#[contract]
pub struct FactoryContract;

#[contractimpl]
impl FactoryContract {
    /// Configures the factory with the auction and core contract addresses.
    ///
    /// This should be called to link the factory with other system components.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `auction_contract` - The address of the auction contract authorized to deploy usernames.
    /// * `core_contract` - The address of the core contract to be associated with new usernames.
    ///
    /// # Complexity
    ///
    /// O(1) - single storage write for each address.
    pub fn configure(env: Env, auction_contract: Address, core_contract: Address) {
        set_auction_contract(&env, &auction_contract);
        set_core_contract(&env, &core_contract);
    }

    /// Deploys a new username record to the factory storage.
    ///
    /// This function can only be called by the configured auction contract.
    /// It validates that the username hash is not already registered.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `username_hash` - The 32-byte hash identifying the unique username.
    /// * `owner` - The address that will own the new username record.
    ///
    /// # Panics
    ///
    /// * `FactoryError::Unauthorized` if the caller is not the configured auction contract.
    /// * `FactoryError::AlreadyDeployed` if the username is already registered.
    /// * `FactoryError::CoreContractNotConfigured` if the core contract has not been set.
    ///
    /// # Complexity
    ///
    /// O(1) - constant time storage lookups and persistence.
    pub fn deploy_username(env: Env, username_hash: BytesN<32>, owner: Address) {
        let auction_contract = match get_auction_contract(&env) {
            Some(address) => address,
            None => panic_with_error!(&env, FactoryError::Unauthorized),
        };
        auction_contract.require_auth();

        if has_username(&env, &username_hash) {
            panic_with_error!(&env, FactoryError::AlreadyDeployed);
        }

        let core_contract = match get_core_contract(&env) {
            Some(address) => address,
            None => panic_with_error!(&env, FactoryError::CoreContractNotConfigured),
        };

        let record = UsernameRecord {
            username_hash: username_hash.clone(),
            owner,
            registered_at: env.ledger().timestamp(),
            core_contract,
        };

        set_username(&env, &record.username_hash.clone(), &record);
        emit_username_deployed(
            &env,
            &record.username_hash,
            &record.owner,
            record.registered_at,
        );
    }

    /// Retrieves the record for a given username hash.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `username_hash` - The 32-byte hash identifying the username.
    ///
    /// # Returns
    ///
    /// * `Some(UsernameRecord)` if the username is registered.
    /// * `None` otherwise.
    ///
    /// # Complexity
    ///
    /// O(1) - single persistent storage lookup.
    pub fn get_username_record(env: Env, username_hash: BytesN<32>) -> Option<UsernameRecord> {
        get_username(&env, &username_hash)
    }

    /// Returns the owner of a deployed username.
    ///
    /// Convenience method to retrieve ownership info without the full record.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `username_hash` - The 32-byte hash identifying the username.
    ///
    /// # Returns
    ///
    /// * `Some(Address)` if the username is registered.
    /// * `None` otherwise.
    ///
    /// # Complexity
    ///
    /// O(1) - single persistent storage lookup.
    ///
    /// # Auth
    ///
    /// None - Read-only, safe for public polling.
    pub fn get_username_owner(env: Env, username_hash: BytesN<32>) -> Option<Address> {
        get_username(&env, &username_hash).map(|r| r.owner)
    }

    /// Retrieves the currently configured auction contract address.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    ///
    /// # Returns
    ///
    /// * `Some(Address)` if configured.
    /// * `None` otherwise.
    pub fn get_auction_contract(env: Env) -> Option<Address> {
        get_auction_contract(&env)
    }

    /// Retrieves the currently configured core contract address.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    ///
    /// # Returns
    ///
    /// * `Some(Address)` if configured.
    /// * `None` otherwise.
    pub fn get_core_contract(env: Env) -> Option<Address> {
        get_core_contract(&env)
    }
}
