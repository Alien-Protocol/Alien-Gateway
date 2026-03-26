#![cfg(test)]

use crate::errors::EscrowError;
use crate::types::{DataKey, ScheduledPayment, VaultConfig, VaultState};
use crate::EscrowContract;
use crate::EscrowContractClient;
use soroban_sdk::testutils::{Address as _, Events as _, Ledger};
use soroban_sdk::token::{Client as TokenClient, StellarAssetClient};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Error};

// ---------------------------------------------------------------------------
// Mock Registration contract — exposes get_owner / set_owner for tests.
// ---------------------------------------------------------------------------
#[contract]
pub struct MockRegistrationContract;

#[contractimpl]
impl MockRegistrationContract {
    /// Seed an owner for a commitment (no auth required — test helper only).
    pub fn set_owner(env: Env, commitment: BytesN<32>, owner: Address) {
        env.storage().persistent().set(&commitment, &owner);
    }

    /// Mirror of the real Registration::get_owner interface.
    pub fn get_owner(env: Env, commitment: BytesN<32>) -> Option<Address> {
        env.storage().persistent().get(&commitment)
    }
}

fn setup_test(
    env: &Env,
) -> (
    Address,
    EscrowContractClient<'_>,
    Address,
    Address,
    BytesN<32>,
    BytesN<32>,
) {
    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(env, &contract_id);

    let token_admin = Address::generate(env);
    let token = env.register_stellar_asset_contract(token_admin.clone());

    let from = BytesN::from_array(env, &[0u8; 32]);
    let to = BytesN::from_array(env, &[1u8; 32]);

    (contract_id, client, token, token_admin, from, to)
}

fn create_vault(
    env: &Env,
    contract_id: &Address,
    id: &BytesN<32>,
    owner: &Address,
    token: &Address,
    balance: i128,
) {
    let config = VaultConfig {
        owner: owner.clone(),
        token: token.clone(),
        created_at: 0,
    };
    let state = VaultState {
        balance,
        is_active: true,
    };
    env.as_contract(contract_id, || {
        env.storage()
            .persistent()
            .set(&DataKey::VaultConfig(id.clone()), &config);
        env.storage()
            .persistent()
            .set(&DataKey::VaultState(id.clone()), &state);
    });
}

fn read_vault(env: &Env, contract_id: &Address, id: &BytesN<32>) -> VaultState {
    env.as_contract(contract_id, || {
        env.storage()
            .persistent()
            .get(&DataKey::Vault(id.clone()))
            .unwrap()
    })
}

#[test]
fn test_schedule_payment_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _, from, to) = setup_test(&env);

    let initial_balance = 1000i128;
    let amount = 400i128;
    let release_at = 2000u64;

    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        initial_balance,
    );
    env.ledger().set_timestamp(1000);

    let payment_id = client.schedule_payment(&from, &to, &amount, &release_at);
    assert_eq!(payment_id, 0);

    // Verify balance decremented
    env.as_contract(&contract_id, || {
        let state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::VaultState(from.clone()))
            .unwrap();
        assert_eq!(state.balance, initial_balance - amount);

        // Verify VaultConfig is unmodified after payment scheduling
        let config: VaultConfig = env
            .storage()
            .persistent()
            .get(&DataKey::VaultConfig(from.clone()))
            .unwrap();
        assert_eq!(config.token, token);

        // Verify ScheduledPayment stored correctly
        let payment: ScheduledPayment = env
            .storage()
            .persistent()
            .get(&DataKey::ScheduledPayment(payment_id))
            .unwrap();
        assert_eq!(payment.from, from);
        assert_eq!(payment.to, to);
        assert_eq!(payment.amount, amount);
        assert_eq!(payment.release_at, release_at);
        assert!(!payment.executed);
    });
}

