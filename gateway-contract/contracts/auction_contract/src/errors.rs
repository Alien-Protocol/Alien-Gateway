use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AuctionError {
    NotWinner = 1,
    AlreadyClaimed = 2,
    NotClosed = 3,
    NoFactoryContract = 4,
    Unauthorized = 5,
    InvalidState = 6,
    BidTooLow = 7,
    AuctionNotOpen = 8,
    AuctionNotClosed = 9,
    SelfBid = 10,
}
