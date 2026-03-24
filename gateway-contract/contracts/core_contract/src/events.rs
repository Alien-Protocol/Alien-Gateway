use soroban_sdk::{contractevent, BytesN};

#[contractevent]
pub struct UsernameRegistered {
    pub commitment: BytesN<32>,
}

#[contractevent]
pub struct MerkleRootUpdated {
    pub old_root: BytesN<32>,
    pub new_root: BytesN<32>,
}

use soroban_sdk::{symbol_short, Symbol};

#[allow(dead_code)]
pub const INIT_EVENT: Symbol = symbol_short!("INIT");
#[allow(dead_code)]
pub const TRANSFER_EVENT: Symbol = symbol_short!("TRANSFER");
#[allow(dead_code)]
pub const REGISTER_EVENT: Symbol = symbol_short!("REGISTER");
#[allow(dead_code)]
pub const ROOT_UPDATED: Symbol = symbol_short!("ROOT_UPD");
#[allow(dead_code)]
pub const MASTER_SET: Symbol = symbol_short!("MSTR_SET");
#[allow(dead_code)]
pub const ADDR_ADDED: Symbol = symbol_short!("ADDR_ADD");
#[allow(dead_code)]
pub const CHAIN_ADD: Symbol = symbol_short!("CHAIN_ADD");
#[allow(dead_code)]
pub const CHAIN_REM: Symbol = symbol_short!("CHAIN_REM");
#[allow(dead_code)]
pub const VAULT_CREATE: Symbol = symbol_short!("VAULT_CRT");
#[allow(dead_code)]
pub const DEPOSIT: Symbol = symbol_short!("DEPOSIT");
#[allow(dead_code)]
pub const WITHDRAW: Symbol = symbol_short!("WITHDRAW");
#[allow(dead_code)]
pub const SCHED_PAY: Symbol = symbol_short!("SCHED_PAY");