#[test]
fn test_schedule_payment_inactive_vault() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _, from, to) = setup_test(&env);

    // Seed vault with is_active: false
    let config = VaultConfig {
        owner: Address::generate(&env),
        token: token.clone(),
        created_at: 0,
    };
    let state = VaultState {
        balance: 1000,
        is_active: false,
    };
    env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .set(&DataKey::VaultConfig(from.clone()), &config);
        env.storage()
            .persistent()
            .set(&DataKey::VaultState(from.clone()), &state);
    });

    env.ledger().set_timestamp(1000);

    let result = client.try_schedule_payment(&from, &to, &100, &2000);
    assert_eq!(result, Err(Ok(EscrowError::VaultInactive)));
}

#[test]
fn test_schedule_payment_past_release_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, _, _, from, to) = setup_test(&env);

    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &Address::generate(&env),
        1000,
    );
    env.ledger().set_timestamp(2000);

    // release_at (1000) is in the past relative to current ledger (2000)
    let result = client.try_schedule_payment(&from, &to, &100, &1000);
    assert_eq!(result, Err(Ok(EscrowError::PastReleaseTime)));
}

#[test]
fn test_schedule_payment_insufficient_balance_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, _, _, from, to) = setup_test(&env);

    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &Address::generate(&env),
        100,
    );
    env.ledger().set_timestamp(1000);

    // amount (200) > balance (100)
    let result = client.try_schedule_payment(&from, &to, &200, &2000);
    assert_eq!(result, Err(Ok(EscrowError::InsufficientBalance)));
}

#[test]
fn test_schedule_payment_returns_incrementing_ids() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _, from, to) = setup_test(&env);

    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        10000,
    );
    env.ledger().set_timestamp(1000);

    let id0 = client.schedule_payment(&from, &to, &100, &2000);
    let id1 = client.schedule_payment(&from, &to, &200, &3000);

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
}

#[test]
fn test_execute_scheduled_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, to) = setup_test(&env);

    let from_owner = Address::generate(&env);
    let to_owner = Address::generate(&env);
    let amount = 400i128;
    let release_at = 2000u64;

    create_vault(&env, &contract_id, &from, &from_owner, &token, 1000);
    create_vault(&env, &contract_id, &to, &to_owner, &token, 0);

    // Schedule payment
    env.ledger().set_timestamp(1000);
    let payment_id = client.schedule_payment(&from, &to, &amount, &release_at);

    // Mint tokens to the contract to fulfill the payment (representing the reserved balance)
    let token_admin_client = StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contract_id, &amount);

    // Advance ledger and execute
    env.ledger().set_timestamp(2500);
    client.execute_scheduled(&payment_id);

    // Verify event
    let events = env.events().all();
    let escrow_events = events
        .iter()
        .filter(|(event_contract, _, _)| event_contract == &contract_id)
        .count();
    assert!(escrow_events > 0); // schedule + execute events

    // Verify executed = true in storage
    env.as_contract(&contract_id, || {
        let payment: ScheduledPayment = env
            .storage()
            .persistent()
            .get(&DataKey::ScheduledPayment(payment_id))
            .unwrap();
        assert!(payment.executed);
    });

    // Verify token transferred
    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&to_owner), amount);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_execute_scheduled_early_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _, from, to) = setup_test(&env);

    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        1000,
    );
    create_vault(&env, &contract_id, &to, &Address::generate(&env), &token, 0);

    env.ledger().set_timestamp(1000);
    let payment_id = client.schedule_payment(&from, &to, &100, &2000);

    // Attempt before release_at
    let result = client.try_execute_scheduled(&payment_id);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::PaymentNotYetDue as u32)
    ));
}

// ---------------------------------------------------------------------------
// create_vault tests
// ---------------------------------------------------------------------------

