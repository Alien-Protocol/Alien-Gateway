#![cfg(test)]

use crate::errors::EscrowError;
use crate::types::{DataKey, ScheduledPayment, VaultState};
use crate::EscrowContract;
use crate::EscrowContractClient;
use soroban_sdk::testutils::{Address as _, Events as _, Ledger};
use soroban_sdk::token::{Client as TokenClient, StellarAssetClient};
use soroban_sdk::{Address, BytesN, Env, Error};

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
    let vault = VaultState {
        owner: owner.clone(),
        token: token.clone(),
        balance,
    };
    env.as_contract(contract_id, || {
        env.storage()
            .persistent()
            .set(&DataKey::Vault(id.clone()), &vault);
    });
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
        let vault: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::Vault(from.clone()))
            .unwrap();
        assert_eq!(vault.balance, initial_balance - amount);

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
        assert_eq!(payment.executed, false);
    });
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
fn test_execute_scheduled_success_transfers_and_marks_executed() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, _token_admin, from, to) = setup_test(&env);

    let from_owner = Address::generate(&env);
    let to_owner = Address::generate(&env);
    let amount = 400i128;
    let release_at = 2000u64;

    create_vault(&env, &contract_id, &from, &from_owner, &token, 1000);
    create_vault(&env, &contract_id, &to, &to_owner, &token, 0);

    env.ledger().set_timestamp(1000);
    let payment_id = client.schedule_payment(&from, &to, &amount, &release_at);

    let token_admin_client = StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contract_id, &amount);

    env.ledger().set_timestamp(2500);
    client.execute_scheduled(&payment_id);

    let events = env.events().all();
    let escrow_events = events
        .iter()
        .filter(|(event_contract, _, _)| event_contract == &contract_id)
        .count();
    assert_eq!(escrow_events, 1);

    env.as_contract(&contract_id, || {
        let payment: ScheduledPayment = env
            .storage()
            .persistent()
            .get(&DataKey::ScheduledPayment(payment_id))
            .unwrap();
        assert_eq!(payment.executed, true);
    });

    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&to_owner), amount);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_execute_scheduled_rejects_early() {
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

    let result = client.try_execute_scheduled(&payment_id);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::PaymentNotYetDue as u32)
    ));
}

#[test]
fn test_execute_scheduled_rejects_double_execution() {
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
    client.execute_scheduled(&payment_id);

    let result = client.try_execute_scheduled(&payment_id);
    assert!(matches!(
        result,
        Err(Ok(err)) if err == Error::from_contract_error(EscrowError::PaymentAlreadyExecuted as u32)
    ));
}
