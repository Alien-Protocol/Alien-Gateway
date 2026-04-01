#![no_std]

pub mod address_manager;
pub mod admin;
pub mod errors;
pub mod events;
pub mod registration;
pub mod resolver;
pub mod smt_root;
pub mod storage;
pub mod transfer;
pub mod types;
pub mod zk_verifier;

#[cfg(test)]
mod test;

use address_manager::AddressManager;
use admin::Admin;
use registration::Registration;
use resolver::Resolver;
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env};
use transfer::Transfer;
use types::{ChainType, PrivacyMode, PublicSignals};

#[contract]
pub struct Contract;

#[rustfmt::skip]
#[contractimpl]
impl Contract {
    /// Initializes the contract with the owner. See [admin::Admin::initialize].
    pub fn initialize(e: Env, o: Address) { Admin::initialize(e, o) }

    /// Retrieves the contract owner. See [admin::Admin::get_contract_owner].
    pub fn get_contract_owner(e: Env) -> Address { Admin::get_contract_owner(e) }

    /// Retrieves the current SMT root. See [admin::Admin::get_smt_root].
    pub fn get_smt_root(e: Env) -> BytesN<32> { Admin::get_smt_root(e) }

    /// Updates the SMT root with owner authorization. See [admin::Admin::update_smt_root].
    pub fn update_smt_root(e: Env, r: BytesN<32>) { Admin::update_smt_root(e, r) }

    /// Registers a username with ZK proof validation. See [resolver::Resolver::register_resolver].
    pub fn register_resolver(e: Env, c: Address, h: BytesN<32>, p: Bytes, s: PublicSignals) { Resolver::register_resolver(e, c, h, p, s); }

    /// Sets a memo for a registered commitment. See [resolver::Resolver::set_memo].
    pub fn set_memo(e: Env, c: BytesN<32>, m: u64) { Resolver::set_memo(e, c, m) }

    /// Sets the privacy mode for a commitment. See [resolver::Resolver::set_privacy_mode].
    pub fn set_privacy_mode(e: Env, h: BytesN<32>, m: PrivacyMode) { Resolver::set_privacy_mode(e, h, m); }

    /// Retrieves the privacy mode for a commitment. See [resolver::Resolver::get_privacy_mode].
    pub fn get_privacy_mode(e: Env, h: BytesN<32>) -> PrivacyMode { Resolver::get_privacy_mode(e, h) }

    /// Resolves a commitment to a wallet and memo. See [resolver::Resolver::resolve].
    pub fn resolve(e: Env, c: BytesN<32>) -> (Address, Option<u64>) { Resolver::resolve(e, c) }

    /// Registers a username commitment. See [registration::Registration::register].
    pub fn register(e: Env, c: Address, h: BytesN<32>) { Registration::register(e, c, h) }

    /// Gets the owner of a commitment. See [registration::Registration::get_owner].
    pub fn get_owner(e: Env, h: BytesN<32>) -> Option<Address> { Registration::get_owner(e, h) }

    /// Gets the registration ledger timestamp for a commitment. See [registration::Registration::get_created_at].
    pub fn get_created_at(e: Env, h: BytesN<32>) -> Option<u64> { Registration::get_created_at(e, h) }

    /// Adds a blockchain address for a commitment. See [address_manager::AddressManager::add_chain_address].
    pub fn add_chain_address(e: Env, c: Address, h: BytesN<32>, t: ChainType, a: Bytes) { AddressManager::add_chain_address(e, c, h, t, a); }

    /// Gets the blockchain address for a commitment. See [address_manager::AddressManager::get_chain_address].
    pub fn get_chain_address(e: Env, h: BytesN<32>, t: ChainType) -> Option<Bytes> { AddressManager::get_chain_address(e, h, t) }

    /// Removes a blockchain address for a commitment. See [address_manager::AddressManager::remove_chain_address].
    pub fn remove_chain_address(e: Env, c: Address, h: BytesN<32>, t: ChainType) { AddressManager::remove_chain_address(e, c, h, t); }

    /// Adds a Stellar address for a commitment. See [address_manager::AddressManager::add_stellar_address].
    pub fn add_stellar_address(e: Env, c: Address, h: BytesN<32>, a: Address) { AddressManager::add_stellar_address(e, c, h, a); }
    pub fn get_stellar_addresses(e: Env, h: BytesN<32>) -> soroban_sdk::Vec<Address> { AddressManager::get_stellar_addresses(e, h) }

    /// Resolves a commitment to its Stellar address. See [address_manager::AddressManager::resolve_stellar].
    pub fn resolve_stellar(e: Env, h: BytesN<32>) -> Address { AddressManager::resolve_stellar(e, h) }

    /// Transfers username ownership. See [transfer::Transfer::transfer_ownership].
    pub fn transfer_ownership(e: Env, c: Address, h: BytesN<32>, n: Address) { Transfer::transfer_ownership(e, c, h, n); }

    /// Transfers username ownership with ZK proof. See [transfer::Transfer::transfer].
    pub fn transfer(e: Env, c: Address, h: BytesN<32>, n: Address, p: Bytes, s: PublicSignals) { Transfer::transfer(e, c, h, n, p, s); }

    /// Adds a shielded address for a commitment. See [address_manager::AddressManager::add_shielded_address].
    pub fn add_shielded_address(e: Env, c: Address, h: BytesN<32>, a: BytesN<32>) { AddressManager::add_shielded_address(e, c, h, a); }

    /// Gets the shielded address for a commitment. See [address_manager::AddressManager::get_shielded_address].
    pub fn get_shielded_address(e: Env, h: BytesN<32>) -> Option<BytesN<32>> { AddressManager::get_shielded_address(e, h) }

    /// Checks if a commitment has a shielded address. See [address_manager::AddressManager::is_shielded].
    pub fn is_shielded(e: Env, h: BytesN<32>) -> bool { AddressManager::is_shielded(e, h) }
}
