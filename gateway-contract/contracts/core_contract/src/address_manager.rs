use soroban_sdk::{contracttype, panic_with_error, Address, Bytes, BytesN, Env, Vec};

use crate::errors::{ChainAddressError, CoreError};
use crate::events::{shielded_add_event, CHAIN_ADD, CHAIN_REM};
use crate::registration::{DataKey as CommitmentKey, Registration};
use crate::storage::{self, PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};
use crate::types::ChainType;

#[contracttype]
#[derive(Clone)]
pub enum ChainAddrKey {
    ChainAddress(BytesN<32>, ChainType),
}

pub struct AddressManager;

impl AddressManager {
    /// Adds a blockchain address for a registered commitment on a specified chain.
    ///
    /// Links a non-Stellar blockchain address (Bitcoin, Ethereum, Solana, Cosmos) to the username.
    /// Only the commitment owner can authorize this action. Validates the address format for the chain.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `caller`: The commitment owner authorizing the addition. Must be authorized.
    /// - `username_hash`: The 32-byte username commitment.
    /// - `chain`: The blockchain type (EVM, Bitcoin, Solana, Cosmos).
    /// - `address`: The blockchain address as bytes (format validated per chain).
    ///
    /// ### Errors
    /// - `NotRegistered`: If the username commitment is not registered.
    /// - `Unauthorized`: If the caller is not the commitment owner.
    /// - `InvalidAddress`: If the address format is invalid for the specified chain.
    ///
    /// ### Events
    /// - Emits `CHAIN_ADD` event with (username_hash, chain, address).
    pub fn add_chain_address(
        env: Env,
        caller: Address,
        username_hash: BytesN<32>,
        chain: ChainType,
        address: Bytes,
    ) {
        caller.require_auth();

        let owner_key = CommitmentKey::Commitment(username_hash.clone());
        let owner: Address = env
            .storage()
            .persistent()
            .get(&owner_key)
            .unwrap_or_else(|| panic_with_error!(&env, ChainAddressError::NotRegistered));

        if owner != caller {
            panic_with_error!(&env, ChainAddressError::Unauthorized);
        }

        if !Self::validate_address(&chain, &address) {
            panic_with_error!(&env, ChainAddressError::InvalidAddress);
        }

        let key = ChainAddrKey::ChainAddress(username_hash.clone(), chain.clone());
        env.storage().persistent().set(&key, &address);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        #[allow(deprecated)]
        env.events()
            .publish((CHAIN_ADD,), (username_hash, chain, address));
    }

    /// Retrieves the blockchain address for a commitment on a specified chain.
    ///
    /// Returns the stored address for the given commitment and blockchain type, if set.
    /// This is a read-only operation with no authentication requirement.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `username_hash`: The 32-byte username commitment.
    /// - `chain`: The blockchain type to query.
    ///
    /// ### Returns
    /// - `Some(Bytes)` if an address exists for this chain.
    /// - `None` if no address is set for this chain.
    pub fn get_chain_address(
        env: Env,
        username_hash: BytesN<32>,
        chain: ChainType,
    ) -> Option<Bytes> {
        let key = ChainAddrKey::ChainAddress(username_hash, chain);
        env.storage().persistent().get(&key)
    }

    /// Removes a blockchain address for a commitment on a specified chain.
    ///
    /// Deletes the stored address for the given commitment and blockchain type.
    /// Only the commitment owner can authorize this action.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `caller`: The commitment owner authorizing the removal. Must be authorized.
    /// - `username_hash`: The 32-byte username commitment.
    /// - `chain`: The blockchain type to remove the address from.
    ///
    /// ### Errors
    /// - `NotRegistered`: If the username commitment is not registered.
    /// - `Unauthorized`: If the caller is not the commitment owner.
    ///
    /// ### Events
    /// - Emits `CHAIN_REM` event with (username_hash, chain).
    pub fn remove_chain_address(
        env: Env,
        caller: Address,
        username_hash: BytesN<32>,
        chain: ChainType,
    ) {
        caller.require_auth();

        let owner_key = CommitmentKey::Commitment(username_hash.clone());
        let owner: Address = env
            .storage()
            .persistent()
            .get(&owner_key)
            .unwrap_or_else(|| panic_with_error!(&env, ChainAddressError::NotRegistered));

        if owner != caller {
            panic_with_error!(&env, ChainAddressError::Unauthorized);
        }

        let key = ChainAddrKey::ChainAddress(username_hash.clone(), chain.clone());
        env.storage().persistent().remove(&key);

        #[allow(deprecated)]
        env.events().publish((CHAIN_REM,), (username_hash, chain));
    }

