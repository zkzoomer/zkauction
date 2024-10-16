use super::{ChainableSubmissions, Order, PlacedOrders, ValidatedOrders};
use crate::{
    allocations::bidder_allocations::BidderAllocation,
    auction_parameters::AuctionParameters,
    constants::{BPS, INITIAL_COLLATERAL_RATIO, MAX_BID_PRICE},
    exit_tree::ExitLeafTokenWithdrawal,
    utils::{add_to_hash_chain, get_key, get_price_hash},
};
use alloy_primitives::{aliases::U96, Address, B256, U256};
use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents a bid to borrow an amount of money for a specific interest rate backed by collateral.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Bid {
    /// Unique identifier for the bid, combined with `bidder` to form a complete key.
    pub id: U96,
    /// Ethereum address of the bidder.
    pub bidder: Address,
    /// Keccak-256 hash of the bid price and a nonce, enabling the blind auction process.
    pub bid_price_hash: B256,
    /// The actual bid price revealed during the reveal phase, initially zero.
    pub bid_price_revealed: U256,
    /// Maximum amount of purchase tokens that can be borrowed.
    pub amount: U256,
    /// Amount of collateral tokens locked for this bid.
    pub collateral_amount: U256,
    /// Indicates whether this bid is part of a rollover process.
    pub is_rollover: bool,
    /// Address of the term repo servicer for rollover pair-offs, if applicable.
    pub rollover_pair_off_term_repo_servicer: Address,
    /// Indicates whether the bid has been revealed in the reveal phase.
    pub is_revealed: bool,
}

impl Order for Bid {
    type OrderSubmission = BidSubmission;
    type OrderReveal = BidReveal;

    fn from_order_submission(bid_submission: &BidSubmission) -> Self {
        Self {
            id: bid_submission.id,
            bidder: bid_submission.bidder,
            bid_price_hash: bid_submission.bidPriceHash,
            bid_price_revealed: U256::ZERO,
            amount: bid_submission.amount,
            collateral_amount: bid_submission.collateralAmount,
            is_rollover: false,
            rollover_pair_off_term_repo_servicer: Address::ZERO,
            is_revealed: false,
        }
    }

    fn update_from_order_submission(&mut self, bid_submission: &BidSubmission) {
        self.amount = bid_submission.amount;
        self.collateral_amount = bid_submission.collateralAmount;
        self.bid_price_hash = bid_submission.bidPriceHash;
    }

    fn update_from_order_reveal<F: Fn(&[u8]) -> B256>(
        &mut self,
        hash_function: &F,
        bid_reveal: &BidReveal,
    ) {
        if get_price_hash(hash_function, &bid_reveal.price, &bid_reveal.nonce)
            == self.bid_price_hash
            && bid_reveal.price <= U256::from(MAX_BID_PRICE)
        {
            self.bid_price_revealed = bid_reveal.price;
            self.is_revealed = true;
        }
    }

    fn is_valid(&self, tokens: &AuctionParameters) -> bool {
        // Calculate the value of collateral and purchase amount
        // If one operation overflows, the bid is invalid
        let (collateral_value, of1) = self
            .collateral_amount
            .overflowing_mul(tokens.collateralPrice);
        let (purchase_value, of2) = self.amount.overflowing_mul(tokens.purchasePrice);
        let (minimum_collateral_side, of3) =
            purchase_value.overflowing_mul(U256::from(INITIAL_COLLATERAL_RATIO));
        let (collateral_side, of4) = collateral_value.overflowing_mul(U256::from(BPS));

        self.is_revealed
            && collateral_side >= minimum_collateral_side
            && (!of1 && !of2 && !of3 && !of4)
    }

    fn to_exit_leaf(&self, tokens: &AuctionParameters) -> ExitLeafTokenWithdrawal {
        ExitLeafTokenWithdrawal {
            recipient: self.bidder,
            token: tokens.collateralToken,
            amount: self.collateral_amount,
        }
    }
}

