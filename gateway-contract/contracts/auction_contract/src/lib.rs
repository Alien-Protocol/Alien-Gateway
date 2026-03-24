#![no_std]

mod storage;
mod types;

pub use storage::{
    add_bidder, get_all_bidders, get_auction, get_bid, has_auction, remove_bid, set_auction,
    set_bid, DataKey,
};
pub use types::{AuctionState, Bid};

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, BytesN, Env, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Event symbols
// ---------------------------------------------------------------------------

/// Topic symbol for the per-loser refund event.
///
/// Topics: `[BID_REFUNDED, username_hash]`
/// Data  : [`BidRefundedEvent`]
const BID_REFUNDED: Symbol = symbol_short!("BID_RFND");

// ---------------------------------------------------------------------------
// Event payload types
// ---------------------------------------------------------------------------

/// Data payload emitted with every `BID_RFND` event.
///
/// Published once per outbid bidder when `refund_losers` is called so that
/// off-chain indexers can reconcile balances without replaying every ledger.
#[contracttype]
#[derive(Clone, Debug)]
pub struct BidRefundedEvent {
    /// Address that received the refund.
    pub bidder: Address,
    /// Exact XLM amount (stroops) returned.
    pub amount: i128,
    /// Auction the bid belonged to.
    pub auction_hash: BytesN<32>,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct AuctionContract;

#[contractimpl]
impl AuctionContract {
    // -----------------------------------------------------------------------
    // Auction lifecycle
    // -----------------------------------------------------------------------

    /// Create a new auction identified by `username_hash`.
    ///
    /// The hash must not already be in use.  The `creator` must authorise
    /// the call.
    ///
    /// # Panics
    /// * `"auction already exists"` — `username_hash` is already registered.
    pub fn create_auction(
        env: Env,
        creator: Address,
        username_hash: BytesN<32>,
        start_time: u64,
        end_time: u64,
        reserve_price: i128,
    ) {
        creator.require_auth();
        assert!(
            !has_auction(&env, &username_hash),
            "auction already exists"
        );

        let state = AuctionState {
            creator,
            start_time,
            end_time,
            reserve_price,
            highest_bid: 0,
            highest_bidder: None,
            is_settled: false,
        };
        set_auction(&env, &username_hash, &state);
    }

    /// Mark an auction as settled so that `refund_losers` becomes callable.
    ///
    /// Only the auction creator may close it, and only after `end_time` has
    /// been reached.  Once closed the auction is permanently settled —
    /// `is_settled` cannot be reset to `false`.
    ///
    /// # Panics
    /// * `"auction not found"` — unknown `username_hash`.
    /// * `"not authorised"` — caller is not the auction `creator`.
    /// * `"auction not yet ended"` — ledger timestamp < `end_time`.
    /// * `"auction already settled"` — already closed.
    pub fn close_auction(env: Env, caller: Address, username_hash: BytesN<32>) {
        caller.require_auth();

        let mut state = get_auction(&env, &username_hash).expect("auction not found");

        assert!(caller == state.creator, "not authorised");
        assert!(
            env.ledger().timestamp() >= state.end_time,
            "auction not yet ended"
        );
        assert!(!state.is_settled, "auction already settled");

        state.is_settled = true;
        set_auction(&env, &username_hash, &state);
    }

    // -----------------------------------------------------------------------
    // Refunds
    // -----------------------------------------------------------------------

