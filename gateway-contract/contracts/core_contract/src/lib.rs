#![no_std]

pub mod events;
pub mod types;
pub mod zk_verifiers;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, vec, Address, BytesN,
    Env,
};
use types::ResolveData;
use zk_verifiers::{verify, Groth16Proof};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ZkError {
    InvalidProof = 2,
}

#[contract]
pub struct Contract;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Resolver(BytesN<32>),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResolverError {
    NotFound = 1,
}

#[contractimpl]
impl Contract {
    pub fn register_resolver(
        env: Env,
        commitment: BytesN<32>,
        wallet: Address,
        memo: Option<u64>,
        proof: Groth16Proof,
    ) {
        let public_inputs = vec![&env, commitment.clone()];

        if !verify(env.clone(), proof, public_inputs) {
            panic_with_error!(&env, ZkError::InvalidProof);
        }

        let key = DataKey::Resolver(commitment);
        let data = ResolveData { wallet, memo };

        env.storage().persistent().set(&key, &data);
    }

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
    }
}

#[cfg(test)]
mod test;
