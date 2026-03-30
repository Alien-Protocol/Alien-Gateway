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
    /// Initializes the contract by storing the Registration contract address.
    ///
    /// ### Arguments
    /// - `admin`: The address that must authorize this initialization.
    /// - `registration_contract`: The address of the deployed Registration contract.
    ///
    /// ### Errors
    /// - `AlreadyInitialized`: If the Registration contract address is already set.
    pub fn initialize(env: Env, admin: Address, registration_contract: Address) {
        admin.require_auth();
        if read_registration_contract(&env).is_some() {
            panic_with_error!(&env, EscrowError::AlreadyInitialized);
        }
        write_registration_contract(&env, &registration_contract);
    }

    /// Creates a new vault for a registered commitment.
    ///
    /// The owner is resolved by calling `get_owner` on the Registration contract.
    /// The caller must be the registered owner of the commitment.
    ///
    /// ### Arguments
    /// - `commitment`: The BytesN<32> identity commitment (Poseidon hash of username).
    /// - `token`: The Stellar asset address this vault will hold.
    ///
    /// ### Errors
    /// - `CommitmentNotRegistered`: If no owner is found for the commitment.
    /// - `VaultAlreadyExists`: If a vault already exists for this commitment.
    pub fn create_vault(env: Env, commitment: BytesN<32>, token: Address) {
        soroban_sdk::log!(
            &env,
            "[CONTRACT DEBUG] create_vault: contract address = {:?}",
            env.current_contract_address()
        );
        // 1. Load Registration contract address (must be initialized first).
        let registration = read_registration_contract(&env)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::CommitmentNotRegistered));

        // 2. Cross-contract call: resolve owner from Registration contract.
        let owner: Option<Address> = env.invoke_contract(
            &registration,
            &Symbol::new(&env, "get_owner"),
            vec![&env, commitment.into_val(&env)],
        );
        let owner =
            owner.unwrap_or_else(|| panic_with_error!(&env, EscrowError::CommitmentNotRegistered));

        // 3. Authenticate: the resolved owner must sign this transaction.
        owner.require_auth();

        // 4. Existence guard: reject if a vault already exists for this commitment.
        if read_vault_config(&env, &commitment).is_some() {
            panic_with_error!(&env, EscrowError::VaultAlreadyExists);
        }

        // 5. Store immutable vault configuration.
        write_vault_config(
            &env,
            &commitment,
            &VaultConfig {
                owner: owner.clone(),
                token: token.clone(),
                created_at: env.ledger().timestamp(),
            },
        );

        // 6. Store initial mutable vault state.
        write_vault_state(
            &env,
            &commitment,
            &VaultState {
                balance: 0,
                is_active: true,
            },
        );

        // 7. Emit VAULT_CRT event with fields in order: (commitment, token, owner).
        Events::vault_crt(&env, commitment, token, owner);
    }

    /// Deposits tokens into an existing vault and increases its internal balance.
    ///
    /// The vault owner must authorize this call. Tokens are transferred from the
    /// owner to this contract before the vault balance is updated.
    ///
    /// ### Arguments
    /// - `commitment`: The `BytesN<32>` identity commitment of the vault.
    /// - `amount`: The amount of tokens to deposit. Must be > 0.
    ///
    /// ### Errors
    /// - `InvalidAmount`: If `amount <= 0`.
    /// - `VaultNotFound`: If the vault does not exist.
    /// - `VaultInactive`: If the vault is cancelled/inactive.
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

        // Transfer tokens from caller to the contract first
        let token_client = token::Client::new(&env, &config.token);
        token_client.transfer(&config.owner, env.current_contract_address(), &amount);

        // Update state safely
        state.balance = state
            .balance
            .checked_add(amount)
            .expect("vault balance overflow");
        write_vault_state(&env, &commitment, &state);

        // Emit DEPOSIT event.
        Events::deposit(&env, commitment, amount, state.balance);
    }

    /// Withdraws tokens from an existing vault.
    ///
    /// The vault owner must authorize this call. Tokens are transferred from the
    /// contract to the owner before the vault balance is updated.
    ///
    /// ### Arguments
    /// - `commitment`: The BytesN<32> identity commitment of the vault.
    /// - `amount`: The amount of tokens to withdraw. Must be > 0.
    ///
    /// ### Errors
    /// - `InvalidAmount`: If `amount <= 0`.
    /// - `VaultNotFound`: If the vault does not exist.
    /// - `VaultInactive`: If the vault is cancelled/inactive.
    /// - `InsufficientBalance`: If the vault balance is less than `amount`.
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

        // Transfer tokens from contract to owner
        let token_client = token::Client::new(&env, &config.token);
        token_client.transfer(&env.current_contract_address(), &config.owner, &amount);

        // Update state safely
        state.balance = state
            .balance
            .checked_sub(amount)
            .expect("vault balance underflow");
        write_vault_state(&env, &commitment, &state);

        // Emit WITHDRAW event.
        Events::withdraw(&env, commitment, amount, state.balance);
    }

    /// Schedules a payment from one vault to another.
    ///
    /// Funds are reserved in the source vault immediately upon scheduling.
    /// The payment can be executed at or after the `release_at` timestamp.
    ///
    /// ### Arguments
    /// - `from`: The commitment ID of the source vault.
    /// - `to`: The commitment ID of the destination vault.
    /// - `amount`: The amount of tokens to schedule. Must be > 0.
    /// - `release_at`: The ledger timestamp (u64) for release. Must be > current time.
    ///
    /// ### Returns
    /// - `u32`: The unique payment ID assigned to this schedule.
    ///
    /// ### Errors
    /// - `VaultNotFound`: If the `from` vault does not exist.
    /// - `InvalidAmount`: If `amount <= 0`.
    /// - `InsufficientBalance`: If the vault has less than `amount`.
    /// - `PastReleaseTime`: If `release_at` is not in the future.
    /// - `PaymentCounterOverflow`: If the global ID counter overflows.
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
        // 1. Validate Input
        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        if release_at <= env.ledger().timestamp() {
            return Err(EscrowError::PastReleaseTime);
        }

        // 2. Read Vault (config + state separately)
        let config = read_vault_config(&env, &from).ok_or(EscrowError::VaultNotFound)?;
        let mut state = read_vault_state(&env, &from).ok_or(EscrowError::VaultNotFound)?;

        // 3. Authenticate caller as owner of from vault
        // Host-level authentication. Panics with host error if unauthorized.
        config.owner.require_auth();

        // 4. Reject if vault is inactive
        if !state.is_active {
            return Err(EscrowError::VaultInactive);
        }

        // 5. Validate Balance
        if state.balance < amount {
            return Err(EscrowError::InsufficientBalance);
        }

        // 6. Reserve Funds
        state.balance -= amount;
        write_vault_state(&env, &from, &state);

        // 7. Generate Payment ID
        let payment_id = increment_payment_id(&env)?;

        // 8. Store Scheduled Payment
        let payment = ScheduledPayment {
            from,
            to,
            token: config.token.clone(),
            amount,
            release_at,
            executed: false,
        };
        write_scheduled_payment(&env, payment_id, &payment);

        // 9. Emit Event
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

    /// Executes a previously scheduled payment once its release time has passed.
    ///
    /// Transfers the reserved tokens from this contract to the resolved owner of
    /// the destination vault. Can be called by anyone (trustless keeper/bot).
    ///
    /// ### Arguments
    /// - `payment_id`: The unique ID returned by [`EscrowContract::schedule_payment`].
    ///
    /// ### Errors
    /// - `PaymentNotFound`: If no payment exists for `payment_id`.
    /// - `PaymentAlreadyExecuted`: If the payment has already been executed.
    /// - `PaymentNotYetDue`: If the current ledger time is before `release_at`.
    /// - `VaultNotFound`: If the source vault no longer exists.
    /// - `VaultInactive`: If the source vault has been cancelled.
    pub fn execute_scheduled(env: Env, payment_id: u32) {
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

        // Reject execution if the source vault was cancelled.
        let state = read_vault_state(&env, &payment.from)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::VaultNotFound));
        if !state.is_active {
            panic_with_error!(&env, EscrowError::VaultInactive);
        }

        let recipient = resolve(&env, &payment.to);
        let token_client = token::Client::new(&env, &payment.token);
        token_client.transfer(&env.current_contract_address(), &recipient, &payment.amount);

        payment.executed = true;
        write_scheduled_payment(&env, payment_id, &payment);

        Events::pay_exec(&env, payment_id, payment.from, payment.to, payment.amount);
    }

    /// Cancels an existing vault by commitment.
    ///
    /// Marks the vault as inactive and refunds any remaining balance to the owner.
    /// Once cancelled, no new deposits/payments/auto-pays should be triggerable on it.
    ///
    /// ### Arguments
    /// - `commitment`: The `BytesN<32>` identity commitment of the vault to cancel.
    ///
    /// ### Errors
    /// - `VaultNotFound`: If no vault exists for `commitment`.
    pub fn cancel_vault(env: Env, commitment: BytesN<32>) {
        // 1) Load vault config + authenticate as owner.
        let config = read_vault_config(&env, &commitment)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::VaultNotFound));
        config.owner.require_auth();

        // 2) Load vault mutable state (panic if vault doesn't exist).
        let mut state = read_vault_state(&env, &commitment)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::VaultNotFound));

        // 3) Refund any remaining balance.
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

        // 4) Mark inactive and zero balance.
        state.is_active = false;
        state.balance = 0;
        write_vault_state(&env, &commitment, &state);

        // 5) Emit cancellation event.
        Events::vault_cancel(&env, commitment, refunded_amount);
    }

    /// Withdraws tokens from an active vault back to the owner.
    ///
    /// The vault owner must authorize this call. The vault remains active and
    /// its balance is reduced by the withdrawn amount.
    pub fn withdraw(env: Env, commitment: BytesN<32>, amount: i128) {
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

        state.balance -= amount;
        write_vault_state(&env, &commitment, &state);

        let token_client = token::Client::new(&env, &config.token);
        token_client.transfer(&env.current_contract_address(), &config.owner, &amount);
    }

    /// Registers a recurring payment rule.
    ///
    /// Once registered, calling `trigger_auto_pay` will send `amount` tokens
    /// every `interval` seconds from the sender's vault to the recipient's resolved address.
    ///
    /// ### Arguments
    /// - `from`: The commitment ID of the source vault.
    /// - `to`: The commitment ID of the destination vault.
    /// - `amount`: The amount of tokens to send each interval. Must be > 0.
    /// - `interval`: The interval in seconds between payments. Must be > 0.
    ///
    /// ### Returns
    /// - `u32`: The unique rule_id assigned to this auto-pay rule.
    ///
    /// ### Errors
    /// - `VaultNotFound`: If the `from` vault does not exist.
    /// - `InvalidAmount`: If `amount <= 0`.
    /// - `InvalidInterval`: If `interval == 0`.
    /// - `AutoPayCounterOverflow`: If the global ID counter overflows.
    pub fn setup_auto_pay(
        env: Env,
        from: BytesN<32>,
        to: BytesN<32>,
        amount: i128,
        interval: u64,
    ) -> Result<u32, EscrowError> {
        // 1. Validate Input
        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        if interval == 0 {
            return Err(EscrowError::InvalidInterval);
        }

        // 2. Reject self-payment
        if from == to {
            return Err(EscrowError::SelfPaymentNotAllowed);
        }

        // 3. Read Vault config to verify it exists and get the token
        let config = read_vault_config(&env, &from).ok_or(EscrowError::VaultNotFound)?;

        // 3. Authenticate caller as owner of from vault
        config.owner.require_auth();

        // 4. Generate AutoPay rule ID
        let rule_id = increment_auto_pay_id(&env)?;

        // 5. Store AutoPay Rule under composite key (from, rule_id)
        let auto_pay = AutoPay {
            from: from.clone(),
            to: to.clone(),
            token: config.token.clone(),
            amount,
            interval,
            last_paid: 0,
        };
        write_auto_pay(&env, &from, rule_id, &auto_pay);

        // 6. Emit Event
        Events::auto_set(&env, rule_id, from, to, amount, interval);

        Ok(rule_id)
    }

    /// Executes one cycle of a recurring auto-pay rule if enough time has passed.
    ///
    /// This function is trustless and can be called by anyone (bots, keeper scripts, SDK).
    /// It checks if the interval has elapsed since the last payment, validates the vault
    /// balance, transfers the tokens, and updates the state.
    ///
    /// ### Arguments
    /// - `from`: The commitment ID of the source vault that owns the rule.
    /// - `rule_id`: The unique identifier of the auto-pay rule to trigger.
    ///
    /// ### Errors
    /// - Panics with `AutoPayNotFound` if the auto-pay rule does not exist.
    /// - Panics with `IntervalNotElapsed` if called before the interval has elapsed.
    /// - Panics with `VaultNotFound` if the source vault does not exist.
    /// - Panics with `InsufficientBalance` if the vault balance is less than the payment amount.
    pub fn trigger_auto_pay(env: Env, from: BytesN<32>, rule_id: u32) {
        // 1. Load AutoPay rule via composite key
        let mut auto_pay = read_auto_pay(&env, &from, rule_id)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::AutoPayNotFound));

        // 2. Check if interval has elapsed
        let current_time = env.ledger().timestamp();
        let next_payment_time = auto_pay.last_paid + auto_pay.interval;

        if current_time < next_payment_time {
            panic_with_error!(&env, EscrowError::IntervalNotElapsed);
        }

        // 3. Load vault state, verify the vault is active before checking balance.
        let mut state = read_vault_state(&env, &from)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::VaultNotFound));

        // Reject if the source vault was cancelled.
        if !state.is_active {
            panic_with_error!(&env, EscrowError::VaultInactive);
        }

        if state.balance < auto_pay.amount {
            panic_with_error!(&env, EscrowError::InsufficientBalance);
        }

        // 4. Resolve recipient address
        let recipient = resolve(&env, &auto_pay.to);

        // 5. Transfer tokens from contract to recipient
        let token_client = token::Client::new(&env, &auto_pay.token);
        token_client.transfer(
            &env.current_contract_address(),
            &recipient,
            &auto_pay.amount,
        );

        // 6. Decrement vault balance
        state.balance -= auto_pay.amount;
        write_vault_state(&env, &from, &state);

        // 7. Update last_paid timestamp
        auto_pay.last_paid = current_time;
        write_auto_pay(&env, &from, rule_id, &auto_pay);

        // 8. Emit event
        Events::auto_pay(
            &env,
            rule_id,
            auto_pay.from,
            auto_pay.to,
            auto_pay.amount,
            current_time,
        );
    }

    /// Returns the current balance of a vault identified by its commitment.
    ///
    /// This is a read-only getter with no side effects and no authentication
    /// requirement. It performs a single O(1) persistent-storage lookup.
    ///
    /// ### Arguments
    /// - `commitment`: The `BytesN<32>` identifier of the vault.
    ///
    /// ### Returns
    /// - `None` if the vault does not exist.
    /// - `Some(balance)` with the vault's current available balance.
    pub fn get_balance(env: Env, commitment: BytesN<32>) -> Option<i128> {
        read_vault_state(&env, &commitment).map(|state| state.balance)
    }

    /// Returns the auto-pay rule for a given source vault and rule ID.
    ///
    /// This is a read-only getter with no side effects and no authentication
    /// requirement. It performs a single O(1) persistent-storage lookup using
    /// the composite key `(from, rule_id)`.
    ///
    /// ### Arguments
    /// - `from`: The `BytesN<32>` commitment ID of the source vault that owns the rule.
    /// - `rule_id`: The unique identifier of the auto-pay rule.
    ///
    /// ### Returns
    /// - `Some(AutoPay)` if the rule exists (i.e. after `setup_auto_pay`).
    /// - `None` if the rule does not exist or has been cancelled.
    pub fn get_auto_pay(env: Env, from: BytesN<32>, rule_id: u32) -> Option<AutoPay> {
        read_auto_pay(&env, &from, rule_id)
    }
}

fn resolve(env: &Env, commitment: &BytesN<32>) -> Address {
    let config = read_vault_config(env, commitment)
        .unwrap_or_else(|| panic_with_error!(env, EscrowError::VaultNotFound));
    config.owner
}
