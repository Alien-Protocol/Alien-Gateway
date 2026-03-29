use soroban_sdk::{symbol_short, Address, BytesN, Env, Symbol};

/// Event topic emitted when a new username is deployed.
pub const USERNAME_DEPLOYED: Symbol = symbol_short!("USR_DEP");
/// Event topic emitted when username ownership is transferred.
#[allow(dead_code)]
pub const OWNERSHIP_TRANSFERRED: Symbol = symbol_short!("OWN_TRF");

/// Emits an event recording the deployment of a new username record.
#[allow(deprecated)]
pub fn emit_username_deployed(
    env: &Env,
    username_hash: &BytesN<32>,
    owner: &Address,
    registered_at: u64,
) {
    env.events().publish(
        (USERNAME_DEPLOYED,),
        (username_hash.clone(), owner.clone(), registered_at),
    );
}

/// Emits an event recording a username ownership transfer.
#[allow(dead_code)]
#[allow(deprecated)]
pub fn emit_ownership_transferred(
    env: &Env,
    username_hash: &BytesN<32>,
    old_owner: &Address,
    new_owner: &Address,
) {
    env.events().publish(
        (OWNERSHIP_TRANSFERRED,),
        (username_hash.clone(), old_owner.clone(), new_owner.clone()),
    );
}
