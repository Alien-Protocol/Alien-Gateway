#![cfg(test)]

use crate::errors::EscrowError;
use crate::types::{DataKey, ScheduledPayment, VaultConfig, VaultState};
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

fn mint_token(env: &Env, token: &Address, token_admin: &Address, to: &Address, amount: i128) {
    let admin_client = StellarAssetClient::new(env, token);
    admin_client.mock_all_auths().mint(to, &amount);
    assert_eq!(admin_client.admin(), *token_admin);
}

fn _read_vault(env: &Env, contract_id: &Address, id: &BytesN<32>) -> VaultState {
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
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address()
        .clone();

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

    // Mint tokens to the owner so they can deposit
    let token_admin_client = StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&owner, &deposit_amount);

    client.deposit(&from, &deposit_amount);

    // Verify balance incremented
    env.as_contract(&contract_id, || {
        let state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::VaultState(from.clone()))
            .unwrap();
        assert_eq!(state.balance, initial_balance + deposit_amount);
    });

    // Verify token transferred to contract
    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&contract_id), deposit_amount);
    assert_eq!(token_client.balance(&owner), 0);

    // Note: Event emission is validated by the existing DepositEvent publish call inside
    // deposit. In native test mode with env.invoke_contract cross-calls, the
    // outer contract's events are not reliably surfaced via env.events().all(), so we rely
    // on the storage and balance assertions above to confirm correct execution.
}

#[test]
fn test_deposit_non_existent_vault() {
    let env = Env::default();
    env.mock_all_auths();
    let (_contract_id, client, _token, _token_admin, from, _to) = setup_test(&env);

    // No vault created for 'from'

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
    // Seed vault with is_active: false
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

    // Zero amount
    let result0 = client.try_deposit(&from, &0);
    assert!(matches!(
        result0,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::InvalidAmount as u32)
    ));

    // Negative amount
    let result_neg = client.try_deposit(&from, &-50);
    assert!(matches!(
        result_neg,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::InvalidAmount as u32)
    ));
}

#[test]
#[should_panic]
fn test_deposit_not_owner() {
    let env = Env::default();
    // No mock_all_auths for the actual call
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    let owner = Address::generate(&env);
    create_vault(&env, &contract_id, &from, &owner, &token, 100);

    // Call deposit without owner's auth
    // client.deposit(&from, &100) will panic because owner.require_auth() fails.
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

    // Balance should reflect the reserved funds
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
fn test_withdraw_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    let owner = Address::generate(&env);
    let initial_balance = 100i128;
    let withdraw_amount = 40i128;

    create_vault(&env, &contract_id, &from, &owner, &token, initial_balance);

    // Mint tokens to the contract to simulate prior deposits
    let token_admin_client = StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contract_id, &initial_balance);

    // Verify initial state
    env.as_contract(&contract_id, || {
        let state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::VaultState(from.clone()))
            .unwrap();
        assert_eq!(state.balance, initial_balance);
    });

    // Perform withdrawal
    client.withdraw(&from, &withdraw_amount);

    // Verify balance decremented
    env.as_contract(&contract_id, || {
        let state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::VaultState(from.clone()))
            .unwrap();
        assert_eq!(state.balance, initial_balance - withdraw_amount);
    });

    // Verify token transferred to owner
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

    // No vault created for 'from'

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
    // Seed vault with is_active: false
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

    // Zero amount
    let result0 = client.try_withdraw(&from, &0);
    assert!(matches!(
        result0,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::InvalidAmount as u32)
    ));

    // Negative amount
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

    // Try to withdraw more than balance
    let result = client.try_withdraw(&from, &100);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::InsufficientBalance as u32)
    ));

    // Verify balance unchanged
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
    // No mock_all_auths for the actual call
    let (contract_id, client, token, _token_admin, from, _to) = setup_test(&env);

    let owner = Address::generate(&env);
    create_vault(&env, &contract_id, &from, &owner, &token, 100);

    // Call withdraw without owner's auth
    // client.withdraw(&from, &50) will panic because owner.require_auth() fails.
    client.withdraw(&from, &50);
}

// ─── auto-pay storage isolation tests ────────────────────────────────────────

/// Registers one auto-pay rule on each of two different vaults and confirms
/// that neither rule is visible when looking up the other vault's commitment.
/// This validates that the composite key (commitment, rule_id) fully isolates
/// rules across vaults even when the global rule_id counter produces the same
/// numeric ID for each.
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

    // Both rules share rule_id = 0 (simulating the global counter starting at 0
    // for each vault). The composite key must keep them isolated.
    env.as_contract(&contract_id, || {
        write_auto_pay(&env, &vault_a, 0, &rule_a);
        write_auto_pay(&env, &vault_b, 0, &rule_b);
    });

    env.as_contract(&contract_id, || {
        // Vault A's rule is retrievable under vault A's commitment.
        let stored_a = read_auto_pay(&env, &vault_a, 0).expect("rule for vault_a not found");
        assert_eq!(stored_a.amount, 100);
        assert_eq!(stored_a.interval, 86_400);

        // Vault B's rule is retrievable under vault B's commitment.
        let stored_b = read_auto_pay(&env, &vault_b, 0).expect("rule for vault_b not found");
        assert_eq!(stored_b.amount, 200);
        assert_eq!(stored_b.interval, 43_200);

        // Vault A's commitment does NOT return vault B's rule, and vice versa.
        assert_ne!(stored_a.amount, stored_b.amount);
        assert_ne!(stored_a.from, stored_b.from);
    });
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

    // Mint tokens to contract so cancel_vault can transfer the refund
    let token_admin_client = StellarAssetClient::new(&env, &token);
    token_admin_client
        .mock_all_auths()
        .mint(&contract_id, &initial_balance);

    // Verify initial state
    assert_eq!(client.get_balance(&from), Some(initial_balance));

    // Cancel vault
    client.cancel_vault(&from);

    // Verify balance is now 0
    assert_eq!(client.get_balance(&from), Some(0));

    // Verify vault is inactive
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

    // Create vault with 0 balance
    create_vault(&env, &contract_id, &from, &owner, &token, 0);

    // Cancel vault should succeed without transfer
    client.cancel_vault(&from);

    // Verify vault is inactive
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

    // Cancel the vault
    client.cancel_vault(&from);

    // Attempt to deposit should fail with VaultInactive and panic
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

    // Cancel the vault
    client.cancel_vault(&from);

    env.ledger().set_timestamp(1000);

    // Attempt to schedule payment should fail with VaultInactive and panic
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

    // Mock auth for non-owner
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
