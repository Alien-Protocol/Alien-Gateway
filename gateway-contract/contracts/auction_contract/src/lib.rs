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
        let factory = storage::get_factory_contract(&env);
        if factory.is_none() {
            return Err(crate::errors::Error::NoFactoryContract);
        }

        env.invoke_contract::<()>(
            &factory.unwrap(),
            &Symbol::new(&env, "deploy_username"),
            vec![&env, username_hash.into_val(&env), claimer.into_val(&env)],
        );

        // Emit USERNAME_CLAIMED event
        events::emit_username_claimed(&env, &username_hash, &claimer);

        Ok(())
    }
}