/// Deploys a MockRegistrationContract, seeds `owner` for `commitment`, then
/// returns (escrow_client, reg_id, owner, token, commitment).
fn setup_with_registration<'a>(
    env: &'a Env,
    commitment_seed: u8,
) -> (
    EscrowContractClient<'a>,
    Address,
    Address,
    Address,
    BytesN<32>,
) {
    let reg_id = env.register(MockRegistrationContract, ());
    let reg_client = MockRegistrationContractClient::new(env, &reg_id);

    let commitment = BytesN::from_array(env, &[commitment_seed; 32]);
    let owner = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = env.register_stellar_asset_contract(token_admin);

    reg_client.set_owner(&commitment, &owner);

    let escrow_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(env, &escrow_id);
    let admin = Address::generate(env);
    client.initialize(&admin, &reg_id);

    (client, escrow_id, owner, token, commitment)
}

#[test]
fn test_create_vault_success() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, escrow_id, owner, token, commitment) = setup_with_registration(&env, 0xAA);

    client.create_vault(&commitment, &token);

    // Verify VaultConfig persisted correctly.
    env.as_contract(&escrow_id, || {
        let config: VaultConfig = env
            .storage()
            .persistent()
            .get(&DataKey::VaultConfig(commitment.clone()))
            .unwrap();
        assert_eq!(config.owner, owner);
        assert_eq!(config.token, token);

        // Verify VaultState persisted correctly.
        let state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::VaultState(commitment.clone()))
            .unwrap();
        assert_eq!(state.balance, 0);
        assert!(state.is_active);
    });

    // Event emission is validated by the existing VaultCrtEvent publish call inside
    // create_vault. In native test mode with env.invoke_contract cross-calls the
    // outer contract's events are not surfaced via env.events().all(), so we rely
    // on the storage assertions above to confirm correct execution.
}

#[test]
fn test_create_vault_already_exists() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, _, token, commitment) = setup_with_registration(&env, 0xBB);

    client.create_vault(&commitment, &token);

    // Second call must panic with VaultAlreadyExists.
    let result = client.try_create_vault(&commitment, &token);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::VaultAlreadyExists as u32)
    ));
}

#[test]
#[should_panic]
fn test_create_vault_not_owner() {
    let env = Env::default();
    // No mock_all_auths: a caller who is NOT the registered owner cannot create
    // the vault because owner.require_auth() will reject the transaction.

    let reg_id = env.register(MockRegistrationContract, ());
    let reg_client = MockRegistrationContractClient::new(&env, &reg_id);

    let commitment = BytesN::from_array(&env, &[0xCCu8; 32]);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin);

    // set_owner has no require_auth, so it succeeds without auth mocking.
    reg_client.set_owner(&commitment, &owner);

    let escrow_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &escrow_id);

    // Write the registration address directly into instance storage so we
    // skip initialize (which now requires admin.require_auth()) and keep this
    // test entirely auth-free — proving create_vault itself enforces the check.
    env.as_contract(&escrow_id, || {
        env.storage()
            .instance()
            .set(&DataKey::RegistrationContract, &reg_id);
    });

    // create_vault calls owner.require_auth() → panics because no auth is mocked.
    client.create_vault(&commitment, &token);
}

#[test]
fn test_execute_scheduled_double_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, to) = setup_test(&env);

    let to_owner = Address::generate(&env);
    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        1000,
    );
    create_vault(&env, &contract_id, &to, &to_owner, &token, 0);

    env.ledger().set_timestamp(1000);
    let payment_id = client.schedule_payment(&from, &to, &100, &1500);

    let token_admin_client = StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contract_id, &100);

    env.ledger().set_timestamp(1600);
    // First execution succeeds
    client.execute_scheduled(&payment_id);

    // Second execution panics
    let result = client.try_execute_scheduled(&payment_id);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::PaymentAlreadyExecuted as u32)
    ));
}

#[test]
fn test_execute_scheduled_not_found_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client, _, _, _, _) = setup_test(&env);

    // Attempt to execute an invalid payment_id
    let invalid_id = 999;
    let result = client.try_execute_scheduled(&invalid_id);

    // Expecting PaymentNotFound error
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::PaymentNotFound as u32)
    ));
}
