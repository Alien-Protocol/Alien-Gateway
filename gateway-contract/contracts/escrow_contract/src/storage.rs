use crate::errors::EscrowError;
use crate::types::{AutoPay, DataKey, LegacyVault, ScheduledPayment, VaultConfig, VaultState};
use soroban_sdk::{Address, BytesN, Env};

/// TTL constants for persistent storage entries.
/// Bump amount: ~30 days (at ~5s per ledger close).
pub(crate) const PERSISTENT_BUMP_AMOUNT: u32 = 518_400;
/// Lifetime threshold: ~7 days — entries are extended when remaining TTL drops below this.
pub(crate) const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;

/// Reads a vault's immutable configuration from persistent storage.
///
/// Checks the new `VaultConfig` key first; if absent, falls back to the legacy `Vault` key and
/// projects the combined record into a `VaultConfig` for backward compatibility.
pub fn read_vault_config(env: &Env, commitment: &BytesN<32>) -> Option<VaultConfig> {
    let storage = env.storage().persistent();
    if let Some(config) = storage.get(&DataKey::VaultConfig(commitment.clone())) {
        return Some(config);
    }
    let legacy: LegacyVault = storage.get(&DataKey::Vault(commitment.clone()))?;
    Some(VaultConfig {
        owner: legacy.owner,
        token: legacy.token,
        created_at: legacy.created_at,
    })
}

/// Writes a vault's immutable configuration to persistent storage.
pub fn write_vault_config(env: &Env, commitment: &BytesN<32>, config: &VaultConfig) {
    let key = DataKey::VaultConfig(commitment.clone());
    env.storage().persistent().set(&key, config);
    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

/// Reads a vault's mutable state from persistent storage.
///
/// Checks the new `VaultState` key first; if absent, falls back to the legacy `Vault` key and
/// projects the combined record into a `VaultState` for backward compatibility.
pub fn read_vault_state(env: &Env, commitment: &BytesN<32>) -> Option<VaultState> {
    let storage = env.storage().persistent();
    if let Some(state) = storage.get(&DataKey::VaultState(commitment.clone())) {
        return Some(state);
    }
    let legacy: LegacyVault = storage.get(&DataKey::Vault(commitment.clone()))?;
    Some(VaultState {
        balance: legacy.balance,
        is_active: legacy.is_active,
    })
}

/// Writes a vault's mutable state to persistent storage.
pub fn write_vault_state(env: &Env, commitment: &BytesN<32>, state: &VaultState) {
    let key = DataKey::VaultState(commitment.clone());
    env.storage().persistent().set(&key, state);
    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

/// Increments the global payment counter and returns the previous ID.
///
/// ### Errors
/// - Returns `EscrowError::PaymentCounterOverflow` if the counter reaches `u32::MAX`.
pub fn increment_payment_id(env: &Env) -> Result<u32, EscrowError> {
    let id: u32 = env
        .storage()
        .instance()
        .get(&DataKey::PaymentCounter)
        .unwrap_or(0);

    let next = id
        .checked_add(1)
        .ok_or(EscrowError::PaymentCounterOverflow)?;

    env.storage()
        .instance()
        .set(&DataKey::PaymentCounter, &next);

    Ok(id)
}

/// Reads the Registration contract address from instance storage.
pub fn read_registration_contract(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::RegistrationContract)
}

/// Writes the Registration contract address to instance storage.
pub fn write_registration_contract(env: &Env, address: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::RegistrationContract, address);
}

/// Records a new scheduled payment in persistent storage.
pub fn write_scheduled_payment(env: &Env, id: u32, payment: &ScheduledPayment) {
    let key = DataKey::ScheduledPayment(id);
    env.storage().persistent().set(&key, payment);
    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

/// Increments the global auto-pay counter and returns the previous ID.
///
/// ### Errors
/// - Returns `EscrowError::AutoPayCounterOverflow` if the counter reaches `u32::MAX`.
pub fn increment_auto_pay_id(env: &Env) -> Result<u32, EscrowError> {
    let id: u32 = env
        .storage()
        .instance()
        .get(&DataKey::AutoPayCounter)
        .unwrap_or(0);

    let next = id
        .checked_add(1)
        .ok_or(EscrowError::AutoPayCounterOverflow)?;

    env.storage()
        .instance()
        .set(&DataKey::AutoPayCounter, &next);

    Ok(id)
}

/// Reads the current auto-pay counter from instance storage.
///
/// Returns `0` when no auto-pay rules have been created yet.
pub fn read_auto_pay_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::AutoPayCounter)
        .unwrap_or(0)
}

/// Records an auto-pay rule in persistent storage under the composite key (vault, rule_id).
pub fn write_auto_pay(env: &Env, commitment: &BytesN<32>, rule_id: u32, auto_pay: &AutoPay) {
    let key = DataKey::AutoPay(commitment.clone(), rule_id as u64);
    env.storage().persistent().set(&key, auto_pay);
    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

/// Reads an auto-pay rule from persistent storage by vault commitment and rule ID.
pub fn read_auto_pay(env: &Env, commitment: &BytesN<32>, rule_id: u32) -> Option<AutoPay> {
    env.storage()
        .persistent()
        .get(&DataKey::AutoPay(commitment.clone(), rule_id as u64))
}

/// Deletes an auto-pay rule from persistent storage by vault commitment and rule ID.
pub fn delete_auto_pay(env: &Env, from: &BytesN<32>, rule_id: u32) {
    let key = DataKey::AutoPay(from.clone(), rule_id as u64);
    env.storage().persistent().remove(&key);
}
