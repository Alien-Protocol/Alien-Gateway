use crate::errors::EscrowError;
use crate::types::{AutoPay, DataKey, LegacyVault, ScheduledPayment, VaultConfig, VaultState};
use crate::EscrowContract;
use crate::EscrowContractClient;
use soroban_sdk::testutils::{Address as _, Events as _, Ledger, MockAuth, MockAuthInvoke};
use soroban_sdk::token::{Client as TokenClient, StellarAssetClient};

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Error, IntoVal};

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
    let token = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address()
        .clone();

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

fn create_legacy_vault(
    env: &Env,
    contract_id: &Address,
    id: &BytesN<32>,
    owner: &Address,
    token: &Address,
    balance: i128,
) {
    let legacy = LegacyVault {
        owner: owner.clone(),
        token: token.clone(),
        created_at: 0,
        balance,
        is_active: true,
    };
    env.as_contract(contract_id, || {
        env.storage()
            .persistent()
            .set(&DataKey::Vault(id.clone()), &legacy);
    });
}

fn mint_token(env: &Env, token: &Address, token_admin: &Address, to: &Address, amount: i128) {
    let admin_client = StellarAssetClient::new(env, token);
    admin_client.mock_all_auths().mint(to, &amount);
    assert_eq!(admin_client.admin(), *token_admin);
}

#[test]
fn test_legacy_vault_key_fallback_and_migration() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, token_admin, from, _to) = setup_test(&env);

    let owner = Address::generate(&env);
    create_legacy_vault(&env, &contract_id, &from, &owner, &token, 1000);

    mint_token(&env, &token, &token_admin, &owner, 500);

    // legacy storage should be readable through current getters.
    assert_eq!(client.get_balance(&from), Some(1000));

    // carry out a transition mutation (deposit) to verify the mutable split key is written.
    client.deposit(&from, &200);
    assert_eq!(client.get_balance(&from), Some(1200));

    env.as_contract(&contract_id, || {
        let config: Option<VaultConfig> = env
            .storage()
            .persistent()
            .get(&DataKey::VaultConfig(from.clone()));
        assert_eq!(config, None);

        let state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::VaultState(from.clone()))
            .unwrap();
        assert_eq!(state.balance, 1200);
        assert!(state.is_active);

        let legacy: LegacyVault = env
            .storage()
            .persistent()
            .get(&DataKey::Vault(from.clone()))
            .unwrap();
        assert_eq!(legacy.owner, owner);
        assert_eq!(legacy.token, token);
    });
}

// ─── get_scheduled_payment tests ─────────────────────────────────────────────

/// Verifies that `get_scheduled_payment` returns `Some(ScheduledPayment)` with
/// all correct fields immediately after `schedule_payment` has been called.
#[test]
fn test_get_scheduled_payment_returns_all_fields_after_schedule() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _, from, to) = setup_test(&env);

    let initial_balance = 1_000i128;
    let amount = 400i128;
    let release_at = 2_000u64;

    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        initial_balance,
    );
    env.ledger().set_timestamp(1_000);

    let payment_id = client.schedule_payment(&from, &to, &amount, &release_at);

    let result = client.get_scheduled_payment(&payment_id);
    assert!(
        result.is_some(),
        "expected Some(ScheduledPayment) for a known payment_id"
    );

    let payment = result.unwrap();
    assert_eq!(payment.from, from);
    assert_eq!(payment.to, to);
    assert_eq!(payment.amount, amount);
    assert_eq!(payment.release_at, release_at);
    assert!(!payment.executed, "payment should not be executed yet");
}

/// Verifies that `get_scheduled_payment` returns `None` for an ID that was
/// never created, confirming it does not fabricate data.
#[test]
fn test_get_scheduled_payment_returns_none_for_unknown_id() {
    let env = Env::default();
    let (_, client, _, _, _, _) = setup_test(&env);

    let result = client.get_scheduled_payment(&99_999u32);
    assert!(
        result.is_none(),
        "expected None for an unknown payment_id"
    );
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
fn test_payment_counter_overflow_returns_error() {
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
    env.ledger().set_timestamp(1000);

    env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .set(&DataKey::PaymentCounter, &u32::MAX);
    });

    let result = client.try_schedule_payment(&from, &to, &100, &2000);
    assert_eq!(result, Err(Ok(EscrowError::PaymentCounterOverflow)));
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
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address()
        .clone();

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

    let reg_id = env.register(MockRegistrationContract, ());
    let reg_client = MockRegistrationContractClient::new(&env, &reg_id);

    let commitment = BytesN::from_array(&env, &[0xCCu8; 32]);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address()
        .clone();

    reg_client.set_owner(&commitment, &owner);

    let escrow_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &escrow_id);

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

    let invalid_id = 999;
    let result = client.try_execute_scheduled(&invalid_id);

    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::PaymentNotFound as u32)
    ));
}

