# Auction Contract Specification

The Auction contract implements two auction flows that coexist in the same contract:

- A singleton username auction flow keyed by instance storage (`close_auction`, `claim_username`).
- An ID-indexed auction flow keyed by persistent storage (`create_auction`, `place_bid`, `close_auction_by_id`, `claim`).

Both flows use Soroban auth (`require_auth`) and ledger timestamp checks to enforce access and timing constraints.

## Public Entry Points

### Function: `create_auction`

Creates a new auction identified by `id`.

#### Interface

```rust
pub fn create_auction(
		env: Env,
		id: u32,
		seller: Address,
		asset: Address,
		min_bid: i128,
		end_time: u64,
)
```

#### Authorization

- `seller.require_auth()` must succeed.

#### Requirements & Validation

- Auction ID must not already exist (`storage::auction_exists(&env, id) == false`).
- If ID already exists, function aborts with `AuctionError::AuctionNotOpen`.

#### State Transitions

- Writes `seller` to `AuctionKey::Seller(id)`.
- Writes bidding token `asset` to `AuctionKey::Asset(id)`.
- Writes `min_bid` to `AuctionKey::MinBid(id)`.
- Writes `end_time` to `AuctionKey::EndTime(id)`.
- Sets status to `AuctionStatus::Open` in `AuctionKey::Status(id)`.

#### Events Emitted

- None in current implementation.

#### Errors

- `AuctionError::AuctionNotOpen` when `id` already exists.
- Host auth failure if `seller` does not authorize.

#### Edge Cases

- **Duplicate auction ID**: explicitly rejected (panic with `AuctionNotOpen`).
- **No min/end validation**: contract currently does not enforce `min_bid > 0` or `end_time > now` at creation.

### Function: `place_bid`

Places a bid on an existing auction and refunds the previously highest bidder.

#### Interface

```rust
pub fn place_bid(env: Env, id: u32, bidder: Address, amount: i128)
```

#### Authorization

- `bidder.require_auth()` must succeed.

#### Requirements & Validation

- Auction must still be open by time: `env.ledger().timestamp() < auction_end_time`.
- Bid must satisfy both:
	- `amount >= min_bid`
	- `amount > highest_bid`

If timing check fails, function aborts with `AuctionError::AuctionNotOpen`.
If bid floor/outbid check fails, function aborts with `AuctionError::BidTooLow`.

#### State Transitions

1. Transfers `amount` of auction asset token from `bidder` to contract.
2. If previous highest bidder exists, transfers prior `highest_bid` from contract back to that bidder.
3. Updates `AuctionKey::HighestBidder(id)` to current `bidder`.
4. Updates `AuctionKey::HighestBid(id)` to `amount`.

#### Events Emitted

- None in current implementation.

#### Errors

- `AuctionError::AuctionNotOpen` when auction time window is closed.
- `AuctionError::BidTooLow` when bid is below min or not strictly above current highest.
- Host auth failure if `bidder` does not authorize.
- Token transfer failure if token contract transfer preconditions are not met.

#### Edge Cases

- **Zero-bid history**: first bid is accepted if it meets `min_bid` and auction is still open.
- **Equal-to-highest bid**: rejected (`amount <= highest_bid` path).
- **Late bid at exact end timestamp**: rejected because condition is `timestamp >= end_time`.

### Function: `close_auction_by_id`

Closes an ID-indexed auction once its end time has passed.

#### Interface

```rust
pub fn close_auction_by_id(env: Env, id: u32)
```

#### Authorization

- No explicit caller auth in current implementation.

#### Requirements & Validation

- Current ledger timestamp must be at least the auction end time.
- If `timestamp < end_time`, function aborts with `AuctionError::AuctionNotClosed`.

#### State Transitions

- Sets `AuctionKey::Status(id)` to `AuctionStatus::Closed`.

#### Events Emitted

- None in current implementation.

#### Errors

- `AuctionError::AuctionNotClosed` when called before end time.

#### Edge Cases

- **Early close attempt**: rejected with `AuctionNotClosed`.
- **No bids placed**: still closes successfully; later claim semantics determine payout/ownership behavior.

### Function: `close_auction`

Closes the singleton username auction flow and emits closure metadata.

#### Interface

```rust
pub fn close_auction(
		env: Env,
		username_hash: BytesN<32>,
) -> Result<(), AuctionError>
```

#### Authorization

- No explicit caller auth in current implementation.

#### Requirements & Validation

- Current instance `status` must be `AuctionStatus::Open`.
- Current ledger timestamp must be at least instance `end_time`.

Returns:

- `Err(AuctionError::AuctionNotOpen)` if status is not `Open`.
- `Err(AuctionError::AuctionNotClosed)` if called before end time.

