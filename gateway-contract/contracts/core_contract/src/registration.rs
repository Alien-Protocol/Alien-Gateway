use crate::errors::CoreError;
use crate::events::REGISTER_EVENT;
use crate::storage::{self, PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};
use soroban_sdk::{contracttype, panic_with_error, Address, BytesN, Env};

// Storage Keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Commitment(BytesN<32>),
}

pub struct Registration;

impl Registration {
    /// Registers a username commitment (Poseidon hash of username).
    ///
    /// Maps a username commitment to the caller's wallet address. The caller must authorize
    /// this transaction. Rejects duplicate commitments to ensure uniqueness.
    /// This is used to establish the initial link between a username and its owner.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `caller`: The address registering the commitment. Must be authorized.
    /// - `commitment`: A 32-byte Poseidon hash of the username.
    ///
    /// ### Errors
    /// - `AlreadyRegistered`: If the commitment has already been registered.
    ///
    /// ### Events
    /// - Emits `REGISTER_EVENT` with (commitment, owner).
    pub fn register(env: Env, caller: Address, commitment: BytesN<32>) {
        // Require authentication from the caller
        caller.require_auth();

        // Check if commitment already exists
        let key = DataKey::Commitment(commitment.clone());
        if env.storage().persistent().has(&key) {
            panic_with_error!(&env, CoreError::AlreadyRegistered);
        }

        // Store commitment -> address mapping
        env.storage().persistent().set(&key, &caller);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // Store registration timestamp
        storage::set_created_at(&env, &commitment, env.ledger().timestamp());

        // Emit registration event
        #[allow(deprecated)]
        env.events()
            .publish((REGISTER_EVENT,), (commitment, caller));
    }

    /// Retrieves the owner address for a given commitment.
    ///
    /// Returns the wallet address associated with the commitment, or None if not yet registered.
    /// This is a read-only query operation with no authentication requirement.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `commitment`: The 32-byte username commitment to look up.
    ///
    /// ### Returns
    /// - `Some(Address)` if the commitment is registered.
    /// - `None` if the commitment is not found.
    pub fn get_owner(env: Env, commitment: BytesN<32>) -> Option<Address> {
        let key = DataKey::Commitment(commitment);
        env.storage().persistent().get(&key)
    }

    /// Retrieves the ledger timestamp at which a commitment was first registered.
    ///
    /// Returns the Unix timestamp (seconds) recorded at registration time, or None if the
    /// commitment has never been registered.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `commitment`: The 32-byte username commitment to look up.
    ///
    /// ### Returns
    /// - `Some(u64)` with the registration ledger timestamp.
    /// - `None` if the commitment is not found.
    pub fn get_created_at(env: Env, commitment: BytesN<32>) -> Option<u64> {
        storage::get_created_at(&env, &commitment)
    }
}