// ---------------------------------------------------------------------------
// deposit tests
// ---------------------------------------------------------------------------

#[test]
fn test_deposit_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    let owner = Address::generate(&env);
    let initial_balance = 100i128;
    let deposit_amount = 50i128;

    create_vault(&env, &contract_id, &from, &owner, &token, initial_balance);

    let token_admin_client = StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&owner, &deposit_amount);

    client.deposit(&from, &deposit_amount);

    env.as_contract(&contract_id, || {
        let state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::VaultState(from.clone()))
            .unwrap();
        assert_eq!(state.balance, initial_balance + deposit_amount);
    });

    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&contract_id), deposit_amount);
    assert_eq!(token_client.balance(&owner), 0);
}

#[test]
fn test_deposit_non_existent_vault() {
    let env = Env::default();
    env.mock_all_auths();
    let (_contract_id, client, _token, _token_admin, from, _to) = setup_test(&env);

    let result = client.try_deposit(&from, &100);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::VaultNotFound as u32)
    ));
}

#[test]
fn test_deposit_inactive_vault() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    let owner = Address::generate(&env);
    let config = VaultConfig {
        owner: owner.clone(),
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

    let result = client.try_deposit(&from, &100);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::VaultInactive as u32)
    ));
}

#[test]
fn test_deposit_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    let owner = Address::generate(&env);
    create_vault(&env, &contract_id, &from, &owner, &token, 100);

    let result0 = client.try_deposit(&from, &0);
    assert!(matches!(
        result0,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::InvalidAmount as u32)
    ));

    let result_neg = client.try_deposit(&from, &-50);
    assert!(matches!(
        result_neg,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::InvalidAmount as u32)
    ));
}

#[test]
fn test_withdraw_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, _) = setup_test(&env);

    let owner = Address::generate(&env);
    let initial_balance = 1_000i128;
    let withdraw_amount = 400i128;

    create_vault(&env, &contract_id, &from, &owner, &token, initial_balance);

    let token_admin_client = StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contract_id, &withdraw_amount);

    client.withdraw(&from, &withdraw_amount);

    env.as_contract(&contract_id, || {
        let state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::VaultState(from.clone()))
            .unwrap();
        assert_eq!(state.balance, initial_balance - withdraw_amount);
        assert!(state.is_active);
    });

    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&owner), withdraw_amount);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_withdraw_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, _) = setup_test(&env);

    let owner = Address::generate(&env);
    create_vault(&env, &contract_id, &from, &owner, &token, 100);

    let result = client.try_withdraw(&from, &200);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::InsufficientBalance as u32)
    ));
}

#[test]
#[should_panic]
fn test_deposit_not_owner() {
    let env = Env::default();
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    let owner = Address::generate(&env);
    create_vault(&env, &contract_id, &from, &owner, &token, 100);

    client.deposit(&from, &100);
}

// ─── get_balance tests ───────────────────────────────────────────────

#[test]
fn test_get_balance_vault_not_found() {
    let env = Env::default();
    let (_, client, _, _, _, _) = setup_test(&env);

    let unknown = BytesN::from_array(&env, &[99u8; 32]);
    assert_eq!(client.get_balance(&unknown), None);
}

#[test]
fn test_get_balance_after_deposit() {
    let env = Env::default();
    let (contract_id, client, token, _, from, _) = setup_test(&env);

    let balance = 5_000i128;
    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        balance,
    );

    assert_eq!(client.get_balance(&from), Some(balance));
}

