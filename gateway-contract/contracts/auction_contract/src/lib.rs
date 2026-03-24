#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Symbol, vec, IntoVal};

pub mod types;
pub mod errors;
pub mod events;
pub mod storage;

#[cfg(test)]
mod test;

#[contract]
pub struct AuctionContract;

#[contractimpl]
impl AuctionContract {
    pub fn claim_username(env: Env, username_hash: BytesN<32>, claimer: Address) -> Result<(), crate::errors::Error> {
        claimer.require_auth();

        let status = storage::get_status(&env);
        
        if status == types::AuctionStatus::Claimed {
            return Err(crate::errors::Error::AlreadyClaimed);
        }

        if status != types::AuctionStatus::Closed {
            return Err(crate::errors::Error::NotClosed);
        }

        let highest_bidder = storage::get_highest_bidder(&env);
        if highest_bidder.is_none() || highest_bidder.unwrap() != claimer {
            return Err(crate::errors::Error::NotWinner);
        }

        // Set status to Claimed
        storage::set_status(&env, types::AuctionStatus::Claimed);

        // Call factory_contract.deploy_username(username_hash, claimer)
        if let Some(factory) = storage::get_factory_contract(&env) {
            env.invoke_contract::<()>(
                &factory,
                &Symbol::new(&env, "deploy_username"),
                vec![&env, username_hash.into_val(&env), claimer.into_val(&env)],
            );
        }

        // Emit USERNAME_CLAIMED event
        events::emit_username_claimed(&env, &username_hash, &claimer);

        Ok(())
    }

    // Helper functions added for testing/setup
    pub fn init_for_test(env: Env, factory: Address, highest_bidder: Address, status: types::AuctionStatus) {
        storage::set_factory_contract(&env, &factory);
        storage::set_highest_bidder(&env, &highest_bidder);
        storage::set_status(&env, status);
    }
}