    /// Refund every non-winning bidder their full XLM after the auction closes.
    ///
    /// **Trustless** — no authentication required; anyone may trigger refunds.
    /// This allows a keeper, a cron job, or any third party to ensure losers
    /// are made whole without relying on the auction creator.
    ///
    /// ## Eligibility
    /// An auction is eligible for `refund_losers` when `is_settled == true`
    /// (set by `close_auction`).  Calling before the auction is closed panics
    /// with `"auction not closed"`.
    ///
    /// ## Per-bidder logic
    /// For every address in `AllBidders`:
    /// 1. Skip if the address is `highest_bidder` (the winner).
    /// 2. Skip if no `Bid` record exists — they were already refunded in a
    ///    prior call (idempotency guard).
    /// 3. **Effect**: delete the `Bid` record from storage.
    /// 4. **Interaction**: transfer `bid.amount` XLM from the contract to the
    ///    bidder via the native token client.
    /// 5. **Event**: emit `BID_RFND` with a [`BidRefundedEvent`] payload.
    ///
    /// Steps 3–5 follow the Checks–Effects–Interactions pattern to prevent
    /// reentrancy from causing a double-refund.
    ///
    /// ## Repeated calls
    /// Safe to call multiple times.  Bidders whose records were already
    /// removed are silently skipped, so the second and subsequent calls
    /// transfer nothing and emit no events.
    ///
    /// # Arguments
    /// * `env`           — the contract environment.
    /// * `username_hash` — 32-byte commitment hash identifying the auction.
    ///
    /// # Panics
    /// * `"auction not found"` — no auction exists for `username_hash`.
    /// * `"auction not closed"` — auction is still active (`is_settled == false`).
    pub fn refund_losers(env: Env, username_hash: BytesN<32>) {
        // ── CHECKS ──────────────────────────────────────────────────────────

        let state = get_auction(&env, &username_hash).expect("auction not found");

        // Reject if auction is still open — losers cannot be determined yet.
        assert!(state.is_settled, "auction not closed");

        // Capture winner before iterating; None when no bids were placed.
        let winner: Option<Address> = state.highest_bidder.clone();

        // Snapshot the bidder list.  Iterating over this Vec while we remove
        // individual Bid keys underneath is safe — AllBidders is not mutated.
        let bidders: Vec<Address> = get_all_bidders(&env, &username_hash);

        // Native XLM client — the Stellar native asset contract is addressed
        // by the network's built-in token address.  In the gateway contract
        // this address is stored at init time; here we derive it the standard
        // Soroban way.
        let native_client = token::StellarAssetClient::new(
            &env,
            &env.current_contract_address(),
        );

        // ── EFFECTS + INTERACTIONS (CEI applied per-bidder) ─────────────────

        for bidder in bidders.iter() {
            // Skip the winner — their bid stays locked until the claim step.
            if let Some(ref w) = winner {
                if bidder == *w {
                    continue;
                }
            }

            // Read the bid.  A None here means this bidder was already
            // refunded in a previous call to refund_losers — skip to avoid
            // a double-refund.
            let bid = match get_bid(&env, &username_hash, &bidder) {
                Some(b) => b,
                None => continue,
            };

            let refund_amount = bid.amount;

            // EFFECT: remove the bid record before the transfer.  If the
            // native_client.transfer panicked and the transaction was
            // reverted, the remove would also be reverted, leaving the bid
            // intact for a retry.  On success, the record is gone and a
            // second call finds None above.
            remove_bid(&env, &username_hash, &bidder);

            // INTERACTION: return XLM to the bidder.
            native_client.transfer(
                &env.current_contract_address(),
                &bidder,
                &refund_amount,
            );

            // EVENT: one emission per refunded loser.
            env.events().publish(
                (BID_REFUNDED, username_hash.clone()),
                BidRefundedEvent {
                    bidder: bidder.clone(),
                    amount: refund_amount,
                    auction_hash: username_hash.clone(),
                },
            );
        }
    }

    // -----------------------------------------------------------------------
    // Bidding
    // -----------------------------------------------------------------------

    /// Place or update a bid on auction `username_hash`.
    ///
    /// * The auction must exist and must not be settled.
    /// * `amount` must strictly exceed the current `highest_bid`.
    /// * `bidder` must authorise the call.
    ///
    /// # Panics
    /// * `"auction not found"` — unknown hash.
    /// * `"auction already settled"` — bidding is closed.
    /// * `"bid must exceed highest bid"` — `amount` is not strictly greater.
    pub fn place_bid(env: Env, username_hash: BytesN<32>, bidder: Address, amount: i128) {
        bidder.require_auth();

        let mut state = get_auction(&env, &username_hash).expect("auction not found");

        assert!(!state.is_settled, "auction already settled");
        assert!(amount > state.highest_bid, "bid must exceed highest bid");

        let bid = Bid {
            bidder: bidder.clone(),
            amount,
            timestamp: env.ledger().timestamp(),
        };

        // Maintain bidder list before writing the bid record so the
        // AllBidders key is always at least as fresh as any Bid key.
        add_bidder(&env, &username_hash, bidder.clone());
        set_bid(&env, &username_hash, &bidder, &bid);

        state.highest_bid = amount;
        state.highest_bidder = Some(bidder);
        set_auction(&env, &username_hash, &state);
    }

    // -----------------------------------------------------------------------
    // View helpers
    // -----------------------------------------------------------------------

