use crate::errors::EscrowError;
use crate::types::{DataKey, ScheduledPayment, VaultConfig, VaultState};
use soroban_sdk::{BytesN, Env};

/// Reads a vault's immutable configuration from persistent storage.
pub fn read_vault_config(env: &Env, commitment: &BytesN<32>) -> Option<VaultConfig> {
    env.storage()
        .persistent()
        .get(&DataKey::VaultConfig(commitment.clone()))
}

/// Writes a vault's immutable configuration to persistent storage.
pub fn write_vault_config(env: &Env, commitment: &BytesN<32>, config: &VaultConfig) {
    env.storage()
        .persistent()
        .set(&DataKey::VaultConfig(commitment.clone()), config);
}

/// Reads a vault's mutable state from persistent storage.
pub fn read_vault_state(env: &Env, commitment: &BytesN<32>) -> Option<VaultState> {
    env.storage()
        .persistent()
        .get(&DataKey::VaultState(commitment.clone()))
}

/// Writes a vault's mutable state to persistent storage.
pub fn write_vault_state(env: &Env, commitment: &BytesN<32>, state: &VaultState) {
    env.storage()
        .persistent()
        .set(&DataKey::VaultState(commitment.clone()), state);
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

/// Records a new scheduled payment in persistent storage.
pub fn write_scheduled_payment(env: &Env, id: u32, payment: &ScheduledPayment) {
    env.storage()
        .persistent()
        .set(&DataKey::ScheduledPayment(id), payment);
}
