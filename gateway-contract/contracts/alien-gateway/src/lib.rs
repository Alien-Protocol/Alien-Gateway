#![no_std]
use soroban_sdk::{contract, contractimpl, vec, Address, BytesN, Env, String, Vec};

pub mod address_manager;
pub mod contract_core;
pub mod registration;
pub mod smt_root;
pub mod types;

pub use address_manager::AddressManager;
pub use contract_core::CoreContract;
pub use registration::Registration;
pub use smt_root::SmtRoot;

#[contract]
pub struct Contract;

//
// ---------------- STORAGE KEY ----------------
//

#[contracttype]
pub enum DataKey {
    Resolver(BytesN<32>),
}

//
// ---------------- STORED VALUE ----------------
//

#[contracttype]
#[derive(Clone)]
pub struct ResolveData {
    pub wallet: Address,
    pub memo: Option<u64>,
}

//
// ---------------- ERRORS ----------------
//

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResolverError {
    NotFound = 1,
}

//
// ---------------- CONTRACT IMPLEMENTATION ----------------
//

#[contractimpl]
impl Contract {
    // Register commitment → wallet (+ optional memo)
    pub fn register(env: Env, commitment: BytesN<32>, wallet: Address, memo: Option<u64>) {
        let data = ResolveData { wallet, memo };

        env.storage()
            .persistent()
            .set(&DataKey::Resolver(commitment), &data);
    }

    // Resolve commitment → wallet (+ memo)
    pub fn resolve(env: Env, commitment: BytesN<32>) -> ResolveData {
        match env
            .storage()
            .persistent()
            .get::<_, ResolveData>(&DataKey::Resolver(commitment.clone()))
        {
            Some(data) => data,
            None => panic_with_error!(&env, ResolverError::NotFound),
        }
    }

    /// Register a username commitment (Poseidon hash of username).
    /// Rejects duplicate commitments.
    /// Maps commitment to caller's wallet address.
    /// Emits REGISTER event on success.
    pub fn register(env: Env, caller: Address, commitment: BytesN<32>) {
        Registration::register(env, caller, commitment)
    }

    /// Get the owner address for a given commitment.
    pub fn get_commitment_owner(env: Env, commitment: BytesN<32>) -> Option<Address> {
        Registration::get_owner(env, commitment)
    }
}

mod test;
