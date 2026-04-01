use soroban_sdk::{panic_with_error, Address, Bytes, BytesN, Env};

use crate::errors::CoreError;
use crate::events::TRANSFER_EVENT;
use crate::registration;
use crate::types::PublicSignals;
use crate::{smt_root, zk_verifier};

pub struct Transfer;

impl Transfer {
    /// Transfers username ownership to a new owner without ZK verification.
    ///
    /// A simple ownership transfer where the current owner directly assigns the username to a new owner.
    /// Both caller and new owner must be different. This operation does NOT require a ZK proof.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `caller`: The current owner authorizing the transfer. Must be authorized.
    /// - `commitment`: The 32-byte username commitment being transferred.
    /// - `new_owner`: The address that will become the new owner.
    ///
    /// ### Errors
    /// - `NotFound`: If the commitment is not registered.
    /// - `Unauthorized`: If the caller is not the current owner.
    /// - `SameOwner`: If the new owner is the same as the current owner.
    ///
    /// ### Events
    /// - Emits `TRANSFER_EVENT` with (commitment, old_owner, new_owner).
    pub fn transfer_ownership(
        env: Env,
        caller: Address,
        commitment: BytesN<32>,
        new_owner: Address,
    ) {
        caller.require_auth();
        let key = registration::DataKey::Commitment(commitment.clone());
        let current_owner: Address = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound));
        if caller != current_owner {
            panic_with_error!(&env, CoreError::Unauthorized);
        }
        if new_owner == current_owner {
            panic_with_error!(&env, CoreError::SameOwner);
        }
        env.storage().persistent().set(&key, &new_owner);
        #[allow(deprecated)]
        env.events()
            .publish((TRANSFER_EVENT,), (commitment, caller, new_owner));
    }

    /// Transfers username ownership with zero-knowledge proof validation.
    ///
    /// An advanced ownership transfer that requires a valid ZK proof, enabling secure transfers
    /// when the current owner cannot directly authorize (e.g., keyless recovery scenarios).
    /// The proof must be valid against the current SMT root. Upon success, the SMT root is updated.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `caller`: The authorizing address for the transfer. Must be authorized.
    /// - `commitment`: The 32-byte username commitment being transferred.
    /// - `new_owner`: The address that will become the new owner.
    /// - `proof`: Serialized Groth16 proof validating the transfer.
    /// - `public_signals`: Public inputs including old_root, new_root, and commitment.
    ///
    /// ### Errors
    /// - `NotFound`: If the commitment is not registered.
    /// - `Unauthorized`: If the caller is not the current owner.
    /// - `SameOwner`: If the new owner is the same as the current owner.
    /// - `RootNotSet`: If the SMT root has not been initialized.
    /// - `StaleRoot`: If the proof's old_root doesn't match the current SMT root.
    /// - `InvalidProof`: If the ZK proof verification fails.
    ///
    /// ### Events
    /// - Emits `TRANSFER_EVENT` with (commitment, old_owner, new_owner).
    /// - Updates the SMT root via `ROOT_UPDATED` event.
    pub fn transfer(
        env: Env,
        caller: Address,
        commitment: BytesN<32>,
        new_owner: Address,
        proof: Bytes,
        public_signals: PublicSignals,
    ) {
        caller.require_auth();
        let key = registration::DataKey::Commitment(commitment.clone());
        let current_owner: Address = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound));
        if caller != current_owner {
            panic_with_error!(&env, CoreError::Unauthorized);
        }
        if new_owner == current_owner {
            panic_with_error!(&env, CoreError::SameOwner);
        }
        let current_root = smt_root::SmtRoot::get_root(env.clone())
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::RootNotSet));
        if public_signals.old_root != current_root {
            panic_with_error!(&env, CoreError::StaleRoot);
        }
        if !zk_verifier::ZkVerifier::verify_groth16_proof(&env, &proof, &public_signals) {
            panic_with_error!(&env, CoreError::InvalidProof);
        }
        env.storage().persistent().set(&key, &new_owner);
        smt_root::SmtRoot::update_root(&env, public_signals.new_root);
        #[allow(deprecated)]
        env.events()
            .publish((TRANSFER_EVENT,), (commitment, caller, new_owner));
    }
}