#[test]
fn test_get_balance_after_payment() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _, from, to) = setup_test(&env);

    let initial = 1_000i128;
    let amount = 300i128;

    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        initial,
    );
    create_vault(&env, &contract_id, &to, &Address::generate(&env), &token, 0);

    env.ledger().set_timestamp(1000);
    client.schedule_payment(&from, &to, &amount, &2000);

    assert_eq!(client.get_balance(&from), Some(initial - amount));
}

#[test]
fn test_deposit_increases_balance() {
    let env = Env::default();
    let (contract_id, client, token, token_admin, from, _) = setup_test(&env);
    let owner = Address::generate(&env);
    let amount = 100_i128;

    create_vault(&env, &contract_id, &from, &owner, &token, 0);
    mint_token(&env, &token, &token_admin, &owner, amount);

    client.mock_all_auths().deposit(&from, &amount);

    assert_eq!(client.get_balance(&from), Some(amount));
    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&owner), 0);
    assert_eq!(token_client.balance(&contract_id), amount);
}

#[test]
#[should_panic]
fn test_deposit_zero_panics() {
    let env = Env::default();
    let (contract_id, client, token, _, from, _) = setup_test(&env);
    let owner = Address::generate(&env);

    create_vault(&env, &contract_id, &from, &owner, &token, 0);
    client.mock_all_auths().deposit(&from, &0);
}

#[test]
#[should_panic]
fn test_deposit_non_owner_panics() {
    let env = Env::default();
    let (contract_id, client, token, token_admin, from, _) = setup_test(&env);
    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let amount = 100_i128;

    create_vault(&env, &contract_id, &from, &owner, &token, 0);
    mint_token(&env, &token, &token_admin, &owner, amount);

    client
        .mock_auths(&[MockAuth {
            address: &non_owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "deposit",
                args: (from.clone(), amount).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .deposit(&from, &amount);
}

#[test]
#[should_panic]
fn test_deposit_vault_not_found_panics() {
    let env = Env::default();
    let (_, client, _, _, _, _) = setup_test(&env);
    let commitment = BytesN::from_array(&env, &[9u8; 32]);

    client.mock_all_auths().deposit(&commitment, &100);
}

// ─── withdraw tests ──────────────────────────────────────────────────────

#[test]
fn test_withdraw_success_with_token_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    let owner = Address::generate(&env);
    let initial_balance = 100i128;
    let withdraw_amount = 40i128;

    create_vault(&env, &contract_id, &from, &owner, &token, initial_balance);

    let token_admin_client = StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contract_id, &initial_balance);

    env.as_contract(&contract_id, || {
        let state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::VaultState(from.clone()))
            .unwrap();
        assert_eq!(state.balance, initial_balance);
    });

    client.withdraw(&from, &withdraw_amount);

    env.as_contract(&contract_id, || {
        let state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::VaultState(from.clone()))
            .unwrap();
        assert_eq!(state.balance, initial_balance - withdraw_amount);
    });

    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&owner), withdraw_amount);
    assert_eq!(
        token_client.balance(&contract_id),
        initial_balance - withdraw_amount
    );
}

#[test]
fn test_withdraw_non_existent_vault() {
    let env = Env::default();
    env.mock_all_auths();
    let (_contract_id, client, _token, _token_admin, from, _to) = setup_test(&env);

    let result = client.try_withdraw(&from, &100);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::VaultNotFound as u32)
    ));
}

#[test]
fn test_withdraw_inactive_vault() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    let owner = Address::generate(&env);
    let config = VaultConfig {
        owner: owner.clone(),
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

    let result = client.try_withdraw(&from, &100);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::VaultInactive as u32)
    ));
}

#[test]
fn test_withdraw_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    let owner = Address::generate(&env);
    create_vault(&env, &contract_id, &from, &owner, &token, 100);

    let result0 = client.try_withdraw(&from, &0);
    assert!(matches!(
        result0,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::InvalidAmount as u32)
    ));

    let result_neg = client.try_withdraw(&from, &-50);
    assert!(matches!(
        result_neg,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::InvalidAmount as u32)
    ));
}

#[test]
fn test_withdraw_overdraft() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    let owner = Address::generate(&env);
    let balance = 50i128;
    create_vault(&env, &contract_id, &from, &owner, &token, balance);

    let result = client.try_withdraw(&from, &100);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::InsufficientBalance as u32)
    ));

    env.as_contract(&contract_id, || {
        let state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::VaultState(from.clone()))
            .unwrap();
        assert_eq!(state.balance, balance);
    });
}

