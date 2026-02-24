use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, Symbol};

// Storage Keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Commitment(BytesN<32>),
}

// Events
const REGISTER_EVENT: Symbol = symbol_short!("REGISTER");

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
            panic!("Commitment already registered");
        }

        // Store commitment -> address mapping
        env.storage().persistent().set(&key, &caller);

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
