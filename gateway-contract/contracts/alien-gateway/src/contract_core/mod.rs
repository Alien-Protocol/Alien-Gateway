use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, Symbol};

pub mod auth;

// Storage Keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Username,
    Owner,
    CreatedAt,
}

// Events
const INIT_EVENT: Symbol = symbol_short!("INIT");
const TRANSFER_EVENT: Symbol = symbol_short!("TRANSFER");

pub struct CoreContract;

impl CoreContract {
    /// Initialize the contract. Owner is the address that will have write access (auth).
    pub fn init(env: Env, username: Symbol, owner: Address) {
        // Prevent re-init
        if env.storage().instance().has(&DataKey::Owner) {
            panic!("Contract already initialized");
        }

        // Validate username: must not be empty (symbol_short!("") is the empty symbol)
        if username == symbol_short!("") {
            panic!("Username cannot be empty");
        }

        // Store values
        env.storage().instance().set(&DataKey::Username, &username);
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage()
            .instance()
            .set(&DataKey::CreatedAt, &env.ledger().timestamp());

        // Emit event
        #[allow(deprecated)]
        env.events().publish((INIT_EVENT,), (username, owner));
    }

    // Getters
    pub fn get_username(env: Env) -> Symbol {
        env.storage().instance().get(&DataKey::Username).unwrap()
    }

    pub fn get_owner(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Owner).unwrap()
    }

    pub fn get_created_at(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::CreatedAt).unwrap()
    }

    /// Transfer ownership to a new address. Caller must be current owner.
    pub fn transfer_ownership(env: Env, new_owner: Address) {
        auth::require_owner(&env);
        env.storage().instance().set(&DataKey::Owner, &new_owner);
    }

    /// Transfer the username commitment to a new owner and update the SMT root.
    /// Only the current owner can call this function.
    /// Emits a TRANSFER event with (username, old_owner, new_owner).
    pub fn transfer(env: Env, new_owner: Address, new_smt_root: BytesN<32>) {
        auth::require_owner(&env);

        let old_owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        if old_owner == new_owner {
            panic!("New owner must differ from current owner");
        }

        let username: Symbol = env.storage().instance().get(&DataKey::Username).unwrap();

        env.storage().instance().set(&DataKey::Owner, &new_owner);

        #[allow(deprecated)]
        env.events()
            .publish((TRANSFER_EVENT,), (username, old_owner, new_owner));

        crate::smt_root::SmtRoot::update_root(env.clone(), new_smt_root);
    }
}