#[test]
#[should_panic]
fn test_withdraw_not_owner() {
    let env = Env::default();
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    let owner = Address::generate(&env);
    create_vault(&env, &contract_id, &from, &owner, &token, 100);

    client.withdraw(&from, &50);
}

// ─── auto-pay storage isolation tests ────────────────────────────────────────

#[test]
fn test_auto_pay_multiple_vaults_no_interference() {
    use crate::storage::{read_auto_pay, write_auto_pay};
    use crate::types::AutoPay;

    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, _client, token, _token_admin, _from, _to) = setup_test(&env);

    let vault_a = BytesN::from_array(&env, &[0xAAu8; 32]);
    let vault_b = BytesN::from_array(&env, &[0xBBu8; 32]);

    let rule_a = AutoPay {
        from: vault_a.clone(),
        to: vault_b.clone(),
        token: token.clone(),
        amount: 100,
        interval: 86_400,
        last_paid: 0,
    };
    let rule_b = AutoPay {
        from: vault_b.clone(),
        to: vault_a.clone(),
        token: token.clone(),
        amount: 200,
        interval: 43_200,
        last_paid: 0,
    };

    env.as_contract(&contract_id, || {
        write_auto_pay(&env, &vault_a, 0, &rule_a);
        write_auto_pay(&env, &vault_b, 0, &rule_b);
    });

    env.as_contract(&contract_id, || {
        let stored_a = read_auto_pay(&env, &vault_a, 0).expect("rule for vault_a not found");
        assert_eq!(stored_a.amount, 100);
        assert_eq!(stored_a.interval, 86_400);

        let stored_b = read_auto_pay(&env, &vault_b, 0).expect("rule for vault_b not found");
        assert_eq!(stored_b.amount, 200);
        assert_eq!(stored_b.interval, 43_200);

        assert_ne!(stored_a.amount, stored_b.amount);
        assert_ne!(stored_a.from, stored_b.from);
    });
}

#[test]
fn test_trigger_auto_pay_inactive_vault_returns_vault_inactive() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, to) = setup_test(&env);

    let owner = Address::generate(&env);
    let config = VaultConfig {
        owner: owner.clone(),
        token: token.clone(),
        created_at: 0,
    };
    let state = VaultState {
        balance: 1000,
        is_active: false,
    };
    let auto_pay = AutoPay {
        from: from.clone(),
        to: to.clone(),
        token: token.clone(),
        amount: 100,
        interval: 1,
        last_paid: 0,
    };

    env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .set(&DataKey::VaultConfig(from.clone()), &config);
        env.storage()
            .persistent()
            .set(&DataKey::VaultState(from.clone()), &state);
        env.storage()
            .persistent()
            .set(&DataKey::AutoPay(from.clone(), 0u64), &auto_pay);
    });

    env.ledger().set_timestamp(1000);

    let result = client.try_trigger_auto_pay(&from, &0);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::VaultInactive as u32)
    ));
}

// ─── cancel_vault tests ──────────────────────────────────────────────

#[test]
fn test_cancel_vault_refunds_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, _) = setup_test(&env);
    let owner = Address::generate(&env);

    let initial_balance = 100i128;
    create_vault(&env, &contract_id, &from, &owner, &token, initial_balance);

    let token_admin_client = StellarAssetClient::new(&env, &token);
    token_admin_client
        .mock_all_auths()
        .mint(&contract_id, &initial_balance);

    assert_eq!(client.get_balance(&from), Some(initial_balance));

    client.cancel_vault(&from);

    assert_eq!(client.get_balance(&from), Some(0));

    env.as_contract(&contract_id, || {
        let state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::VaultState(from.clone()))
            .unwrap();
        assert!(!state.is_active);
        assert_eq!(state.balance, 0);
    });
}

#[test]
fn test_cancel_vault_empty_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _, from, _) = setup_test(&env);
    let owner = Address::generate(&env);

    create_vault(&env, &contract_id, &from, &owner, &token, 0);

    client.cancel_vault(&from);

    env.as_contract(&contract_id, || {
        let state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::VaultState(from.clone()))
            .unwrap();
        assert!(!state.is_active);
        assert_eq!(state.balance, 0);
    });
}

