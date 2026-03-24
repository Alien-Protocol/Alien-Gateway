#![allow(dead_code)]

use soroban_sdk::{symbol_short, Symbol};

pub const INIT_EVENT: Symbol = symbol_short!("INIT");
pub const TRANSFER_EVENT: Symbol = symbol_short!("TRANSFER");
pub const REGISTER_EVENT: Symbol = symbol_short!("REGISTER");
pub const ROOT_UPDATED: Symbol = symbol_short!("ROOT_UPD");
pub const MASTER_SET: Symbol = symbol_short!("MSTR_SET");
pub const ADDR_ADDED: Symbol = symbol_short!("ADDR_ADD");
pub const CHAIN_ADD: Symbol = symbol_short!("CHAIN_ADD");
pub const CHAIN_REM: Symbol = symbol_short!("CHAIN_REM");
pub const VAULT_CREATE: Symbol = symbol_short!("VAULT_CRT");
pub const DEPOSIT: Symbol = symbol_short!("DEPOSIT");
pub const WITHDRAW: Symbol = symbol_short!("WITHDRAW");
pub const SCHED_PAY: Symbol = symbol_short!("SCHED_PAY");
