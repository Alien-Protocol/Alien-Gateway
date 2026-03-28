#![no_std]

pub mod address_manager;
pub mod errors;
pub mod events;
pub mod registration;
pub mod smt_root;
pub mod storage;
pub mod types;
pub mod zk_verifier;

#[cfg(test)]
mod test;

use address_manager::AddressManager;
use errors::CoreError;
use events::{privacy_set_event, shielded_add_event, INIT_EVENT, REGISTER_EVENT, TRANSFER_EVENT};
use registration::Registration;
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Bytes, BytesN, Env};
use types::{ChainType, PrivacyMode, PublicSignals, ResolveData};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    /// Initializes the contract by setting the owner address.
    ///
    /// Must be called once after deployment. Panics with `AlreadyInitialized`
    /// if called again on an already-initialized instance.
    pub fn initialize(env: Env, owner: Address) {
        owner.require_auth();

        if storage::is_initialized(&env) {
            panic_with_error!(&env, errors::CoreError::AlreadyInitialized);
        }

        storage::set_owner(&env, &owner);

        #[allow(deprecated)]
        env.events().publish((INIT_EVENT,), (owner,));
    }

    /// Returns the contract owner address set during `initialize`.
    ///
    /// Panics with `NotFound` if `initialize` has not been called yet.
    pub fn get_contract_owner(env: Env) -> Address {
        storage::get_owner(&env)
            .unwrap_or_else(|| panic_with_error!(&env, errors::CoreError::NotFound))
    }

    pub fn get_smt_root(env: Env) -> BytesN<32> {
        smt_root::SmtRoot::get_root(env.clone())
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::RootNotSet))
    }

    pub fn register_resolver(
        env: Env,
        caller: Address,
        commitment: BytesN<32>,
        proof: Bytes,
        public_signals: PublicSignals,
    ) {
        caller.require_auth();

        let key = storage::DataKey::Resolver(commitment.clone());
        if env.storage().persistent().has(&key) {
            panic_with_error!(&env, CoreError::DuplicateCommitment);
        }

        let current_root = smt_root::SmtRoot::get_root(env.clone())
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::RootNotSet));
        if public_signals.old_root != current_root {
            panic_with_error!(&env, CoreError::StaleRoot);
        }

        if !zk_verifier::ZkVerifier::verify_groth16_proof(&env, &proof, &public_signals) {
            panic_with_error!(&env, CoreError::InvalidProof);
        }

        let data = ResolveData {
            wallet: caller.clone(),
            memo: None,
        };
        env.storage().persistent().set(&key, &data);
        env.storage().persistent().extend_ttl(
            &key,
            storage::PERSISTENT_LIFETIME_THRESHOLD,
            storage::PERSISTENT_BUMP_AMOUNT,
        );

        smt_root::SmtRoot::update_root(&env, public_signals.new_root);

        #[allow(deprecated)]
        env.events()
            .publish((REGISTER_EVENT,), (commitment, caller));
    }

    pub fn set_memo(env: Env, commitment: BytesN<32>, memo_id: u64) {
        let key = storage::DataKey::Resolver(commitment);
        let mut data = env
            .storage()
            .persistent()
            .get::<storage::DataKey, ResolveData>(&key)
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound));

        data.memo = Some(memo_id);
        env.storage().persistent().set(&key, &data);
        env.storage().persistent().extend_ttl(
            &key,
            storage::PERSISTENT_LIFETIME_THRESHOLD,
            storage::PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn set_privacy_mode(env: Env, username_hash: BytesN<32>, mode: PrivacyMode) {
        let owner = Registration::get_owner(env.clone(), username_hash.clone())
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound));
        owner.require_auth();

        storage::set_privacy_mode(&env, &username_hash, &mode);

        #[allow(deprecated)]
        env.events()
            .publish((privacy_set_event(&env),), (username_hash, mode));
    }

    pub fn get_privacy_mode(env: Env, username_hash: BytesN<32>) -> PrivacyMode {
        storage::get_privacy_mode(&env, &username_hash)
    }

    pub fn resolve(env: Env, commitment: BytesN<32>) -> (Address, Option<u64>) {
        match env
            .storage()
            .persistent()
            .get::<storage::DataKey, ResolveData>(&storage::DataKey::Resolver(commitment.clone()))
        {
            Some(data) => {
                if storage::get_privacy_mode(&env, &commitment) == PrivacyMode::Shielded {
                    (env.current_contract_address(), data.memo)
                } else {
                    (data.wallet, data.memo)
                }
            }
            None => panic_with_error!(&env, CoreError::NotFound),
        }
    }

    pub fn register(env: Env, caller: Address, commitment: BytesN<32>) {
        Registration::register(env, caller, commitment);
    }

    pub fn get_owner(env: Env, commitment: BytesN<32>) -> Option<Address> {
        Registration::get_owner(env, commitment)
    }

    pub fn add_chain_address(
        env: Env,
        caller: Address,
        username_hash: BytesN<32>,
        chain: ChainType,
        address: Bytes,
    ) {
        AddressManager::add_chain_address(env, caller, username_hash, chain, address);
    }

    pub fn get_chain_address(
        env: Env,
        username_hash: BytesN<32>,
        chain: ChainType,
    ) -> Option<Bytes> {
        AddressManager::get_chain_address(env, username_hash, chain)
    }

    pub fn remove_chain_address(
        env: Env,
        caller: Address,
        username_hash: BytesN<32>,
        chain: ChainType,
    ) {
        AddressManager::remove_chain_address(env, caller, username_hash, chain);
    }

    pub fn add_stellar_address(
        env: Env,
        caller: Address,
        username_hash: BytesN<32>,
        stellar_address: Address,
    ) {
        caller.require_auth();

        let owner = Registration::get_owner(env.clone(), username_hash.clone())
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound));

        if owner != caller {
            panic_with_error!(&env, CoreError::NotFound);
        }

        let key = storage::DataKey::StellarAddress(username_hash);
        env.storage().persistent().set(&key, &stellar_address);
        env.storage().persistent().extend_ttl(
            &key,
            storage::PERSISTENT_LIFETIME_THRESHOLD,
            storage::PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Transfer ownership of a commitment to a new owner.
    /// The caller must be the current registered owner.
    /// Panics with `Unauthorized` if caller is not the owner.
    /// Panics with `SameOwner` if new_owner equals the current owner.
    pub fn transfer_ownership(
        env: Env,
        caller: Address,
        commitment: BytesN<32>,
        new_owner: Address,
    ) {
        caller.require_auth();

        let key = registration::DataKey::Commitment(commitment.clone());
        let current_owner: Address = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound));

        if caller != current_owner {
            panic_with_error!(&env, CoreError::Unauthorized);
        }

        if new_owner == current_owner {
            panic_with_error!(&env, CoreError::SameOwner);
        }

        env.storage().persistent().set(&key, &new_owner);
        env.storage().persistent().extend_ttl(
            &key,
            storage::PERSISTENT_LIFETIME_THRESHOLD,
            storage::PERSISTENT_BUMP_AMOUNT,
        );

        #[allow(deprecated)]
        env.events()
            .publish((TRANSFER_EVENT,), (commitment, caller, new_owner));
    }

    /// Transfer ownership of a commitment with ZK proof verification and SMT root update.
    /// The caller must be the current registered owner.
    /// Panics with `Unauthorized` if caller is not the owner.
    /// Panics with `SameOwner` if new_owner equals the current owner.
    /// Panics with `StaleRoot` if public_signals.old_root does not match the on-chain root.
    pub fn transfer(
        env: Env,
        caller: Address,
        commitment: BytesN<32>,
        new_owner: Address,
        proof: Bytes,
        public_signals: PublicSignals,
    ) {
        caller.require_auth();

        let key = registration::DataKey::Commitment(commitment.clone());
        let current_owner: Address = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound));

        if caller != current_owner {
            panic_with_error!(&env, CoreError::Unauthorized);
        }

        if new_owner == current_owner {
            panic_with_error!(&env, CoreError::SameOwner);
        }

        // SMT root consistency
        let current_root = smt_root::SmtRoot::get_root(env.clone())
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::RootNotSet));
        if public_signals.old_root != current_root {
            panic_with_error!(&env, CoreError::StaleRoot);
        }

        // ZK proof verification (Phase 4 stub)
        if !zk_verifier::ZkVerifier::verify_groth16_proof(&env, &proof, &public_signals) {
            panic_with_error!(&env, CoreError::InvalidProof);
        }

        // Update ownership
        env.storage().persistent().set(&key, &new_owner);
        env.storage().persistent().extend_ttl(
            &key,
            storage::PERSISTENT_LIFETIME_THRESHOLD,
            storage::PERSISTENT_BUMP_AMOUNT,
        );

        // Advance SMT root
        smt_root::SmtRoot::update_root(&env, public_signals.new_root);

        // Emit TRANSFER event
        #[allow(deprecated)]
        env.events()
            .publish((TRANSFER_EVENT,), (commitment, caller, new_owner));
    }

    /// Stores a ZK commitment as the shielded address for the given username.
    ///
    /// Only the registered owner of `username_hash` may call this function.
    /// The raw address is never stored — only the commitment (ZK proof handle).
    /// Emits a `SHIELDED_ADD` event on success.
    pub fn add_shielded_address(
        env: Env,
        caller: Address,
        username_hash: BytesN<32>,
        address_commitment: BytesN<32>,
    ) {
        caller.require_auth();

        let owner = registration::Registration::get_owner(env.clone(), username_hash.clone())
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound));

        if owner != caller {
            panic_with_error!(&env, CoreError::Unauthorized);
        }

        storage::set_shielded_address(&env, &username_hash, &address_commitment);

        #[allow(deprecated)]
        env.events().publish(
            (shielded_add_event(&env),),
            (username_hash, address_commitment),
        );
    }

    /// Returns the shielded address commitment for the given username hash, if any.
    pub fn get_shielded_address(env: Env, username_hash: BytesN<32>) -> Option<BytesN<32>> {
        storage::get_shielded_address(&env, &username_hash)
    }

    /// Returns `true` if a shielded address commitment has been stored for the given username hash.
    pub fn is_shielded(env: Env, username_hash: BytesN<32>) -> bool {
        storage::has_shielded_address(&env, &username_hash)
    }

    /// Resolve a username hash to its primary linked Stellar address.
    ///
    /// Returns `NotFound` if the username hash is not registered.
    /// Returns `NoAddressLinked` if registered but no primary Stellar address has been set.
    pub fn resolve_stellar(env: Env, username_hash: BytesN<32>) -> Address {
        if Registration::get_owner(env.clone(), username_hash.clone()).is_none() {
            panic_with_error!(&env, CoreError::NotFound);
        }

        env.storage()
            .persistent()
            .get::<storage::DataKey, Address>(&storage::DataKey::StellarAddress(username_hash))
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NoAddressLinked))
    }
}