#[test]
#[should_panic]
fn test_cancel_vault_blocks_deposit() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _, from, _) = setup_test(&env);
    let owner = Address::generate(&env);

    create_vault(&env, &contract_id, &from, &owner, &token, 100);

    client.cancel_vault(&from);

    let amount = 50i128;
    client.deposit(&from, &amount);
}

#[test]
#[should_panic]
fn test_cancel_vault_blocks_schedule() {
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

    client.cancel_vault(&from);

    env.ledger().set_timestamp(1000);

    client.schedule_payment(&from, &to, &100, &2000);
}

#[test]
#[should_panic]
fn test_cancel_vault_non_owner_panics() {
    let env = Env::default();
    let (contract_id, client, token, _, from, _) = setup_test(&env);
    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);

    create_vault(&env, &contract_id, &from, &owner, &token, 100);

    client
        .mock_auths(&[MockAuth {
            address: &non_owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "cancel_vault",
                args: (from.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .cancel_vault(&from);
}

// ─── get_auto_pay_count tests ─────────────────────────────────────────────────

#[test]
fn test_get_auto_pay_count_returns_zero_before_any_rules() {
    let env = Env::default();
    let (_, client, _, _, _, _) = setup_test(&env);

    assert_eq!(client.get_auto_pay_count(), 0);
}

#[test]
fn test_get_auto_pay_count_increments_after_setup() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _, from, to) = setup_test(&env);

    create_vault(&env, &contract_id, &from, &Address::generate(&env), &token, 1000);

    assert_eq!(client.get_auto_pay_count(), 0);

    client.setup_auto_pay(&from, &to, &100, &86_400);

    assert_eq!(client.get_auto_pay_count(), 1);
}

#[test]
fn test_get_auto_pay_count_increments_with_multiple_rules() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _, from, to) = setup_test(&env);

    create_vault(&env, &contract_id, &from, &Address::generate(&env), &token, 1000);

    client.setup_auto_pay(&from, &to, &100, &86_400);
    assert_eq!(client.get_auto_pay_count(), 1);

    client.setup_auto_pay(&from, &to, &200, &43_200);
    assert_eq!(client.get_auto_pay_count(), 2);

    client.setup_auto_pay(&from, &to, &50, &3_600);
    assert_eq!(client.get_auto_pay_count(), 3);
}

// ─── initialize tests ──────────────────────────────────────────────

#[test]
fn test_initialize_twice_returns_already_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let reg_id = env.register(MockRegistrationContract, ());
    let escrow_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &escrow_id);
    let admin = Address::generate(&env);

    client.initialize(&admin, &reg_id);

    let result = client.try_initialize(&admin, &reg_id);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::AlreadyInitialized as u32)
    ));
}

// ─── get_auto_pay tests ──────────────────────────────────────────────

#[test]
fn test_get_auto_pay_returns_rule_after_setup() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, to) = setup_test(&env);

    let amount = 250i128;
    let interval = 86_400u64;

    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        1_000,
    );

    let rule_id = client.setup_auto_pay(&from, &to, &amount, &interval);

    let result = client.get_auto_pay(&from, &rule_id);
    assert!(
        result.is_some(),
        "expected Some(AutoPay) after setup_auto_pay"
    );

    let rule = result.unwrap();
    assert_eq!(rule.from, from);
    assert_eq!(rule.to, to);
    assert_eq!(rule.amount, amount);
    assert_eq!(rule.interval, interval);
    assert_eq!(rule.last_paid, 0);
}

#[test]
fn test_get_auto_pay_returns_none_for_unknown_rule() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        1_000,
    );

    let result = client.get_auto_pay(&from, &999u32);
    assert!(
        result.is_none(),
        "expected None for an unregistered rule_id"
    );
}

// ─── auto-pay self-payment test ──────────────────────────────────────────────

#[test]
fn test_auto_pay_self_payment_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    let owner = Address::generate(&env);
    create_vault(&env, &contract_id, &from, &owner, &token, 1000);

    // Attempt to setup auto-pay with from == to (self-payment)
    let result = client.try_setup_auto_pay(&from, &from, &100, &86400);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::SelfPaymentNotAllowed as u32)
    ));
}