    /// Adds a Stellar address (receiver) for a registered commitment.
    ///
    /// Links a Stellar wallet address to the username, enabling payment resolution on Stellar.
    /// Only the commitment owner can authorize this action. This address is separate from
    /// the owner address and represents where payments should be received.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `caller`: The commitment owner authorizing the addition. Must be authorized.
    /// - `username_hash`: The 32-byte username commitment.
    /// - `stellar_address`: The Stellar address to link for payment reception.
    ///
    /// ### Errors
    /// - `NotFound`: If the commitment is not registered.
    ///
    /// ### Events
    /// - No explicit event emitted (stored in persistent storage).
    pub fn add_stellar_address(
        env: Env,
        caller: Address,
        username_hash: BytesN<32>,
        stellar_address: Address,
    ) {
        caller.require_auth();

        let owner = Registration::get_owner(env.clone(), username_hash.clone())
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound));

        if owner != caller {
            panic_with_error!(&env, CoreError::NotFound);
        }

        let mut linked_addresses: Vec<Address> = env
            .storage()
            .persistent()
            .get(&storage::DataKey::StellarAddresses(username_hash.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        linked_addresses.push_back(stellar_address.clone());
        env.storage().persistent().set(
            &storage::DataKey::StellarAddresses(username_hash.clone()),
            &linked_addresses,
        );

        env.storage().persistent().set(
            &storage::DataKey::StellarAddress(username_hash),
            &stellar_address,
        );
    }

    pub fn get_stellar_addresses(env: Env, username_hash: BytesN<32>) -> Vec<Address> {
        if Registration::get_owner(env.clone(), username_hash.clone()).is_none() {
            panic_with_error!(&env, CoreError::NotFound);
        }

        env.storage()
            .persistent()
            .get::<storage::DataKey, Vec<Address>>(&storage::DataKey::StellarAddresses(
                username_hash,
            ))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Resolves a commitment to its linked Stellar address.
    ///
    /// Returns the Stellar address designated for receiving payments for this username.
    /// This is a read-only query that must have a valid linked address.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `username_hash`: The 32-byte username commitment.
    ///
    /// ### Returns
    /// The Stellar address linked to this commitment.
    ///
    /// ### Errors
    /// - `NotFound`: If the commitment is not registered.
    /// - `NoAddressLinked`: If no Stellar address has been set for this commitment.
    pub fn resolve_stellar(env: Env, username_hash: BytesN<32>) -> Address {
        if Registration::get_owner(env.clone(), username_hash.clone()).is_none() {
            panic_with_error!(&env, CoreError::NotFound);
        }

        env.storage()
            .persistent()
            .get::<storage::DataKey, Address>(&storage::DataKey::StellarAddress(username_hash))
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NoAddressLinked))
    }

    /// Adds a shielded (privacy-preserving) address commitment for a commitment.
    ///
    /// Stores a privacy commitment (e.g., a hash of a private address) that enables
    /// shielded payment routing. Only the commitment owner can authorize this action.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `caller`: The commitment owner authorizing the addition. Must be authorized.
    /// - `username_hash`: The 32-byte username commitment.
    /// - `address_commitment`: A 32-byte privacy commitment (e.g., hash of private address).
    ///
    /// ### Errors
    /// - `NotFound`: If the commitment is not registered.
    /// - `Unauthorized`: If the caller is not the commitment owner.
    ///
    /// ### Events
    /// - Emits shielded add event with (username_hash, address_commitment).
    pub fn add_shielded_address(
        env: Env,
        caller: Address,
        username_hash: BytesN<32>,
        address_commitment: BytesN<32>,
    ) {
        caller.require_auth();
        let owner = Registration::get_owner(env.clone(), username_hash.clone())
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound));
        if owner != caller {
            panic_with_error!(&env, CoreError::Unauthorized);
        }
        storage::set_shielded_address(&env, &username_hash, &address_commitment);
        #[allow(deprecated)]
        env.events().publish(
            (shielded_add_event(&env),),
            (username_hash, address_commitment),
        );
    }

    /// Retrieves the shielded address commitment for a commitment, if set.
    ///
    /// Returns the stored privacy commitment for the given username, or None if not set.
    /// This is a read-only query operation with no authentication requirement.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `username_hash`: The 32-byte username commitment.
    ///
    /// ### Returns
    /// - `Some(BytesN<32>)` if a shielded address commitment exists.
    /// - `None` if no shielded address has been set.
    pub fn get_shielded_address(env: Env, username_hash: BytesN<32>) -> Option<BytesN<32>> {
        storage::get_shielded_address(&env, &username_hash)
    }

    /// Checks if a shielded address commitment has been set for a commitment.
    ///
    /// Returns true if a shielded address commitment exists for this username, false otherwise.
    /// This is a read-only query with no authentication requirement.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `username_hash`: The 32-byte username commitment.
    ///
    /// ### Returns
    /// `true` if a shielded address is set, `false` otherwise.
    pub fn is_shielded(env: Env, username_hash: BytesN<32>) -> bool {
        storage::has_shielded_address(&env, &username_hash)
    }

    /// (Internal) Validates a blockchain address format for a given chain.
    ///
    /// This private helper function validates address format constraints per blockchain type:
    /// - EVM: 42 bytes starting with "0x"
    /// - Bitcoin: 25-62 bytes
    /// - Solana: 32-44 bytes
    /// - Cosmos: 39-45 bytes
    fn validate_address(chain: &ChainType, address: &Bytes) -> bool {
        let len = address.len();
        match chain {
            ChainType::Evm => {
                len == 42 && address.get(0) == Some(0x30) && address.get(1) == Some(0x78)
            }
            ChainType::Bitcoin => (25..=62).contains(&len),
            ChainType::Solana => (32..=44).contains(&len),
            ChainType::Cosmos => (39..=45).contains(&len),
        }
    }
}
