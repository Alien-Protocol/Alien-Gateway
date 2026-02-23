use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

use crate::contract_core;
use crate::types::AddressMetadata;

// Storage Keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Address(Address),
    MasterAddress,
    StellarAddress(Address),
}

// Event Symbols
const MASTER_SET: Symbol = symbol_short!("MSTR_SET");
const ADDR_ADDED: Symbol = symbol_short!("ADDR_ADD");

pub struct AddressManager;

impl AddressManager {
    // Initialize contract with owner (sets shared owner for auth middleware)
    pub fn init(env: Env, owner: Address) {
        if env.storage().instance().has(&contract_core::DataKey::Owner) {
            panic!("Already initialized");
        }
        env.storage()
            .instance()
            .set(&contract_core::DataKey::Owner, &owner);
    }

    // Helper: check owner via shared auth middleware
    fn require_owner(env: &Env) {
        contract_core::auth::require_owner(env);
    }

    // Helper: check address exists
    fn address_exists(env: &Env, address: &Address) -> bool {
        env.storage()
            .instance()
            .has(&DataKey::Address(address.clone()))
    }

    // Optional helper to register address
    pub fn register_address(env: Env, address: Address) {
        Self::require_owner(&env);
        env.storage()
            .instance()
            .set(&DataKey::Address(address.clone()), &true);
    }

    // ✅ Main Function
    pub fn set_master_stellar_address(env: Env, address: Address) {
        Self::require_owner(&env);

        // Address must exist
        if !Self::address_exists(&env, &address) {
            panic!("Address does not exist");
        }

        // Unset previous master (if any)
        if env.storage().instance().has(&DataKey::MasterAddress) {
            env.storage().instance().remove(&DataKey::MasterAddress);
        }

        // Set new master
        env.storage()
            .instance()
            .set(&DataKey::MasterAddress, &address);

        // Emit Event
        #[allow(deprecated)]
        env.events().publish((MASTER_SET,), address);
    }

    // Getter
    pub fn get_master(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::MasterAddress)
    }

    // Add a Stellar address with a label. Owner-only. Prevents duplicates.
    pub fn add_stellar_address(env: Env, address: Address, label: Symbol) {
        Self::require_owner(&env);
        if env
            .storage()
            .instance()
            .has(&DataKey::StellarAddress(address.clone()))
        {
            panic!("Address already exists");
        }
        let metadata = AddressMetadata {
            label: label.clone(),
        };
        env.storage()
            .instance()
            .set(&DataKey::StellarAddress(address.clone()), &metadata);
        #[allow(deprecated)]
        env.events().publish((ADDR_ADDED,), (address, label));
    }

    // Retrieve metadata for a registered Stellar address.
    pub fn get_stellar_address(env: Env, address: Address) -> Option<AddressMetadata> {
        env.storage()
            .instance()
            .get(&DataKey::StellarAddress(address))
    }
}
