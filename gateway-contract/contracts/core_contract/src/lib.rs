#![no_std]

pub mod address_manager;
pub mod events;
pub mod registration;
pub mod types;

use address_manager::AddressManager;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, BytesN, Env,
};
use types::{PrivacyMode, ResolveData};

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
    pub fn register_resolver(env: Env, commitment: BytesN<32>, wallet: Address, memo: Option<u64>) {
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

    pub fn set_privacy_mode(env: Env, username_hash: BytesN<32>, mode: PrivacyMode) {
        AddressManager::set_privacy_mode(env, username_hash, mode);
    }

    pub fn get_privacy_mode(env: Env, username_hash: BytesN<32>) -> PrivacyMode {
        AddressManager::get_privacy_mode(env, username_hash)
    }

    pub fn resolve(env: Env, commitment: BytesN<32>) -> (Address, Option<u64>) {
        let key = DataKey::Resolver(commitment.clone());

        match env.storage().persistent().get::<DataKey, ResolveData>(&key) {
            Some(data) => {
                // If Private, return shielded address (contract's own address)
                if AddressManager::get_privacy_mode(env.clone(), commitment) == PrivacyMode::Private
                {
                    (env.current_contract_address(), data.memo)
                } else {
                    (data.wallet, data.memo)
                }
            }
            None => panic_with_error!(&env, ResolverError::NotFound),
        }
    }
}

#[cfg(test)]
mod test;