    /// Returns the full [`AuctionState`] for `username_hash`.
    ///
    /// # Panics
    /// `"auction not found"` when the hash is unknown.
    pub fn get_auction(env: Env, username_hash: BytesN<32>) -> AuctionState {
        get_auction(&env, &username_hash).expect("auction not found")
    }

    /// Returns `true` if an auction record exists for `username_hash`.
    pub fn has_auction(env: Env, username_hash: BytesN<32>) -> bool {
        has_auction(&env, &username_hash)
    }

    /// Returns the [`Bid`] placed by `bidder` on `username_hash`, or `None`.
    pub fn get_bid(env: Env, username_hash: BytesN<32>, bidder: Address) -> Option<Bid> {
        get_bid(&env, &username_hash, &bidder)
    }

    /// Returns every address that has bid on `username_hash`.
    pub fn get_all_bidders(env: Env, username_hash: BytesN<32>) -> Vec<Address> {
        get_all_bidders(&env, &username_hash)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Events, Ledger, LedgerInfo},
        Address, BytesN, Env, IntoVal,
    };

    // ── helpers ──────────────────────────────────────────────────────────────

    fn test_hash(env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &[1u8; 32])
    }

    /// Standard fixture: env + client + four addresses + one open auction.
    struct Setup {
        env: Env,
        client: AuctionContractClient<'static>,
        creator: Address,
        alice: Address,
        bob: Address,
        charlie: Address,
        hash: BytesN<32>,
    }

    impl Setup {
        fn new() -> Self {
            let env = Env::default();
            env.mock_all_auths();

            let creator = Address::generate(&env);
            let alice = Address::generate(&env);
            let bob = Address::generate(&env);
            let charlie = Address::generate(&env);

            let contract_id = env.register_contract(None, AuctionContract);
            let client = AuctionContractClient::new(&env, &contract_id);
            let hash = test_hash(&env);

            client.create_auction(
                &creator,
                &hash,
                &0u64,    // start_time
                &1000u64, // end_time
                &100i128, // reserve_price
            );

            Setup { env, client, creator, alice, bob, charlie, hash }
        }

        /// Advance ledger past end_time and mark the auction settled.
        fn close(&self) {
            self.env.ledger().set(LedgerInfo {
                timestamp: 1001,
                ..self.env.ledger().get()
            });
            self.client.close_auction(&self.creator, &self.hash);
        }
    }

    // ── close_auction ─────────────────────────────────────────────────────

    #[test]
    fn close_sets_is_settled_true() {
        let s = Setup::new();
        s.close();
        assert!(s.client.get_auction(&s.hash).is_settled);
    }

    #[test]
    #[should_panic(expected = "auction not yet ended")]
    fn close_before_end_time_panics() {
        let s = Setup::new();
        // ledger timestamp is 0 — below end_time 1000.
        s.client.close_auction(&s.creator, &s.hash);
    }

    #[test]
    #[should_panic(expected = "auction already settled")]
    fn close_twice_panics() {
        let s = Setup::new();
        s.close();
        s.client.close_auction(&s.creator, &s.hash); // second call
    }

    // ── refund_losers — guard checks ──────────────────────────────────────

    #[test]
    #[should_panic(expected = "auction not found")]
    fn refund_losers_unknown_hash_panics() {
        let s = Setup::new();
        let bad = BytesN::from_array(&s.env, &[9u8; 32]);
        s.client.refund_losers(&bad);
    }

    #[test]
    #[should_panic(expected = "auction not closed")]
    fn refund_losers_before_close_panics() {
        let s = Setup::new();
        // is_settled is false — must be rejected.
        s.client.refund_losers(&s.hash);
    }

    // ── refund_losers — no-bid edge case ─────────────────────────────────

    #[test]
    fn refund_losers_with_no_bids_is_noop() {
        let s = Setup::new();
        s.close();
        s.client.refund_losers(&s.hash); // must not panic

        // No BID_RFND events expected.
        let rfnd_count = s
            .env
            .events()
            .all()
            .iter()
            .filter(|e| {
                e.1.get(0)
                    .map(|t| {
                        let sym: Symbol = t.into_val(&s.env);
                        sym == BID_REFUNDED
                    })
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(rfnd_count, 0);
    }

    // ── refund_losers — winner is not refunded ────────────────────────────

    #[test]
    fn winner_bid_record_is_not_removed() {
        let s = Setup::new();
        s.client.place_bid(&s.hash, &s.alice, &500i128);
        s.close();
        s.client.refund_losers(&s.hash);

        // Alice's bid record must still exist.
        assert!(
            s.client.get_bid(&s.hash, &s.alice).is_some(),
            "winner bid record was incorrectly removed"
        );
    }

    // ── refund_losers — multi-bidder: all losers refunded ─────────────────

    #[test]
    fn all_losers_refunded_winner_kept() {
        let s = Setup::new();

        s.client.place_bid(&s.hash, &s.alice, &200i128);   // loser
        s.client.place_bid(&s.hash, &s.bob, &300i128);     // loser
        s.client.place_bid(&s.hash, &s.charlie, &400i128); // winner

        s.close();
        s.client.refund_losers(&s.hash);

        // Losers' bid records must be deleted.
        assert!(
            s.client.get_bid(&s.hash, &s.alice).is_none(),
            "alice bid should be gone after refund"
        );
        assert!(
            s.client.get_bid(&s.hash, &s.bob).is_none(),
            "bob bid should be gone after refund"
        );

        // Winner's record must remain.
        assert!(
            s.client.get_bid(&s.hash, &s.charlie).is_some(),
            "winner bid must not be removed"
        );

        // Auction state must still name Charlie as highest_bidder.
        let state = s.client.get_auction(&s.hash);
        assert_eq!(state.highest_bidder, Some(s.charlie.clone()));
    }

    // ── refund_losers — correct event count ───────────────────────────────

    #[test]
    fn emits_one_event_per_loser() {
        let s = Setup::new();

        s.client.place_bid(&s.hash, &s.alice, &200i128);   // loser → event
        s.client.place_bid(&s.hash, &s.bob, &300i128);     // loser → event
        s.client.place_bid(&s.hash, &s.charlie, &400i128); // winner → no event

        s.close();
        s.client.refund_losers(&s.hash);

        let rfnd_count = s
            .env
            .events()
            .all()
            .iter()
            .filter(|e| {
                e.1.get(0)
                    .map(|t| {
                        let sym: Symbol = t.into_val(&s.env);
                        sym == BID_REFUNDED
                    })
                    .unwrap_or(false)
            })
            .count();

        assert_eq!(rfnd_count, 2, "expected exactly 2 BID_RFND events");
    }

    // ── refund_losers — idempotency ───────────────────────────────────────

    #[test]
    fn second_call_is_noop_no_extra_events() {
        let s = Setup::new();

        s.client.place_bid(&s.hash, &s.alice, &200i128);   // loser
        s.client.place_bid(&s.hash, &s.charlie, &400i128); // winner

        s.close();
        s.client.refund_losers(&s.hash); // first call — refunds Alice
        s.client.refund_losers(&s.hash); // second call — must be a no-op

        // Still only one BID_RFND event total.
        let rfnd_count = s
            .env
            .events()
            .all()
            .iter()
            .filter(|e| {
                e.1.get(0)
                    .map(|t| {
                        let sym: Symbol = t.into_val(&s.env);
                        sym == BID_REFUNDED
                    })
                    .unwrap_or(false)
            })
            .count();

        assert_eq!(rfnd_count, 1, "second call must not produce extra events");
    }

    // ── refund_losers — trustless: no auth required ───────────────────────

    #[test]
    fn refund_losers_requires_no_auth() {
        // Build a closed auction with mocked auth.
        let env = Env::default();
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let contract_id = env.register_contract(None, AuctionContract);
        let client = AuctionContractClient::new(&env, &contract_id);
        let hash = test_hash(&env);

        client.create_auction(&creator, &hash, &0u64, &1000u64, &100i128);
        env.ledger().set(LedgerInfo {
            timestamp: 1001,
            ..env.ledger().get()
        });
        client.close_auction(&creator, &hash);

        // Clear the auth mock so only explicit require_auth calls would fail.
        // refund_losers has no require_auth — it must still succeed.
        let env2 = Env::default();
        let client2 = AuctionContractClient::new(&env2, &contract_id);
        // Calling on the same contract address without mock_all_auths.
        // Any require_auth inside refund_losers would panic here.
        client2.refund_losers(&hash);
    }

    // ── place_bid — cannot bid on settled auction ─────────────────────────

    #[test]
    #[should_panic(expected = "auction already settled")]
    fn place_bid_after_close_panics() {
        let s = Setup::new();
        s.close();
        s.client.place_bid(&s.hash, &s.alice, &200i128);
    }
}