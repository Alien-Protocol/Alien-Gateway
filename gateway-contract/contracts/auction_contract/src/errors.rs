use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Error {
    AlreadyExists = 1,
    BidTooLow = 2,
    AuctionClosed = 3,
    AuctionNotEnded = 4,
    AuctionNotClosed = 5,
    NotWinner = 6,
    AlreadyClaimed = 7,
}
