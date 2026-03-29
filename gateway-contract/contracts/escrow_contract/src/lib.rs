//! The Escrow contract handles scheduled payments between vaults.
//! This implementation focuses on security, identity commitment, and host-level authentication.

#![no_std]

pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use crate::errors::EscrowError;
use crate::events::Events;
use crate::storage::{
    increment_auto_pay_id, increment_payment_id, read_auto_pay, read_registration_contract,
    read_vault_config, read_vault_state, write_auto_pay, write_registration_contract,
    write_scheduled_payment, write_vault_config, write_vault_state,
};
use crate::types::{AutoPay, DataKey, ScheduledPayment, VaultConfig, VaultState};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, token, vec, Address, BytesN, Env, IntoVal, Symbol,
};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn initialize(env: Env, admin: Address, registration_contract: Address) {
        admin.require_auth();
        if read_registration_contract(&env).is_some() {
            panic_with_error!(&env, EscrowError::AlreadyInitialized);
        }
        write_registration_contract(&env, &registration_contract);
    }

    pub fn create_vault(env: Env, commitment: BytesN<32>, token: Address) {
        soroban_sdk::log!(
            &env,
            "[CONTRACT DEBUG] create_vault: contract address = {:?}",
            env.current_contract_address()
        );
        let registration = read_registration_contract(&env)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::CommitmentNotRegistered));

        let owner: Option<Address> = env.invoke_contract(
            &registration,
            &Symbol::new(&env, "get_owner"),
            vec![&env, commitment.into_val(&env)],
        );
        let owner =
            owner.unwrap_or_else(|| panic_with_error!(&env, EscrowError::CommitmentNotRegistered));

        owner.require_auth();

        if read_vault_config(&env, &commitment).is_some() {
            panic_with_error!(&env, EscrowError::VaultAlreadyExists);
        }

        write_vault_config(
            &env,
            &commitment,
            &VaultConfig {
                owner: owner.clone(),
                token: token.clone(),
                created_at: env.ledger().timestamp(),
            },
        );

        write_vault_state(
            &env,
            &commitment,
            &VaultState {
                balance: 0,
                is_active: true,
            },
        );

        Events::vault_crt(&env, commitment, token, owner);
    }

    pub fn deposit(env: Env, commitment: BytesN<32>, amount: i128) {
        soroban_sdk::log!(
            &env,
            "[CONTRACT DEBUG] deposit: contract address = {:?}",
            env.current_contract_address()
        );
        if amount <= 0 {
            panic_with_error!(&env, EscrowError::InvalidAmount);
        }

        let config = read_vault_config(&env, &commitment)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::VaultNotFound));
        let mut state = read_vault_state(&env, &commitment)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::VaultNotFound));

        config.owner.require_auth();

        if !state.is_active {
            panic_with_error!(&env, EscrowError::VaultInactive);
        }

        let token_client = token::Client::new(&env, &config.token);
        token_client.transfer(&config.owner, env.current_contract_address(), &amount);

        state.balance = state
            .balance
            .checked_add(amount)
            .expect("vault balance overflow");
        write_vault_state(&env, &commitment, &state);

        Events::deposit(&env, commitment, amount, state.balance);
    }

    pub fn withdraw(env: Env, commitment: BytesN<32>, amount: i128) {
        soroban_sdk::log!(
            &env,
            "[CONTRACT DEBUG] withdraw: contract address = {:?}",
            env.current_contract_address()
        );
        if amount <= 0 {
            panic_with_error!(&env, EscrowError::InvalidAmount);
        }

        let config = read_vault_config(&env, &commitment)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::VaultNotFound));
        let mut state = read_vault_state(&env, &commitment)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::VaultNotFound));

        config.owner.require_auth();

        if !state.is_active {
            panic_with_error!(&env, EscrowError::VaultInactive);
        }

        if state.balance < amount {
            panic_with_error!(&env, EscrowError::InsufficientBalance);
        }

        let token_client = token::Client::new(&env, &config.token);
        token_client.transfer(&env.current_contract_address(), &config.owner, &amount);

        state.balance = state
            .balance
            .checked_sub(amount)
            .expect("vault balance underflow");
        write_vault_state(&env, &commitment, &state);

        Events::withdraw(&env, commitment, amount, state.balance);
    }

    pub fn schedule_payment(
        env: Env,
        from: BytesN<32>,
        to: BytesN<32>,
        amount: i128,
        release_at: u64,
    ) -> Result<u32, EscrowError> {
        soroban_sdk::log!(
            &env,
            "[CONTRACT DEBUG] schedule_payment: contract address = {:?}",
            env.current_contract_address()
        );
        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        if release_at <= env.ledger().timestamp() {
            return Err(EscrowError::PastReleaseTime);
        }

        let config = read_vault_config(&env, &from).ok_or(EscrowError::VaultNotFound)?;
        let mut state = read_vault_state(&env, &from).ok_or(EscrowError::VaultNotFound)?;

        config.owner.require_auth();

        if !state.is_active {
            return Err(EscrowError::VaultInactive);
        }

        if state.balance < amount {
            return Err(EscrowError::InsufficientBalance);
        }

        state.balance -= amount;
        write_vault_state(&env, &from, &state);

        let payment_id = increment_payment_id(&env)?;

        let payment = ScheduledPayment {
            from,
            to,
            token: config.token.clone(),
            amount,
            release_at,
            executed: false,
        };
        write_scheduled_payment(&env, payment_id, &payment);

        Events::schedule_pay(
            &env,
            payment_id,
            payment.from,
            payment.to,
            payment.amount,
            payment.release_at,
        );

        Ok(payment_id)
    }

    pub fn execute_scheduled(env: Env, payment_id: u32) {
        soroban_sdk::log!(
            &env,
            "[CONTRACT DEBUG] execute_scheduled: contract address = {:?}",
            env.current_contract_address()
        );

        let key = crate::types::DataKey::ScheduledPayment(payment_id);
        let payment: crate::types::ScheduledPayment =
            env.storage().persistent().get(&key).unwrap_or_else(|| {
                soroban_sdk::log!(
                    &env,
                    "[CONTRACT DEBUG] execute_scheduled: payment_id {} not found",
                    payment_id
                );
                panic_with_error!(&env, crate::errors::EscrowError::PaymentNotFound)
            });
        soroban_sdk::log!(
            &env,
            "[CONTRACT DEBUG] execute_scheduled: about to read_vault_state for commitment {:?}",
            payment.from
        );
        let vault_state_exists = crate::storage::read_vault_state(&env, &payment.from).is_some();
        soroban_sdk::log!(
            &env,
            "[CONTRACT DEBUG] execute_scheduled: vault_state_exists = {}",
            vault_state_exists
        );
        let key = DataKey::ScheduledPayment(payment_id);
        let mut payment: ScheduledPayment = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::PaymentNotFound));

        if payment.executed {
            panic_with_error!(&env, EscrowError::PaymentAlreadyExecuted);
        }

        if env.ledger().timestamp() < payment.release_at {
            panic_with_error!(&env, EscrowError::PaymentNotYetDue);
        }

        soroban_sdk::log!(&env, "[CONTRACT DEBUG] execute_scheduled: about to read_vault_state (source) for commitment {:?}", payment.from);
        let state = read_vault_state(&env, &payment.from)
            .unwrap_or_else(|| {
                soroban_sdk::log!(&env, "[CONTRACT DEBUG] execute_scheduled: read_vault_state (source) FAILED for commitment {:?}", payment.from);
                panic_with_error!(&env, EscrowError::VaultNotFound)
            });
        if !state.is_active {
            panic_with_error!(&env, EscrowError::VaultInactive);
        }

        soroban_sdk::log!(
            &env,
            "[CONTRACT DEBUG] execute_scheduled: about to resolve recipient for to commitment {:?}",
            payment.to
        );
        let recipient = resolve(&env, &payment.to);
        let token_client = token::Client::new(&env, &payment.token);
        token_client.transfer(&env.current_contract_address(), &recipient, &payment.amount);

        soroban_sdk::log!(&env, "[CONTRACT DEBUG] execute_scheduled: after payment, about to read_vault_state (source) for commitment {:?}", payment.from);
        let _ = read_vault_state(&env, &payment.from);

        payment.executed = true;
        write_scheduled_payment(&env, payment_id, &payment);

        Events::pay_exec(&env, payment_id, payment.from, payment.to, payment.amount);
    }

    pub fn cancel_vault(env: Env, commitment: BytesN<32>) {
        let config = read_vault_config(&env, &commitment)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::VaultNotFound));
        config.owner.require_auth();

        let mut state = read_vault_state(&env, &commitment)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::VaultNotFound));

        let refunded_amount = if state.balance > 0 {
            let token_client = token::Client::new(&env, &config.token);
            token_client.transfer(
                &env.current_contract_address(),
                &config.owner,
                &state.balance,
            );
            state.balance
        } else {
            0
        };

        state.is_active = false;
        state.balance = 0;
        write_vault_state(&env, &commitment, &state);

        Events::vault_cancel(&env, commitment, refunded_amount);
    }

    pub fn setup_auto_pay(
        env: Env,
        from: BytesN<32>,
        to: BytesN<32>,
        amount: i128,
        interval: u64,
    ) -> Result<u32, EscrowError> {
        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        if interval == 0 {
            return Err(EscrowError::InvalidInterval);
        }

        let config = read_vault_config(&env, &from).ok_or(EscrowError::VaultNotFound)?;

        config.owner.require_auth();

        let rule_id = increment_auto_pay_id(&env)?;

        let auto_pay = AutoPay {
            from: from.clone(),
            to: to.clone(),
            token: config.token.clone(),
            amount,
            interval,
            last_paid: 0,
        };
        write_auto_pay(&env, &from, rule_id, &auto_pay);

        Events::auto_set(&env, rule_id, from, to, amount, interval);

        Ok(rule_id)
    }

    pub fn trigger_auto_pay(env: Env, from: BytesN<32>, rule_id: u32) {
        let mut auto_pay = read_auto_pay(&env, &from, rule_id)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::AutoPayNotFound));

        let current_time = env.ledger().timestamp();
        let next_payment_time = auto_pay.last_paid + auto_pay.interval;

        if current_time < next_payment_time {
            panic_with_error!(&env, EscrowError::IntervalNotElapsed);
        }

        let mut state = read_vault_state(&env, &from)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::VaultNotFound));

        if !state.is_active {
            panic_with_error!(&env, EscrowError::VaultInactive);
        }

        if state.balance < auto_pay.amount {
            panic_with_error!(&env, EscrowError::InsufficientBalance);
        }

        let recipient = resolve(&env, &auto_pay.to);

        let token_client = token::Client::new(&env, &auto_pay.token);
        token_client.transfer(
            &env.current_contract_address(),
            &recipient,
            &auto_pay.amount,
        );

        state.balance -= auto_pay.amount;
        write_vault_state(&env, &from, &state);

        auto_pay.last_paid = current_time;
        write_auto_pay(&env, &from, rule_id, &auto_pay);

        Events::auto_pay(
            &env,
            rule_id,
            auto_pay.from,
            auto_pay.to,
            auto_pay.amount,
            current_time,
        );
    }

    pub fn get_balance(env: Env, commitment: BytesN<32>) -> Option<i128> {
        read_vault_state(&env, &commitment).map(|state| state.balance)
    }

    pub fn get_auto_pay(env: Env, from: BytesN<32>, rule_id: u32) -> Option<AutoPay> {
        read_auto_pay(&env, &from, rule_id)
    }

    pub fn is_vault_active(env: Env, commitment: BytesN<32>) -> Option<bool> {
        read_vault_state(&env, &commitment).map(|state| state.is_active)
    }
}

fn resolve(env: &Env, commitment: &BytesN<32>) -> Address {
    let config = read_vault_config(env, commitment)
        .unwrap_or_else(|| panic_with_error!(env, EscrowError::VaultNotFound));
    config.owner
}