/// Happy path: setup an auto-pay rule, cancel it, then confirm the rule is gone.
///
/// `get_auto_pay` must return `None` after a successful cancellation, proving
/// the persistent storage entry was actually deleted rather than just marked.
#[test]
fn test_cancel_auto_pay_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, to) = setup_test(&env);

    // Create a funded vault so setup_auto_pay can verify it exists.
    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        1_000,
    );

    // Register an auto-pay rule.
    let rule_id = client.setup_auto_pay(&from, &to, &100, &86_400);

    // Confirm the rule is present before cancellation.
    assert!(
        client.get_auto_pay(&from, &rule_id).is_some(),
        "rule must exist before cancel_auto_pay"
    );

    // Cancel the rule.
    client.cancel_auto_pay(&from, &rule_id);

    // After cancellation get_auto_pay must return None — the record is deleted.
    assert!(
        client.get_auto_pay(&from, &rule_id).is_none(),
        "get_auto_pay must return None after cancel_auto_pay"
    );
}

/// Core acceptance criterion: calling `trigger_auto_pay` after `cancel_auto_pay`
/// must panic (return an error) with `AutoPayNotFound`.
///
/// This is the primary invariant for the cancel feature: a cancelled rule is
/// indistinguishable from a rule that was never created.
#[test]
fn test_cancel_auto_pay_then_trigger_panics_with_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, token_admin, from, to) = setup_test(&env);

    let owner = Address::generate(&env);
    create_vault(&env, &contract_id, &from, &owner, &token, 1_000);

    // Mint tokens to the contract so trigger_auto_pay has funds to transfer.
    let token_admin_client = StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contract_id, &500);

    // Register, then immediately cancel.
    let rule_id = client.setup_auto_pay(&from, &to, &100, &1);
    client.cancel_auto_pay(&from, &rule_id);

    // Advance ledger so the interval check would pass if the rule still existed.
    env.ledger().set_timestamp(10_000);

    // trigger_auto_pay must fail with AutoPayNotFound — NOT IntervalNotElapsed
    // or InsufficientBalance.  The rule lookup must be the first thing that fails.
    let result = client.try_trigger_auto_pay(&from, &rule_id);
    assert!(
        matches!(
            result,
            Err(Ok(err)) if err == Error::from_contract_error(EscrowError::AutoPayNotFound as u32)
        ),
        "expected AutoPayNotFound after cancel, got: {:?}",
        result
    );
}

/// Security: a non-owner must not be able to cancel another user's auto-pay rule.
///
/// `cancel_auto_pay` calls `config.owner.require_auth()` which will panic when
/// the presented auth is for a different address.
#[test]
#[should_panic]
fn test_cancel_auto_pay_non_owner_panics() {
    let env = Env::default();
    let (contract_id, client, token, _token_admin, from, to) = setup_test(&env);

    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);

    // Set up vault with a known owner.
    create_vault(&env, &contract_id, &from, &owner, &token, 1_000);

    // Register rule as the real owner (with full auth for setup only).
    let rule_id = client.mock_all_auths().setup_auto_pay(&from, &to, &100, &86_400);

    // Attempt to cancel as non_owner — must panic because owner.require_auth() fails.
    client
        .mock_auths(&[MockAuth {
            address: &non_owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "cancel_auto_pay",
                args: (from.clone(), rule_id).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .cancel_auto_pay(&from, &rule_id);
}

/// Edge case: attempting to cancel a rule that does not exist must return
/// `AutoPayNotFound`.  This covers two sub-cases:
///   1. The rule was never registered (bad rule_id).
///   2. The rule was already cancelled (double-cancel).
#[test]
fn test_cancel_auto_pay_nonexistent_rule_returns_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        1_000,
    );

    // rule_id 999 was never created.
    let result = client.try_cancel_auto_pay(&from, &999u32);
    assert!(
        matches!(
            result,
            Err(Ok(err)) if err == Error::from_contract_error(EscrowError::AutoPayNotFound as u32)
        ),
        "expected AutoPayNotFound for a rule that was never registered, got: {:?}",
        result
    );
}

