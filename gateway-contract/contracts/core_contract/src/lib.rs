#![no_std]

mod errors;
mod events;
mod storage;
mod types;
pub mod events;
pub mod types;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, BytesN, Env,
};
use types::ResolveData;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractclient, contractimpl, panic_with_error, Address, BytesN, Env};

pub use crate::errors::ContractError;
pub use crate::events::UsernameRegistered;
pub use crate::types::{Proof, PublicSignals};

#[contract]
pub struct CoreContract;

#[contractclient(name = "VerifierContractClient")]
pub trait VerifierContract {
    fn verify_proof(env: Env, proof: Proof, public_signals: PublicSignals) -> bool;
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResolverError {
    NotFound = 1,
}

#[contractimpl]
impl CoreContract {
    pub fn init(env: Env, verifier: Address, root: BytesN<32>) {
        if storage::is_initialized(&env) {
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }

        storage::set_verifier(&env, &verifier);
        storage::set_root(&env, &root);
    }

    pub fn submit_proof(env: Env, proof: Proof, public_signals: PublicSignals) {
        let current_root = storage::get_root(&env)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::NotInitialized));

        if current_root != public_signals.old_root.clone() {
            panic_with_error!(&env, ContractError::RootMismatch);
        }

        if storage::has_commitment(&env, &public_signals.commitment) {
            panic_with_error!(&env, ContractError::DuplicateCommitment);
        }

        let verifier = storage::get_verifier(&env)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::NotInitialized));
        let verifier_client = VerifierContractClient::new(&env, &verifier);
        let is_valid = verifier_client.verify_proof(&proof, &public_signals);
        if !is_valid {
            panic_with_error!(&env, ContractError::InvalidProof);
    pub fn set_memo(env: Env, commitment: BytesN<32>, memo_id: u64) {
        let key = DataKey::Resolver(commitment);
        let mut data = env
            .storage()
            .persistent()
            .get::<DataKey, ResolveData>(&key)
            .unwrap_or_else(|| panic_with_error!(&env, ResolverError::NotFound));

        data.memo = Some(memo_id);
        env.storage().persistent().set(&key, &data);
    }

    pub fn resolve(env: Env, commitment: BytesN<32>) -> (Address, Option<u64>) {
        let key = DataKey::Resolver(commitment);

        match env.storage().persistent().get::<DataKey, ResolveData>(&key) {
            Some(data) => (data.wallet, data.memo),
            None => panic_with_error!(&env, ResolverError::NotFound),
        }

        storage::store_commitment(&env, &public_signals.commitment);
        storage::set_root(&env, &public_signals.new_root);

        UsernameRegistered {
            commitment: public_signals.commitment,
        }
        .publish(&env);
    }

    pub fn get_root(env: Env) -> Option<BytesN<32>> {
        storage::get_root(&env)
    }

    pub fn get_verifier(env: Env) -> Option<Address> {
        storage::get_verifier(&env)
    }

    pub fn has_commitment(env: Env, commitment: BytesN<32>) -> bool {
        storage::has_commitment(&env, &commitment)
    }
}

#[cfg(test)]
mod test;
