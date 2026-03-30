use soroban_sdk::{contractevent, symbol_short, Address, BytesN, Env};

/// Event emitted when a new payment is scheduled.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulePayEvent {
    /// The unique identifier assigned to this payment.
    #[topic]
    pub payment_id: u32,
    /// The commitment identifier of the source vault.
    pub from: BytesN<32>,
    /// The commitment identifier of the intended recipient.
    pub to: BytesN<32>,
    /// The amount of tokens to be transferred.
    pub amount: i128,
    /// The timestamp at or after which the payment can be executed.
    pub release_at: u64,
}

/// Event emitted when a scheduled payment is executed.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayExecEvent {
    /// The unique identifier assigned to this payment.
    #[topic]
    pub payment_id: u32,
    /// The commitment identifier of the source vault.
    pub from: BytesN<32>,
    /// The commitment identifier of the intended recipient.
    pub to: BytesN<32>,
    /// The amount of tokens transferred.
    pub amount: i128,
}

/// Event emitted when a new auto-pay rule is registered.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutoSetEvent {
    /// The unique identifier assigned to this auto-pay rule.
    #[topic]
    pub auto_pay_id: u32,
    /// The commitment identifier of the source vault.
    pub from: BytesN<32>,
    /// The commitment identifier of the intended recipient.
    pub to: BytesN<32>,
    /// The amount of tokens to be transferred each interval.
    pub amount: i128,
    /// The interval in seconds between automatic payments.
    pub interval: u64,
}

/// Event emitted when an auto-pay rule is triggered and payment is executed.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutoPayEvent {
    /// The unique identifier of the auto-pay rule that was triggered.
    #[topic]
    pub auto_pay_id: u32,
    /// The commitment identifier of the source vault.
    pub from: BytesN<32>,
    /// The commitment identifier of the recipient.
    pub to: BytesN<32>,
    /// The amount of tokens transferred.
    pub amount: i128,
    /// The timestamp when the payment was executed.
    pub timestamp: u64,
}

/// Event emitted when a vault is cancelled.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultCancelEvent {
    /// The commitment identifier of the cancelled vault.
    #[topic]
    pub commitment: BytesN<32>,
    /// Amount refunded back to the vault owner.
    pub refunded_amount: i128,
}

/// Event emitted when a deposit is made into a vault.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositEvent {
    /// The commitment identifier of the vault.
    #[topic]
    pub commitment: BytesN<32>,
    /// The amount of tokens deposited.
    pub amount: i128,
    /// The new balance of the vault.
    pub new_balance: i128,
}

/// Event emitted when tokens are withdrawn from a vault.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawEvent {
    /// The commitment identifier of the vault.
    #[topic]
    pub commitment: BytesN<32>,
    /// The amount of tokens withdrawn.
    pub amount: i128,
    /// The new balance of the vault.
    pub new_balance: i128,
}

/// Helper for emitting contract events.
pub struct Events;

impl Events {
    /// Emits a `SchedulePayEvent` to the host.
    pub fn schedule_pay(
        env: &Env,
        payment_id: u32,
        from: BytesN<32>,
        to: BytesN<32>,
        amount: i128,
        release_at: u64,
    ) {
        SchedulePayEvent {
            payment_id,
            from,
            to,
            amount,
            release_at,
        }
        .publish(env);
    }

    /// Emits a `PayExecEvent` to the host.
    pub fn pay_exec(env: &Env, payment_id: u32, from: BytesN<32>, to: BytesN<32>, amount: i128) {
        PayExecEvent {
            payment_id,
            from,
            to,
            amount,
        }
        .publish(env);
    }

    /// Emits a VAULT_CRT event with topics (symbol!("VAULT_CRT"), commitment)
    /// and data (token, owner), exactly as specified in Issue #71.
    #[allow(deprecated)]
    pub fn vault_crt(env: &Env, commitment: BytesN<32>, token: Address, owner: Address) {
        env.events()
            .publish((symbol_short!("VAULT_CRT"), commitment), (token, owner));
    }

    /// Emits an `AutoSetEvent` to the host.
    pub fn auto_set(
        env: &Env,
        auto_pay_id: u32,
        from: BytesN<32>,
        to: BytesN<32>,
        amount: i128,
        interval: u64,
    ) {
        AutoSetEvent {
            auto_pay_id,
            from,
            to,
            amount,
            interval,
        }
        .publish(env);
    }

    /// Emits an `AutoPayEvent` to the host.
    pub fn auto_pay(
        env: &Env,
        auto_pay_id: u32,
        from: BytesN<32>,
        to: BytesN<32>,
        amount: i128,
        timestamp: u64,
    ) {
        AutoPayEvent {
            auto_pay_id,
            from,
            to,
            amount,
            timestamp,
        }
        .publish(env);
    }

    /// Emits a `VaultCancelEvent` to the host.
    pub fn vault_cancel(env: &Env, commitment: BytesN<32>, refunded_amount: i128) {
        VaultCancelEvent {
            commitment,
            refunded_amount,
        }
        .publish(env);
    }

    /// Emits a DEPOSIT event to the host.
    pub fn deposit(env: &Env, commitment: BytesN<32>, amount: i128, new_balance: i128) {
        DepositEvent {
            commitment,
            amount,
            new_balance,
        }
        .publish(env);
    }

    /// Emits a WITHDRAW event to the host.
    pub fn withdraw(env: &Env, commitment: BytesN<32>, amount: i128, new_balance: i128) {
        WithdrawEvent {
            commitment,
            amount,
            new_balance,
        }
        .publish(env);
    }
}
