use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    /// The vault balance is insufficient to cover the requested amount.
    InsufficientBalance = 1,
    /// The release timestamp must be in the future relative to the current ledger time.
    PastReleaseTime = 2,
    /// The requested amount must be strictly greater than 0.
    InvalidAmount = 4,
    /// The specified vault commitment was not found in the persistent storage.
    VaultNotFound = 5,
    /// The payment counter has reached its maximum value (u32::MAX), preventing new IDs.
    PaymentCounterOverflow = 6,
    /// The specified scheduled payment was not found.
    PaymentNotFound = 7,
    /// The scheduled payment has already been executed.
    PaymentAlreadyExecuted = 8,
    /// The scheduled payment is not yet due for execution.
    PaymentNotYetDue = 9,
}