/// A collection of all bids, indexed by their unique keys.
///
/// # Key
/// The key is a `B256` (32-byte) value, created by concatenating:
/// - The bidder's Ethereum address (20 bytes)
/// - The bid's unique ID (12 bytes)
///
/// # Value
/// The value is a `Bid` struct, containing all details of the bid.
pub type Bids = BTreeMap<B256, Bid>;

impl PlacedOrders for Bids {
    type OrderSubmission = BidSubmission;
    type Allocation = BidderAllocation;
    type Order = Bid;

    /// # Behavior
    ///
    /// - If the bid's collateral amount is zero, the bid is removed from the collection.
    /// - If a bid with the same key already exists, it is updated with the new submission details.
    /// - If no bid exists for the key, a new `Bid` instance is created and inserted.
    fn save_or_update_order(&mut self, order_submission: &BidSubmission) {
        let key: B256 = get_key(&order_submission.bidder, &order_submission.id);
        if order_submission.collateralAmount.is_zero() {
            // Assuming a zero collateral amount indicates a bid cancellation.
            self.remove(&key);
        } else {
            self.entry(key)
                .and_modify(|existing_bid: &mut Bid| {
                    existing_bid.update_from_order_submission(order_submission);
                })
                .or_insert_with(|| Bid::from_order_submission(order_submission));
        }
    }
}

/// A collection of all validated bids.
pub type ValidatedBids = Vec<Bid>;

impl ValidatedOrders for ValidatedBids {
    type Order = Bid;
    type Allocation = BidderAllocation;

    /// Inversely sorts bids from least competitive to most competitive, such that the last item in the list is the most competitive bid
    fn sort_orders(&mut self) {
        self.sort_by(|a: &Bid, b: &Bid| a.bid_price_revealed.cmp(&b.bid_price_revealed));
    }
}

sol! {
    /// A `BidSubmission` represents a bid submission to borrow an amount of money for a specific interest rate
    #[derive(Serialize, Deserialize)]
    struct BidSubmission {
        /// The address of the bidder
        address bidder;
        /// Defines, alongside the `bidder`, a unique identifier for the bid
        uint96 id;
        /// Hash of the offered price as a percentage of the initial loaned amount vs amount returned at maturity. This stores 9 decimal places
        bytes32 bidPriceHash;
        /// The maximum amount of purchase tokens that can be borrowed
        uint256 amount;
        /// The amount of collateral tokens that were locked onchain
        uint256 collateralAmount;
    }
}

/// Represents the history of all bid submissions made onchain.
pub type BidSubmissions = Vec<BidSubmission>;

impl ChainableSubmissions for BidSubmissions {
    type T = Bid;
    /// # Behavior
    ///
    /// - If a bid with the same key already exists, it updates the amount, collateral amount, and bid price hash.
    /// - If no bid exists for the key, it creates a new `Bid` instance with the provided details.
    fn hash_chain<F>(&self, hash_function: &F, start_value: B256, bids: &mut Bids) -> B256
    where
        F: Fn(&[u8]) -> B256,
    {
        self.iter()
            .fold(start_value, |acc: B256, bid_submission: &BidSubmission| {
                bids.save_or_update_order(bid_submission);
                add_to_hash_chain(hash_function, bid_submission, &acc)
            })
    }
}

sol! {
    /// A `BidReveal` represents the bid reveal process that was carried out onchain
    #[derive(Serialize, Deserialize)]
    struct BidReveal {
        /// The ID of the bid that was revealed
        uint256 orderId;
        /// The price of the bid that was revealed
        uint256 price;
        /// Nonce value that was used to generate the bid price hash
        uint256 nonce;
    }
}

/// Represents the history of all bid reveals made onchain.
pub type BidReveals = Vec<BidReveal>;

