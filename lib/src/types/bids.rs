use super::exit_tree::ExitLeafWithdrawal;
use super::utils::{add_to_hash_chain, get_key, get_price_hash};
use super::{ChainableSubmissions, Order, PlacedOrders, ValidatedOrders};
use crate::constants::MAX_BID_PRICE;
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
    /// Address of the ERC20 token to be borrowed (purchase token).
    pub purchase_token: Address,
    /// Address of the ERC20 token used as collateral.
    pub collateral_token: Address,
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
            purchase_token: bid_submission.purchaseToken,
            collateral_token: bid_submission.collateralToken,
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

    fn is_valid(&self) -> bool {
        // TODO: Bids must also consider that the collateral amount is sufficient
        self.is_revealed
    }

    fn to_exit_leaf(&self) -> ExitLeafWithdrawal {
        ExitLeafWithdrawal {
            recipient: self.bidder,
            token: self.collateral_token,
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

    fn sort_orders(&mut self) {
        self.sort_by(|a: &Bid, b: &Bid| b.bid_price_revealed.cmp(&a.bid_price_revealed));
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
        /// The address of the ERC20 purchase token
        address purchaseToken;
        /// The addresses of the collateral ERC20 token in the bid
        address collateralToken;
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
mod tests {
    use super::*;
    use crate::types::{
        exit_tree::{ExitLeaf, ExitLeaves},
        utils::test::calculate_expected_hash_chain_output,
    };
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
        assert_eq!(bid.purchase_token, bid_submission.purchaseToken);
        assert_eq!(bid.collateral_token, bid_submission.collateralToken);
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
        let price: U256 = U256::from(rand::random::<u64>() % crate::constants::MAX_BID_PRICE);
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
        let price: U256 = U256::from(crate::constants::MAX_BID_PRICE + 1);
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
        let mut bid: Bid = random_revealed_bid();
        bid.is_revealed = true;
        assert!(bid.is_valid());

        bid.is_revealed = false;
        assert!(!bid.is_valid());
    }

    #[test]
    fn test_bid_to_exit_leaf() {
        let bid: Bid = random_revealed_bid();
        let exit_leaf = bid.to_exit_leaf();

        assert_eq!(exit_leaf.recipient, bid.bidder);
        assert_eq!(exit_leaf.token, bid.collateral_token);
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
                let price: U256 =
                    U256::from(rand::random::<u64>() % crate::constants::MAX_BID_PRICE);
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
    fn test_validate_bids() {
        let mut placed_bids: Bids = Bids::new();
        let mut exit_leaves: ExitLeaves = ExitLeaves::new();
        let revealed_bid: Bid = random_revealed_bid();
        // TODO: let undercollateralized_bid = random_undercollateralized_bid();
        let non_revealed_bid: Bid = random_non_revealed_bid();

        placed_bids.insert(
            get_key(&revealed_bid.bidder, &revealed_bid.id),
            revealed_bid.clone(),
        );
        placed_bids.insert(
            get_key(&non_revealed_bid.bidder, &non_revealed_bid.id),
            non_revealed_bid.clone(),
        );
        // TODO: placed_bids.insert(get_key(&undercollateralized_bid.bidder, &undercollateralized_bid.id), undercollateralized_bid.clone());

        let validated_bids = placed_bids.into_validated_orders(&mut exit_leaves);

        assert_eq!(validated_bids.len(), 1);
        assert_eq!(exit_leaves.len(), 1);
        // TODO: assert_eq!(exit_leaves.len(), 2);
        assert_eq!(validated_bids[0], revealed_bid);
        assert_eq!(
            exit_leaves[0],
            ExitLeaf::Withdrawal(non_revealed_bid.to_exit_leaf())
        );
        // TODO: assert_eq!(exit_leaves[1], ExitLeaf::Withdrawal(undercollateralized_bid.to_exit_leaf()));
    }

    #[test]
    fn test_validated_bids_sort_orders() {
        let mut bids: ValidatedBids = vec![
            random_revealed_bid(),
            random_revealed_bid(),
            random_revealed_bid(),
        ];
        bids.sort_orders();
        assert!(bids[0].bid_price_revealed >= bids[1].bid_price_revealed);
        assert!(bids[1].bid_price_revealed >= bids[2].bid_price_revealed);
    }

    // HELPER FUNCTIONS
    /// Creates a new BidSubmission with random values for testing purposes.
    fn random_bid_submission() -> BidSubmission {
        BidSubmission {
            bidder: Address::random(),
            id: U96::from(rand::random::<u64>()),
            bidPriceHash: B256::random(),
            amount: U256::from(rand::random::<u128>()),
            collateralAmount: U256::from(rand::random::<u128>()),
            purchaseToken: Address::random(),
            collateralToken: Address::random(),
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
            purchaseToken: Address::random(),
            collateralToken: Address::random(),
        }
    }

    /// Creates a random non-revealed Bid.
    pub fn random_non_revealed_bid() -> Bid {
        Bid {
            id: U96::from(rand::random::<u64>()),
            bidder: Address::random(),
            bid_price_hash: B256::random(),
            bid_price_revealed: U256::from(rand::random::<u64>() % crate::constants::MAX_BID_PRICE),
            amount: U256::from(rand::random::<u128>()),
            collateral_amount: U256::from(rand::random::<u128>()),
            purchase_token: Address::random(),
            collateral_token: Address::random(),
            is_rollover: false,
            rollover_pair_off_term_repo_servicer: Address::ZERO,
            is_revealed: false,
        }
    }

    /// Creates a random revealed Bid.
    pub fn random_revealed_bid() -> Bid {
        Bid {
            id: U96::from(rand::random::<u64>()),
            bidder: Address::random(),
            bid_price_hash: B256::random(),
            bid_price_revealed: U256::from(rand::random::<u64>() % crate::constants::MAX_BID_PRICE),
            amount: U256::from(rand::random::<u128>()),
            collateral_amount: U256::from(rand::random::<u128>()),
            purchase_token: Address::random(),
            collateral_token: Address::random(),
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
        assert_eq!(bid_expected.purchase_token, bid.purchase_token);
        assert_eq!(bid_expected.collateral_token, bid.collateral_token);
        assert_eq!(bid_expected.is_rollover, bid.is_rollover);
        assert_eq!(
            bid_expected.rollover_pair_off_term_repo_servicer,
            bid.rollover_pair_off_term_repo_servicer
        );
        assert_eq!(bid_expected.is_revealed, bid.is_revealed);
    }
}
