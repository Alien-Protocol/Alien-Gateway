use soroban_sdk::{contracttype, Address};

#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum AuctionStatus {
    Open,
    Closed,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Auction {
    pub seller: Address,
    pub asset: Address,
    pub min_bid: i128,
    pub end_time: u64,
    pub highest_bidder: Option<Address>,
    pub highest_bid: i128,
    pub status: AuctionStatus,
}