#### State Transitions

- Sets instance `DataKey::Status` to `AuctionStatus::Closed`.
- Reads instance `DataKey::HighestBidder` and `DataKey::HighestBid` for event payload.

#### Events Emitted

- Emits `AuctionClosedEvent` via `emit_auction_closed` with:
	- `username_hash`
	- `winner: Option<Address>`
	- `winning_bid: u128`

#### Errors

- `AuctionError::AuctionNotOpen`
- `AuctionError::AuctionNotClosed`

#### Edge Cases

- **Zero bids**: event emits `winner = None`, `winning_bid = 0`.
- **Repeated close**: second close call fails with `AuctionNotOpen` because status is no longer `Open`.

### Function: `claim_username`

Allows winner of singleton username auction to deploy/claim the username via factory contract.

#### Interface

```rust
pub fn claim_username(
		env: Env,
		username_hash: BytesN<32>,
		claimer: Address,
) -> Result<(), AuctionError>
```

#### Authorization

- `claimer.require_auth()` must succeed.

#### Requirements & Validation

- Instance status must not already be `Claimed`.
- Instance status must be `Closed`.
- `claimer` must equal stored highest bidder.
- Factory contract address must exist in `DataKey::FactoryContract`.

Returns:

- `Err(AuctionError::AlreadyClaimed)` if already claimed.
- `Err(AuctionError::NotClosed)` if not closed.
- `Err(AuctionError::NotWinner)` if caller is not winner.
- `Err(AuctionError::NoFactoryContract)` if factory address is missing.

#### State Transitions

1. Sets instance `DataKey::Status` to `AuctionStatus::Claimed`.
2. Invokes factory contract method `deploy_username(username_hash, claimer)`.

#### Events Emitted

- Emits `UsernameClaimedEvent` via `emit_username_claimed` with:
	- `username_hash`
	- `claimer`

#### Errors

- `AuctionError::AlreadyClaimed`
- `AuctionError::NotClosed`
- `AuctionError::NotWinner`
- `AuctionError::NoFactoryContract`
- Host auth failure if `claimer` does not authorize.

#### Edge Cases

- **No bids**: no highest bidder exists, so claim fails with `NotWinner`.
- **Claim race**: first valid claim sets status to `Claimed`; subsequent claims fail with `AlreadyClaimed`.

### Function: `claim`

Finalizes an ID-indexed auction by allowing the winner to release funds to seller.

#### Interface

```rust
pub fn claim(env: Env, id: u32, claimant: Address)
```

#### Authorization

- `claimant.require_auth()` must succeed.

#### Requirements & Validation

- Auction status for `id` must be `AuctionStatus::Closed`.
- Auction must not already be claimed (`auction_is_claimed == false`).
- `claimant` must equal current highest bidder.

Function aborts with:

- `AuctionError::NotClosed` when status is not closed.
- `AuctionError::AlreadyClaimed` when already claimed.
- `AuctionError::NotWinner` when claimant is not highest bidder.

#### State Transitions

1. Reads token `asset`, `winning_bid`, and `seller` for auction `id`.
2. Transfers `winning_bid` from contract to `seller`.
3. Marks `AuctionKey::Claimed(id)` as `true`.

#### Events Emitted

- None in current implementation.

#### Errors

- `AuctionError::NotClosed`
- `AuctionError::AlreadyClaimed`
- `AuctionError::NotWinner`
- Host auth failure if `claimant` does not authorize.
- Token transfer failure if transfer cannot be completed.

#### Edge Cases

- **No bids**: highest bidder is `None`; all claim attempts fail with `NotWinner`.
- **Double claim**: second successful claimant attempt is blocked by `AlreadyClaimed`.

## Error Variants (Contract-Wide)

Defined in `errors.rs`:

- `NotWinner`
- `AlreadyClaimed`
- `NotClosed`
- `NoFactoryContract`
- `Unauthorized`
- `InvalidState`
- `BidTooLow`
- `AuctionNotOpen`
- `AuctionNotClosed`

Note: `Unauthorized` and `InvalidState` are defined but not currently emitted by the public entry points above; authorization failures are enforced primarily through host-level `require_auth`.

## Event Types (Contract-Wide)

Defined in `events.rs`:

- `AuctionCreatedEvent`
- `BidPlacedEvent`
- `AuctionClosedEvent`
- `UsernameClaimedEvent`
- `BidRefundedEvent`

Current emission in public entry points:

- `close_auction` emits `AuctionClosedEvent`.
- `claim_username` emits `UsernameClaimedEvent`.
- Other listed entry points currently emit no events.

