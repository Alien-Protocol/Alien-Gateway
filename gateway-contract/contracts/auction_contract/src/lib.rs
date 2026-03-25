#![no_std]
use soroban_sdk::{contract, contractimpl, vec, Address, BytesN, Env, IntoVal, Symbol};

pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

// Ensure event symbols are linked from the main contract entrypoint module.
use crate::events::{AUCTION_CLOSED, AUCTION_CREATED, BID_PLACED, BID_REFUNDED, USERNAME_CLAIMED};

#[allow(dead_code)]
fn _touch_event_symbols() {
    let _ = (
        AUCTION_CREATED,
        BID_PLACED,
        AUCTION_CLOSED,
        USERNAME_CLAIMED,
        BID_REFUNDED,
    );
}

#[cfg(test)]
mod test;

#[contract]
pub struct AuctionContract;

#[contractimpl]
impl AuctionContract {
    pub fn claim_username(
        env: Env,
        username_hash: BytesN<32>,
        claimer: Address,
    ) -> Result<(), crate::errors::AuctionError> {
        claimer.require_auth();

        let status = storage::get_status(&env);

        if status == types::AuctionStatus::Claimed {
            return Err(crate::errors::AuctionError::AlreadyClaimed);
        }

        if status != types::AuctionStatus::Closed {
            return Err(crate::errors::AuctionError::NotClosed);
        }

        let highest_bidder = storage::get_highest_bidder(&env);
        if !highest_bidder.map(|h| h == claimer).unwrap_or(false) {
            return Err(crate::errors::AuctionError::NotWinner);
        }

        // Set status to Claimed
        storage::set_status(&env, types::AuctionStatus::Claimed);

        // Call factory_contract.deploy_username(username_hash, claimer)
        let factory = storage::get_factory_contract(&env);
        if factory.is_none() {
            return Err(crate::errors::AuctionError::NoFactoryContract);
        }

        let factory_addr = factory.ok_or(crate::errors::AuctionError::NoFactoryContract)?;
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
