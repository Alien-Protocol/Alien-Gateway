#![no_std]
use crate::types::{ChainAddress, ChainId, ChainRegistryEvent};
use soroban_sdk::{contract, contractimpl, Env, String, Address, Vec, Map};

#[contract]
pub struct ChainRegistry;

const OWNER_KEY: &str = "owner";
const ADDRESSES_KEY: &str = "addresses";

/// Storage key for chain addresses: "chain:{chain_id}"
fn chain_storage_key(chain: &ChainId) -> String {
    let env = Env::new();
    String::from_str(&env, &format!("chain:{}", chain.to_string()))
}

#[contractimpl]
impl ChainRegistry {
    /// Initialize the contract with an owner
    pub fn initialize(env: Env, owner: Address) {
        let owner_key = String::from_str(&env, OWNER_KEY);
        env.storage().instance().set(&owner_key, &owner);
    }

    /// Add a chain address for the specified chain
    /// 
    /// # Arguments
    /// * `chain` - The chain identifier (e.g., Ethereum, Bitcoin, Solana)
    /// * `address` - The address on that chain
    /// * `label` - A human-readable label for the address
    ///
    /// # Errors
    /// * Returns error if caller is not the owner
    /// * Returns error if address already exists for this chain
    pub fn add_chain_address(
        env: Env,
        chain: ChainId,
        address: String,
        label: String,
    ) -> Result<(), String> {
        // Verify owner authorization
        let owner_key = String::from_str(&env, OWNER_KEY);
        let owner: Address = env.storage().instance().get(&owner_key)
            .ok_or(String::from_str(&env, "Owner not set"))?;
        
        env.invoker().require_auth();
        if env.invoker() != owner {
            return Err(String::from_str(&env, "Only owner can add chain addresses"));
        }

        // Check for duplicates
        let chain_key = String::from_str(&env, &format!("chain:{}", chain.to_string()));
        let addresses_vec: Vec<ChainAddress> = env.storage()
            .instance()
            .get(&chain_key)
            .unwrap_or(Vec::new(&env));

        // Check if address already exists for this chain
        for existing in addresses_vec.iter() {
            if existing.address == address {
                return Err(String::from_str(&env, "Address already exists for this chain"));
            }
        }

        // Create new chain address
        let chain_address = ChainAddress {
            chain,
            address: address.clone(),
            label: label.clone(),
        };

        // Store the address
        let mut updated_addresses = addresses_vec.clone();
        updated_addresses.push_back(chain_address);
        env.storage().instance().set(&chain_key, &updated_addresses);

        // Emit event
        env.events().publish(
            (String::from_str(&env, "ChainRegistry"), String::from_str(&env, "ChainAddressAdded")),
            ChainRegistryEvent::ChainAddressAdded {
                chain,
                address,
                label,
            },
        );

        Ok(())
    }

    /// Retrieve all addresses for a specific chain
    pub fn get_chain_addresses(env: Env, chain: ChainId) -> Vec<ChainAddress> {
        let chain_key = String::from_str(&env, &format!("chain:{}", chain.to_string()));
        env.storage()
            .instance()
            .get(&chain_key)
            .unwrap_or(Vec::new(&env))
    }

    /// Get a specific address by index for a chain
    pub fn get_chain_address_at(
        env: Env,
        chain: ChainId,
        index: u32,
    ) -> Result<ChainAddress, String> {
        let addresses = Self::get_chain_addresses(env.clone(), chain);
        
        if index >= addresses.len() {
            return Err(String::from_str(&env, "Index out of bounds"));
        }

        Ok(addresses.get(index).unwrap())
    }

    /// Get count of addresses for a chain
    pub fn get_chain_address_count(env: Env, chain: ChainId) -> u32 {
        Self::get_chain_addresses(env, chain).len()
    }

    /// Check if an address exists for a specific chain
    pub fn has_chain_address(env: Env, chain: ChainId, address: String) -> bool {
        let addresses = Self::get_chain_addresses(env, chain);
        
        for existing in addresses.iter() {
            if existing.address == address {
                return true;
            }
        }
        
        false
    }

    /// Remove a chain address (owner only)
    pub fn remove_chain_address(
        env: Env,
        chain: ChainId,
        address: String,
    ) -> Result<(), String> {
        // Verify owner authorization
        let owner_key = String::from_str(&env, OWNER_KEY);
        let owner: Address = env.storage().instance().get(&owner_key)
            .ok_or(String::from_str(&env, "Owner not set"))?;
        
        env.invoker().require_auth();
        if env.invoker() != owner {
            return Err(String::from_str(&env, "Only owner can remove chain addresses"));
        }

        // Get current addresses
        let chain_key = String::from_str(&env, &format!("chain:{}", chain.to_string()));
        let mut addresses: Vec<ChainAddress> = env.storage()
            .instance()
            .get(&chain_key)
            .unwrap_or(Vec::new(&env));

        // Find and remove the address
        let original_len = addresses.len();
        let mut found = false;
        
        let mut new_addresses = Vec::new(&env);
        for addr in addresses.iter() {
            if addr.address != address {
                new_addresses.push_back(addr.clone());
            } else {
                found = true;
            }
        }

        if !found {
            return Err(String::from_str(&env, "Address not found"));
        }

        // Update storage
        env.storage().instance().set(&chain_key, &new_addresses);

        // Emit event
        env.events().publish(
            (String::from_str(&env, "ChainRegistry"), String::from_str(&env, "ChainAddressRemoved")),
            ChainRegistryEvent::ChainAddressRemoved {
                chain,
                address,
            },
        );

        Ok(())
    }

    /// Get the current owner
    pub fn get_owner(env: Env) -> Result<Address, String> {
        let owner_key = String::from_str(&env, OWNER_KEY);
        env.storage()
            .instance()
            .get(&owner_key)
            .ok_or(String::from_str(&env, "Owner not set"))
    }

    /// Change the owner (current owner only)
    pub fn change_owner(env: Env, new_owner: Address) -> Result<(), String> {
        let owner_key = String::from_str(&env, OWNER_KEY);
        let current_owner: Address = env.storage().instance().get(&owner_key)
            .ok_or(String::from_str(&env, "Owner not set"))?;
        
        env.invoker().require_auth();
        if env.invoker() != current_owner {
            return Err(String::from_str(&env, "Only current owner can change owner"));
        }

        env.storage().instance().set(&owner_key, &new_owner);

        // Emit event
        env.events().publish(
            (String::from_str(&env, "ChainRegistry"), String::from_str(&env, "OwnerChanged")),
            ChainRegistryEvent::OwnerChanged {
                new_owner: new_owner.clone(),
            },
        );

        Ok(())
    }
}
