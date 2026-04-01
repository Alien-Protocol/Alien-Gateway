use soroban_sdk::{contracttype, Address, BytesN};

/// Storage keys for the Escrow contract's persistent and instance storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Key for a vault's immutable configuration, indexed by its BytesN<32> commitment.
    VaultConfig(BytesN<32>),
    /// Key for a vault's mutable state, indexed by its BytesN<32> commitment.
    VaultState(BytesN<32>),
    /// Key for a specific scheduled payment, indexed by its unique payment_id (u32).
    ScheduledPayment(u32),
    /// Key for the auto-incrementing payment counter in instance storage.
    PaymentCounter,
    /// Key for an auto-payment rule, indexed by the source vault's commitment and a rule ID.
    AutoPay(BytesN<32>, u64),
    /// Key for the auto-incrementing auto-pay counter in instance storage.
    AutoPayCounter,
    /// Legacy key for a vault record (pre-split architecture). Kept for backward compatibility.
    Vault(BytesN<32>),
    /// Key for the Registration contract address stored in instance storage.
    RegistrationContract,
}

/// Immutable configuration for a vault. Written once at creation, never mutated.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultConfig {
    /// The Stellar address authorized to manage this vault.
    pub owner: Address,
    /// The asset token associated with this vault.
    pub token: Address,
    /// The ledger timestamp at which this vault was created.
    pub created_at: u64,
}

/// Mutable runtime state for a vault.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultState {
    /// The current available balance in the vault.
    pub balance: i128,
    /// Whether the vault is currently active and accepting operations.
    pub is_active: bool,
}

/// Represents a payment that has been scheduled but not yet executed.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduledPayment {
    /// The commitment identifier of the source vault.
    pub from: BytesN<32>,
    /// The commitment identifier of the intended recipient.
    pub to: BytesN<32>,
    /// The token to be transferred upon execution.
    pub token: Address,
    /// The amount of tokens to be transferred.
    pub amount: i128,
    /// The timestamp at or after which the payment can be executed.
    pub release_at: u64,
    /// Whether the payment has already been executed.
    pub executed: bool,
}

/// Legacy combined vault record (pre-split architecture; kept for migration compatibility).
///
/// - Historical key: `DataKey::Vault(BytesN<32>)`.
/// - Current split keys: `DataKey::VaultConfig(BytesN<32>)` + `DataKey::VaultState(BytesN<32>)`.
/// - Migration path: `read_vault_config` and `read_vault_state` in `storage.rs` first query the split keys;
///   if absent, they fall back to `LegacyVault` and project into the new types.
/// - Snapshot tests that currently record the legacy key form are in:
///   - `gateway-contract/contracts/escrow_contract/test_snapshots/test/test_auto_pay_setup_success.1.json`
///   - `gateway-contract/contracts/escrow_contract/test_snapshots/test/test_auto_pay_trigger_success.1.json`
///   - `gateway-contract/contracts/escrow_contract/test_snapshots/test/test_auto_pay_second_cycle_success.1.json`
///   - `gateway-contract/contracts/escrow_contract/test_snapshots/test/test_auto_pay_insufficient_balance_panics.1.json`
///   - `gateway-contract/contracts/escrow_contract/test_snapshots/test/test_auto_pay_early_trigger_panics.1.json`
///
/// Removing the legacy form is safe after a one-time migration rewrites every on-chain
/// `DataKey::Vault(BytesN<32>)` entry into `DataKey::VaultConfig(BytesN<32>)` plus
/// `DataKey::VaultState(BytesN<32>)`, and those snapshot fixtures are regenerated.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyVault {
    pub owner: Address,
    pub token: Address,
    pub created_at: u64,
    pub balance: i128,
    pub is_active: bool,
}

/// Represents a recurring automatic payment rule between two vaults.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutoPay {
    /// The commitment identifier of the source vault.
    pub from: BytesN<32>,
    /// The commitment identifier of the destination vault.
    pub to: BytesN<32>,
    /// The token to be transferred on each execution.
    pub token: Address,
    /// The amount of tokens to transfer per interval.
    pub amount: i128,
    /// The time interval in ledger seconds between automatic payments.
    pub interval: u64,
    /// The ledger timestamp of the last successful payment (0 if never executed).
    pub last_paid: u64,
}