impl ChainableSubmissions for BidReveals {
    type T = Bid;
    /// # Behavior
    ///
    /// - If a bid with the matching `orderId` is found and the calculated price hash
    ///   matches the stored hash:
    ///   - Updates the `bid_price_revealed` with the revealed price.
    ///   - Sets `is_revealed` to `true`.
    /// - If no matching bid is found or the price hash doesn't match, no changes are made.
    fn hash_chain<F>(&self, hash_function: &F, start_value: B256, bids: &mut Bids) -> B256
    where
        F: Fn(&[u8]) -> B256,
    {
        self.iter()
            .fold(start_value, |acc: B256, item: &BidReveal| {
                // Set bid price if bid exists and was revealed properly
                if let Some(bid) = bids.get_mut::<B256>(&item.orderId.into()) {
                    bid.update_from_order_reveal(hash_function, item);
                }
                add_to_hash_chain(hash_function, item, &acc)
            })
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        allocations::AuctionResults, auction_parameters::tests::random_auction_parameters,
        utils::test::calculate_expected_hash_chain_output,
    };

    use super::*;
    use alloy_primitives::keccak256;

    #[test]
    fn test_bid_from_order_submission() {
        let bid_submission: BidSubmission = random_bid_submission();

        let bid: Bid = Bid::from_order_submission(&bid_submission);
        assert_eq!(bid.bidder, bid_submission.bidder);
        assert_eq!(bid.id, bid_submission.id);
        assert_eq!(bid.bid_price_hash, bid_submission.bidPriceHash);
        assert_eq!(bid.amount, bid_submission.amount);
        assert_eq!(bid.collateral_amount, bid_submission.collateralAmount);
    }

    #[test]
    fn test_bid_update_from_order_submission() {
        let bid_submission: BidSubmission = random_bid_submission();

        let mut bid = Bid::from_order_submission(&bid_submission);
        let new_order_submission: BidSubmission = random_bid_submission();

        bid.update_from_order_submission(&new_order_submission);
        assert_eq!(bid.amount, new_order_submission.amount);
        assert_eq!(bid.collateral_amount, new_order_submission.collateralAmount);
        assert_eq!(bid.bid_price_hash, new_order_submission.bidPriceHash);
    }

    #[test]
    fn test_bid_update_from_order_reveal() {
        // Valid reveal
        let price: U256 = U256::from(rand::random::<u32>() % MAX_BID_PRICE);
        let nonce: U256 = U256::from(rand::random::<u128>());
        let bid_submission: BidSubmission = valid_random_bid_submission(&price, &nonce);
        let mut bid: Bid = Bid::from_order_submission(&bid_submission);
        bid.update_from_order_reveal(
            &|x| keccak256(x),
            &BidReveal {
                orderId: get_key(&bid_submission.bidder, &bid_submission.id).into(),
                price,
                nonce,
            },
        );
        assert_eq!(bid.bid_price_revealed, price);
        assert!(bid.is_revealed);

        // Invalid reveal
        let mut bid: Bid = Bid::from_order_submission(&bid_submission);
        bid.update_from_order_reveal(
            &|x: &[u8]| keccak256(x),
            &BidReveal {
                orderId: get_key(&bid_submission.bidder, &bid_submission.id).into(),
                price: U256::from(rand::random::<u128>()),
                nonce: U256::from(rand::random::<u128>()),
            },
        );
        assert_eq!(bid.bid_price_revealed, U256::ZERO);
        assert!(!bid.is_revealed);

        // Valid reveal with out of bounds price
        let price: U256 = U256::from(MAX_BID_PRICE + 1);
        let nonce: U256 = U256::from(rand::random::<u128>());
        let bid_submission: BidSubmission = valid_random_bid_submission(&price, &nonce);
        let mut bid: Bid = Bid::from_order_submission(&bid_submission);
        bid.update_from_order_reveal(
            &|x: &[u8]| keccak256(x),
            &BidReveal {
                orderId: get_key(&bid_submission.bidder, &bid_submission.id).into(),
                price,
                nonce,
            },
        );
        assert_eq!(bid.bid_price_revealed, U256::ZERO);
        assert!(!bid.is_revealed);
    }

