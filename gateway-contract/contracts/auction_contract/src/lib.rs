#![no_std]
use crate::errors::Error;
use crate::events::Events;
use crate::storage::Storage;
use crate::types::{Auction, AuctionStatus};
use soroban_sdk::{contract, contractimpl, panic_with_error, token, Address, Env};

#[contract]
pub struct AuctionContract;

#[contractimpl]
impl AuctionContract {
    pub fn create_auction(
        env: Env,
        id: u32,
        seller: Address,
        asset: Address,
        min_bid: i128,
        end_time: u64,
    ) {
        if Storage::has_auction(&env, id) {
            panic_with_error!(&env, Error::AlreadyExists);
        }
        let auction = Auction {
            seller,
            asset,
            min_bid,
            end_time,
            highest_bidder: None,
            highest_bid: 0,
            status: AuctionStatus::Open,
        };
        Storage::set_auction(&env, id, &auction);
    }

    pub fn place_bid(env: Env, id: u32, bidder: Address, amount: i128) {
        bidder.require_auth();
        let mut auction = Storage::get_auction(&env, id);

        if env.ledger().timestamp() >= auction.end_time {
            panic_with_error!(&env, Error::AuctionClosed);
        }
        if amount < auction.min_bid || amount <= auction.highest_bid {
            panic_with_error!(&env, Error::BidTooLow);
        }

        let token_client = token::Client::new(&env, &auction.asset);
        token_client.transfer(&bidder, env.current_contract_address(), &amount);

        if let Some(prev_bidder) = auction.highest_bidder {
            token_client.transfer(
                &env.current_contract_address(),
                &prev_bidder,
                &auction.highest_bid,
            );
        }

        auction.highest_bidder = Some(bidder.clone());
        auction.highest_bid = amount;
        Storage::set_auction(&env, id, &auction);
        Events::bid_placed(&env, id, bidder, amount);
    }

    pub fn close_auction(env: Env, id: u32) {
        let mut auction = Storage::get_auction(&env, id);
        if env.ledger().timestamp() < auction.end_time {
            panic_with_error!(&env, Error::AuctionNotEnded);
        }
        auction.status = AuctionStatus::Closed;
        Storage::set_auction(&env, id, &auction);
    }

    pub fn claim(env: Env, id: u32, claimant: Address) {
        claimant.require_auth();
        let mut auction = Storage::get_auction(&env, id);
        if auction.status != AuctionStatus::Closed {
            panic_with_error!(&env, Error::AuctionNotClosed);
        }
        if let Some(ref winner) = auction.highest_bidder {
            if winner == &claimant {
                if auction.highest_bid == -1 {
                    panic_with_error!(&env, Error::AlreadyClaimed);
                }
                auction.highest_bid = -1;
                Storage::set_auction(&env, id, &auction);
                Events::claimed(&env, id, claimant);
                return;
            }
        }
        panic_with_error!(&env, Error::NotWinner);
    }
}

mod errors;
mod events;
mod storage;
mod test;
mod types;
