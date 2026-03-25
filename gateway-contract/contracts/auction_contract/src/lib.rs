#![no_std]
use soroban_sdk::{contract, contractimpl, vec, Address, BytesN, Env, IntoVal, Symbol};

pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

#[contract]
pub struct AuctionContract;

#[contractimpl]
impl AuctionContract {
    pub fn close_auction(env: Env, username_hash: BytesN<32>) -> Result<(), crate::errors::Error> {
        let status = storage::get_status(&env);

        // Reject if status is not Open
        if status != types::AuctionStatus::Open {
            return Err(crate::errors::Error::NotOpen);
        }

        // Get current ledger timestamp and end time
        let current_time = env.ledger().timestamp();
        let end_time = storage::get_end_time(&env);

        // Reject if timestamp < end_time
        if current_time < end_time {
            return Err(crate::errors::Error::AuctionNotClosed);
        }

        // Set status to Closed
        storage::set_status(&env, types::AuctionStatus::Closed);

        // Get winner and winning bid
        let winner = storage::get_highest_bidder(&env);
        let winning_bid = storage::get_highest_bid(&env);

        // Emit AUCTION_CLOSED event with winner and winning bid
        events::emit_auction_closed(&env, &username_hash, winner.clone(), winning_bid);

        Ok(())
    }

    pub fn claim_username(
        env: Env,
        username_hash: BytesN<32>,
        claimer: Address,
    ) -> Result<(), crate::errors::Error> {
        claimer.require_auth();

        let status = storage::get_status(&env);

        if status == types::AuctionStatus::Claimed {
            return Err(crate::errors::Error::AlreadyClaimed);
        }

        if status != types::AuctionStatus::Closed {
            return Err(crate::errors::Error::NotClosed);
        }

        let highest_bidder = storage::get_highest_bidder(&env);
        if !highest_bidder.map(|h| h == claimer).unwrap_or(false) {
            return Err(crate::errors::Error::NotWinner);
        }

        // Set status to Claimed
        storage::set_status(&env, types::AuctionStatus::Claimed);

        // Call factory_contract.deploy_username(username_hash, claimer)
        let factory = storage::get_factory_contract(&env);
        if factory.is_none() {
            return Err(crate::errors::Error::NoFactoryContract);
        }

        let factory_addr = factory.ok_or(crate::errors::Error::NoFactoryContract)?;
        env.invoke_contract::<()>(
            &factory_addr,
            &Symbol::new(&env, "deploy_username"),
            vec![&env, username_hash.into_val(&env), claimer.into_val(&env)],
        );

        // Emit USERNAME_CLAIMED event
        events::emit_username_claimed(&env, &username_hash, &claimer);

        Ok(())
    }
}
