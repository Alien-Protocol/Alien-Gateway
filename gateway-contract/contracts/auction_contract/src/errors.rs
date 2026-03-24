use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotWinner = 1,
    AlreadyClaimed = 2,
    NotClosed = 3,
    NoFactoryContract = 4,
}
