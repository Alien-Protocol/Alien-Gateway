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

// This is a sample contract. Replace this placeholder with your own contract logic.
// A corresponding test example is available in `test.rs`.
//
// For comprehensive examples, visit <https://github.com/stellar/soroban-examples>.
// The repository includes use cases for the Stellar ecosystem, such as data storage on
// the blockchain, token swaps, liquidity pools, and more.
//
// Refer to the official documentation:
// <https://developers.stellar.org/docs/build/smart-contracts/overview>.
#[contractimpl]
impl Contract {
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "Hello"), to]
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
