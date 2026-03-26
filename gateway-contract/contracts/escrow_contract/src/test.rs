#![cfg(test)]

use crate::errors::EscrowError;
use crate::types::{DataKey, ScheduledPayment, VaultState};
use crate::EscrowContract;
use crate::EscrowContractClient;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Address, BytesN, Env};

fn setup_test(
    env: &Env,
) -> (
    Address,
    EscrowContractClient<'_>,
    Address,
    BytesN<32>,
    BytesN<32>,
) {
    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(env, &contract_id);

    let token = Address::generate(env);

    let from = BytesN::from_array(env, &[0u8; 32]);
    let to = BytesN::from_array(env, &[1u8; 32]);

    (contract_id, client, token, from, to)
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
            .set(&DataKey::VaultState(id.clone()), &vault);
    });
}

#[test]
fn test_schedule_payment_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, from, to) = setup_test(&env);

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
            .get(&DataKey::VaultState(from.clone()))
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
    let (contract_id, client, _, from, to) = setup_test(&env);

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
    let (contract_id, client, _, from, to) = setup_test(&env);

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
    let (contract_id, client, token, from, to) = setup_test(&env);

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
fn test_get_balance_existing_vault() {
    let env = Env::default();
    let (contract_id, client, token, from, _) = setup_test(&env);

    let initial_balance = 1000i128;
    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        initial_balance,
    );

    let balance = client.get_balance(&from);
    assert_eq!(balance, initial_balance);
}

#[test]
fn test_get_balance_nonexistent_vault() {
    let env = Env::default();
    let (_, client, _, _, _) = setup_test(&env);

    let nonexistent_commitment = BytesN::from_array(&env, &[99u8; 32]);
    let balance = client.get_balance(&nonexistent_commitment);
    assert_eq!(balance, 0);
}

#[test]
fn test_get_balance_after_deposit() {
    let env = Env::default();
    let (contract_id, client, token, from, _) = setup_test(&env);

    // Create vault with initial balance
    let initial_balance = 500i128;
    create_vault(
        &env,
        &contract_id,
        &from,
        &Address::generate(&env),
        &token,
        initial_balance,
    );

    // Verify initial balance
    let balance = client.get_balance(&from);
    assert_eq!(balance, initial_balance);

    // Simulate deposit by updating vault balance directly
    let deposit_amount = 300i128;
    let new_balance = initial_balance + deposit_amount;
    env.as_contract(&contract_id, || {
        let mut vault: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::VaultState(from.clone()))
            .unwrap();
        vault.balance = new_balance;
        env.storage()
            .persistent()
            .set(&DataKey::VaultState(from.clone()), &vault);
    });

    // Verify updated balance
    let updated_balance = client.get_balance(&from);
    assert_eq!(updated_balance, new_balance);
}

#[test]
fn test_get_balance_after_withdraw() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, token, from, to) = setup_test(&env);

    let initial_balance = 1000i128;
    let withdraw_amount = 400i128;
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

    // Schedule payment (which reserves funds)
    client.schedule_payment(&from, &to, &withdraw_amount, &release_at);

    // Verify balance decreased after scheduling payment
    let balance = client.get_balance(&from);
    assert_eq!(balance, initial_balance - withdraw_amount);
}