    #[test]
    fn test_bid_is_valid() {
        let tokens: AuctionParameters = random_auction_parameters();

        let revealed_bid: Bid =
            random_collateralized_revealed_bid(&tokens.purchasePrice, &tokens.collateralPrice);
        assert!(revealed_bid.is_valid(&tokens));

        let non_revealed_bid: Bid =
            random_collateralized_non_revealed_bid(&tokens.purchasePrice, &tokens.collateralPrice);
        assert!(!non_revealed_bid.is_valid(&tokens));

        let undercollateralized_bid: Bid =
            random_undercollateralized_bid(&tokens.purchasePrice, &tokens.collateralPrice);
        assert!(!undercollateralized_bid.is_valid(&tokens));
    }

    #[test]
    fn test_bid_to_exit_leaf() {
        let bid: Bid = random_revealed_bid();
        let tokens: AuctionParameters = random_auction_parameters();
        let exit_leaf: ExitLeafTokenWithdrawal = bid.to_exit_leaf(&tokens);

        assert_eq!(exit_leaf.recipient, bid.bidder);
        assert_eq!(exit_leaf.token, tokens.collateralToken);
        assert_eq!(exit_leaf.amount, bid.collateral_amount);
    }

    #[test]
    fn test_save_or_update_bid() {
        let mut bids: Bids = Bids::new();
        let mut bid_submission: BidSubmission = random_bid_submission();

        // Saves the bid if new
        bids.save_or_update_order(&bid_submission);

        let bid: Bid = Bid::from_order_submission(&bid_submission);
        assert_eq!(bids.len(), 1);
        bid_eq(
            &bid,
            bids.get(&get_key(&bid_submission.bidder, &bid_submission.id))
                .unwrap(),
        );

        // Updates the bid if it already exists
        bid_submission.bidPriceHash = B256::random();
        bid_submission.amount = U256::from(rand::random::<u128>());
        bid_submission.collateralAmount = U256::from(rand::random::<u128>());
        bids.save_or_update_order(&bid_submission);

        let bid: Bid = Bid::from_order_submission(&bid_submission);
        assert_eq!(bids.len(), 1);
        bid_eq(
            &bid,
            bids.get(&get_key(&bid_submission.bidder, &bid_submission.id))
                .unwrap(),
        );

        // Deletes the bid if collateral amount is zero
        bid_submission.collateralAmount = U256::ZERO;
        bids.save_or_update_order(&bid_submission);
        assert_eq!(bids.len(), 0);
    }

    #[test]
    fn test_order_submissions_hash_chain() {
        // Random values
        let start_value: B256 = B256::ZERO;
        let mut expected_bids: Bids = Bids::new();
        let bid_submissions: BidSubmissions = (0..42)
            .map(|_| {
                let bid_submission: BidSubmission = random_bid_submission();
                expected_bids.save_or_update_order(&bid_submission);
                bid_submission
            })
            .collect();
        let expected_output: B256 =
            calculate_expected_hash_chain_output(&start_value, &bid_submissions);

        let mut bids: Bids = Bids::new();
        let output: B256 =
            bid_submissions.hash_chain(&|x: &[u8]| keccak256(x), start_value, &mut bids);

        assert_eq!(expected_output, output);
        assert_eq!(expected_bids, bids);
    }

