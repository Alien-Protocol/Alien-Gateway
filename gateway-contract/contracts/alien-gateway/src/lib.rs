#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, BytesN, Env,
};

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
}

mod test;
