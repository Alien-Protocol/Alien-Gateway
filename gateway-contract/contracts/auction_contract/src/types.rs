use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuctionStatus {
    Open,
    Closed,
    Claimed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Status,
    HighestBidder,
    FactoryContract,
    EndTime,
    HighestBid,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuctionKey {
    Seller(u32),
    Asset(u32),
    MinBid(u32),
    EndTime(u32),
    HighestBidder(u32),
    HighestBid(u32),
    Status(u32),
    Claimed(u32),
}