    #[test]
    fn test_order_reveals_hash_chain() {
        // Random values
        let start_value: B256 = B256::random();
        let mut expected_bids: Bids = Bids::new();
        let mut bid_reveals: BidReveals = BidReveals::new();
        let bid_submissions: BidSubmissions = (0..42)
            .map(|_| {
                let price: U256 = U256::from(rand::random::<u32>() % MAX_BID_PRICE);
                let nonce: U256 = U256::from(rand::random::<u128>());
                let bid_submission: BidSubmission = valid_random_bid_submission(&price, &nonce);
                expected_bids.save_or_update_order(&bid_submission);
                bid_reveals.push(BidReveal {
                    orderId: get_key(&bid_submission.bidder, &bid_submission.id).into(),
                    price,
                    nonce,
                });
                bid_submission
            })
            .collect();
        bid_reveals.iter().for_each(|bid_reveal: &BidReveal| {
            if let Some(bid) = expected_bids.get_mut::<B256>(&bid_reveal.orderId.into()) {
                bid.update_from_order_reveal(&|x: &[u8]| keccak256(x), bid_reveal);
            }
        });
        let mut expected_output: B256 =
            calculate_expected_hash_chain_output(&start_value, &bid_submissions);
        expected_output = calculate_expected_hash_chain_output(&expected_output, &bid_reveals);

        let mut bids: Bids = Bids::new();
        let mut output: B256 =
            bid_submissions.hash_chain(&|x: &[u8]| keccak256(x), start_value, &mut bids);
        output = bid_reveals.hash_chain(&|x: &[u8]| keccak256(x), output, &mut bids);

        assert_eq!(expected_output, output);
        assert_eq!(expected_bids, bids);
    }

    #[test]
    fn test_validated_bids_sort_orders() {
        let mut bids: ValidatedBids = vec![
            random_revealed_bid(),
            random_revealed_bid(),
            random_revealed_bid(),
        ];
        bids.sort_orders();
        assert!(bids[0].bid_price_revealed <= bids[1].bid_price_revealed);
        assert!(bids[1].bid_price_revealed <= bids[2].bid_price_revealed);
    }

    #[test]
    fn test_validated_bids_unlock_outstanding_orders() {
        let prover_address: Address = Address::random();
        let mut auction_results: AuctionResults = AuctionResults::new(&prover_address);
        let validated_bids: ValidatedBids = ValidatedBids::from([random_revealed_bid()]);
        validated_bids.unlock_outstanding_orders(&mut auction_results.bidder_allocations);

        // Allocations get assigned
        assert_eq!(auction_results.bidder_allocations.len(), 1);
    }

    // HELPER FUNCTIONS
    /// Creates a new BidSubmission with random values for testing purposes.
    pub fn random_bid_submission() -> BidSubmission {
        BidSubmission {
            bidder: Address::random(),
            id: U96::from(rand::random::<u64>()),
            bidPriceHash: B256::random(),
            amount: U256::from(rand::random::<u128>()),
            collateralAmount: U256::from(rand::random::<u128>()),
        }
    }

    /// Creates a random BidSubmission with a valid bid price hash for the given price and nonce.
    fn valid_random_bid_submission(price: &U256, nonce: &U256) -> BidSubmission {
        BidSubmission {
            bidder: Address::random(),
            id: U96::from(rand::random::<u64>()),
            bidPriceHash: get_price_hash(&|x| keccak256(x), price, nonce),
            amount: U256::from(rand::random::<u128>()),
            collateralAmount: U256::from(rand::random::<u128>()),
        }
    }

    /// Creates a random non-revealed Bid.
    pub fn random_collateralized_non_revealed_bid(
        purchase_price: &U256,
        collateral_price: &U256,
    ) -> Bid {
        let purchase_amount: U256 = U256::from(rand::random::<u128>());
        let minimum_collateral_amount: U256 =
            (purchase_amount * purchase_price * U256::from(INITIAL_COLLATERAL_RATIO))
                / (collateral_price * U256::from(BPS));
        Bid {
            id: U96::from(rand::random::<u64>()),
            bidder: Address::random(),
            bid_price_hash: B256::random(),
            bid_price_revealed: U256::from(rand::random::<u32>() % MAX_BID_PRICE),
            amount: purchase_amount,
            collateral_amount: minimum_collateral_amount
                .saturating_add(U256::from(rand::random::<u128>())),
            is_rollover: false,
            rollover_pair_off_term_repo_servicer: Address::ZERO,
            is_revealed: false,
        }
    }

