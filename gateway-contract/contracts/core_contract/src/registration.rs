use crate::errors::CoreError;
use crate::events::REGISTER_EVENT;
use crate::storage::{PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};
use soroban_sdk::{contracttype, panic_with_error, Address, BytesN, Env};

// Storage Keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Commitment(BytesN<32>),
}

pub struct Registration;

impl Registration {
    /// Register a username commitment (Poseidon hash of username).
    /// Maps the commitment to the caller's wallet address.
    /// Rejects duplicate commitments.
    /// Emits REGISTER event with (commitment, owner).
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

        // Emit registration event
        #[allow(deprecated)]
        env.events()
            .publish((REGISTER_EVENT,), (commitment, caller));
    }

    /// Get the owner address for a given commitment.
    /// Returns None if the commitment is not registered.
    pub fn get_owner(env: Env, commitment: BytesN<32>) -> Option<Address> {
        let key = DataKey::Commitment(commitment);
        env.storage().persistent().get(&key)
    }
}