/// Edge case: double-cancel — cancelling a rule twice must return `AutoPayNotFound`
/// on the second attempt rather than silently succeeding.
#[test]
fn test_cancel_auto_pay_double_cancel_returns_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, to) = setup_test(&env);

    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        1_000,
    );

    let rule_id = client.setup_auto_pay(&from, &to, &100, &86_400);

    // First cancel — must succeed.
    client.cancel_auto_pay(&from, &rule_id);

    // Second cancel — must fail because the rule no longer exists.
    let result = client.try_cancel_auto_pay(&from, &rule_id);
    assert!(
        matches!(
            result,
            Err(Ok(err)) if err == Error::from_contract_error(EscrowError::AutoPayNotFound as u32)
        ),
        "expected AutoPayNotFound on double-cancel, got: {:?}",
        result
    );
}

/// Verify that cancelling one rule does not affect sibling rules on the same vault.
///
/// This guards against a storage bug where a cancel could accidentally delete the
/// wrong record due to incorrect key construction.
#[test]
fn test_cancel_auto_pay_does_not_affect_sibling_rules() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, to) = setup_test(&env);

    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        2_000,
    );

    // Register two rules on the same vault.
    let rule_a = client.setup_auto_pay(&from, &to, &100, &86_400);
    let rule_b = client.setup_auto_pay(&from, &to, &200, &43_200);

    // Cancel only rule_a.
    client.cancel_auto_pay(&from, &rule_a);

    // rule_a must be gone.
    assert!(
        client.get_auto_pay(&from, &rule_a).is_none(),
        "cancelled rule_a must return None"
    );

    // rule_b must be completely unaffected.
    let surviving = client.get_auto_pay(&from, &rule_b);
    assert!(
        surviving.is_some(),
        "rule_b must survive cancellation of rule_a"
    );
    assert_eq!(
        surviving.unwrap().amount,
        200,
        "rule_b amount must be unchanged after rule_a cancel"
    );
}

/// Verify that cancelling a rule on vault A does not affect a same-numbered rule
/// on vault B.  This is a cross-vault variant of the sibling isolation test and
/// directly validates that the composite key (from, rule_id) is correctly scoped.
#[test]
fn test_cancel_auto_pay_cross_vault_isolation() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, to) = setup_test(&env);

    let vault_a = from.clone();
    let vault_b = to.clone();

    // Both vaults need config + state so setup_auto_pay succeeds for each.
    create_vault(
        &env,
        &contract_id,
        &vault_a,
        &Address::generate(&env),
        &token,
        1_000,
    );
    create_vault(
        &env,
        &contract_id,
        &vault_b,
        &Address::generate(&env),
        &token,
        1_000,
    );

    // Each vault gets rule_id = 0 (first rule).
    let rule_a = client.setup_auto_pay(&vault_a, &vault_b, &100, &86_400);
    let rule_b = client.setup_auto_pay(&vault_b, &vault_a, &200, &43_200);

    assert_eq!(rule_a, 0, "first rule on vault_a must have id 0");
    // rule_b may be 1 depending on the global counter; what matters is that
    // vault_b has its own rule regardless of the numeric ID.

    // Cancel vault_a's rule.
    client.cancel_auto_pay(&vault_a, &rule_a);

    // vault_a rule gone.
    assert!(
        client.get_auto_pay(&vault_a, &rule_a).is_none(),
        "vault_a rule must be deleted"
    );

    // vault_b rule untouched — use the returned rule_b id.
    assert!(
        client.get_auto_pay(&vault_b, &rule_b).is_some(),
        "vault_b rule must survive cancellation on vault_a"
    );
}

/// Verify that `cancel_auto_pay` fails with `VaultNotFound` when called for a
/// commitment that has no vault at all.  This guards the auth path — without a
/// vault there is no config, so we cannot resolve an owner.
#[test]
fn test_cancel_auto_pay_vault_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client, _, _, _, _) = setup_test(&env);

    let nonexistent_commitment = BytesN::from_array(&env, &[0xFFu8; 32]);

    let result = client.try_cancel_auto_pay(&nonexistent_commitment, &0u32);
    assert!(
        matches!(
            result,
            Err(Ok(err)) if err == Error::from_contract_error(EscrowError::VaultNotFound as u32)
        ),
        "expected VaultNotFound for nonexistent vault, got: {:?}",
        result
    );
}