    /// Creates a random undercollateralized Bid.
    pub fn random_undercollateralized_bid(purchase_price: &U256, collateral_price: &U256) -> Bid {
        let purchase_amount: U256 = U256::from(rand::random::<u64>());
        let purchase_value: U256 = purchase_amount * purchase_price;
        let minimum_collateral_side: U256 = purchase_value * U256::from(INITIAL_COLLATERAL_RATIO);

        let (mut underwater_collateral_amount, _): (U256, U256) =
            minimum_collateral_side.div_rem(collateral_price * U256::from(BPS));
        underwater_collateral_amount -= U256::from(1);

        Bid {
            id: U96::from(rand::random::<u64>()),
            bidder: Address::random(),
            bid_price_hash: B256::random(),
            bid_price_revealed: U256::from(rand::random::<u32>() % MAX_BID_PRICE),
            amount: purchase_amount,
            collateral_amount: underwater_collateral_amount,
            is_rollover: false,
            rollover_pair_off_term_repo_servicer: Address::ZERO,
            is_revealed: true,
        }
    }

    /// Creates a random revealed Bid.
    pub fn random_collateralized_revealed_bid(
        purchase_price: &U256,
        collateral_price: &U256,
    ) -> Bid {
        let amount: U256 = U256::from(rand::random::<u64>());
        let purchase_value: U256 = amount * purchase_price;
        let minimum_collateral_side: U256 = purchase_value * U256::from(INITIAL_COLLATERAL_RATIO);
        let minimum_collateral_amount: U256 =
            minimum_collateral_side.div_ceil(collateral_price * U256::from(BPS));

        Bid {
            id: U96::from(rand::random::<u64>()),
            bidder: Address::random(),
            bid_price_hash: B256::random(),
            bid_price_revealed: U256::from(rand::random::<u32>() % MAX_BID_PRICE),
            amount,
            collateral_amount: minimum_collateral_amount
                .saturating_add(U256::from(rand::random::<u128>())),
            is_rollover: false,
            rollover_pair_off_term_repo_servicer: Address::ZERO,
            is_revealed: true,
        }
    }

    /// Creates a random revealed Bid.
    pub fn random_revealed_bid() -> Bid {
        Bid {
            id: U96::from(rand::random::<u64>()),
            bidder: Address::random(),
            bid_price_hash: B256::random(),
            bid_price_revealed: U256::from(rand::random::<u32>() % MAX_BID_PRICE),
            amount: U256::from(rand::random::<u128>()),
            collateral_amount: U256::from(rand::random::<u128>()),
            is_rollover: false,
            rollover_pair_off_term_repo_servicer: Address::ZERO,
            is_revealed: true,
        }
    }

    /// Compares two Bid structs for equality, asserting that all fields match.
    fn bid_eq(bid_expected: &Bid, bid: &Bid) {
        assert_eq!(bid_expected.bidder, bid.bidder);
        assert_eq!(bid_expected.id, bid.id);
        assert_eq!(bid_expected.bid_price_hash, bid.bid_price_hash);
        assert_eq!(bid_expected.bid_price_revealed, bid.bid_price_revealed);
        assert_eq!(bid_expected.amount, bid.amount);
        assert_eq!(bid_expected.collateral_amount, bid.collateral_amount);
        assert_eq!(bid_expected.is_rollover, bid.is_rollover);
        assert_eq!(
            bid_expected.rollover_pair_off_term_repo_servicer,
            bid.rollover_pair_off_term_repo_servicer
        );
        assert_eq!(bid_expected.is_revealed, bid.is_revealed);
    }
}
