use soroban_sdk::{contracttype, panic_with_error, Address, Bytes, BytesN, Env};

use crate::errors::ChainAddressError;
use crate::events::{CHAIN_ADD, CHAIN_REM};
use crate::registration::DataKey as CommitmentKey;
use crate::types::ChainType;

#[contracttype]
#[derive(Clone)]
pub enum ChainAddrKey {
    ChainAddress(BytesN<32>, ChainType),
}

pub struct AddressManager;

impl AddressManager {
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

        #[allow(deprecated)]
        env.events()
            .publish((CHAIN_ADD,), (username_hash, chain, address));
    }

    pub fn get_chain_address(
        env: Env,
        username_hash: BytesN<32>,
        chain: ChainType,
    ) -> Option<Bytes> {
        let key = ChainAddrKey::ChainAddress(username_hash, chain);
        env.storage().persistent().get(&key)
    }

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
