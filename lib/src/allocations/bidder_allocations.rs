use alloy_primitives::{Address, U256};
use std::collections::BTreeMap;

use crate::{
    exit_tree::{ExitLeaf, ExitLeafRepurchaseObligation, ExitLeafTokenWithdrawal, ExitLeaves},
    orders::bids::Bid,
    tokens::Tokens,
};

use super::{Allocation, Allocations};

#[derive(PartialEq, Debug)]
/// Represents a repurchase obligation for a bidder.
pub struct RepurchaseObligation {
    /// The amount to be repurchased.
    repurchase_amount: U256,
    /// The amount of collateral associated with the repurchase.
    collateral_amount: U256,
}

impl Default for RepurchaseObligation {
    /// Creates a default `RepurchaseObligation` with zero amounts.
    fn default() -> Self {
        Self {
            repurchase_amount: U256::ZERO,
            collateral_amount: U256::ZERO,
        }
    }
}

/// Represents the allocation for a bidder in the auction.
pub struct BidderAllocation {
    /// The amount of purchase tokens assigned to the bidder, if any.
    purchase_amount: U256,
    /// The amount of collateral tokens left on the table for the bidder, if any.
    collateral_amount: U256,
    /// The bidder's repurchase obligation, if any.
    repurchase_obligation: RepurchaseObligation,
}

impl Default for BidderAllocation {
    /// Creates a default `BidderAllocation` with zero amounts.
    fn default() -> Self {
        Self {
            purchase_amount: U256::ZERO,
            collateral_amount: U256::ZERO,
            repurchase_obligation: RepurchaseObligation {
                repurchase_amount: U256::ZERO,
                collateral_amount: U256::ZERO,
            },
        }
    }
}

impl BidderAllocation {
    /// Updates the purchase token amount for the bidder.
    ///
    /// # Arguments
    ///
    /// * `self` - The allocation to be updated.
    /// * `amount` - The amount to add to the current purchase amount.
    pub fn update_purchase_amount(&mut self, amount: U256) {
        self.purchase_amount = self.purchase_amount.saturating_add(amount);
    }

    /// Updates the collateral amount for the bidder.
    ///
    /// # Arguments
    ///
    /// * `self` - The allocation to be updated.
    /// * `amount` - The amount to add to the current collateral amount.
    pub fn update_collateral_amount(&mut self, amount: U256) {
        self.collateral_amount = self.collateral_amount.saturating_add(amount);
    }

    /// Updates the repurchase obligation for the bidder.
    ///
    /// # Arguments
    ///
    /// * `self` - The allocation to be updated.
    /// * `repurchase_amount` - The amount to add to the current repurchase amount.
    /// * `collateral_amount` - The amount to add to the current collateral amount associated with the repurchase obligation.
    pub fn update_repurchase_obligation(
        &mut self,
        repurchase_amount: U256,
        collateral_amount: U256,
    ) {
        self.repurchase_obligation.repurchase_amount = self
            .repurchase_obligation
            .repurchase_amount
            .saturating_add(repurchase_amount);
        self.repurchase_obligation.collateral_amount = self
            .repurchase_obligation
            .collateral_amount
            .saturating_add(collateral_amount);
    }
}

impl Allocation for BidderAllocation {
    fn into_exit_leaves(self, address: Address, tokens: &Tokens, exit_leaves: &mut ExitLeaves) {
        if self.purchase_amount != U256::ZERO {
            exit_leaves.push(ExitLeaf::TokenWithdrawal(ExitLeafTokenWithdrawal {
                recipient: address,
                token: tokens.purchaseToken,
                amount: self.purchase_amount,
            }));
        }

        if self.collateral_amount != U256::ZERO {
            exit_leaves.push(ExitLeaf::TokenWithdrawal(ExitLeafTokenWithdrawal {
                recipient: address,
                token: tokens.collateralToken,
                amount: self.collateral_amount,
            }));
        }

        if self.repurchase_obligation != RepurchaseObligation::default() {
            exit_leaves.push(ExitLeaf::RepurchaseObligation(
                ExitLeafRepurchaseObligation {
                    debtor: address,
                    repurchaseAmount: self.repurchase_obligation.repurchase_amount,
                    collateralAmount: self.repurchase_obligation.collateral_amount,
                },
            ));
        }
    }
}

/// A map of bidder addresses to their respective allocations.
pub type BidderAllocations = BTreeMap<Address, BidderAllocation>;

impl Allocations for BidderAllocations {
    type Allocation = BidderAllocation;
    type Order = Bid;

    fn add_from_order(&mut self, order: &Self::Order) {
        let bidder_allocation: &mut BidderAllocation = self.get_allocation(&order.bidder);
        bidder_allocation.update_collateral_amount(order.collateral_amount);
    }

    fn get_allocation(&mut self, address: &Address) -> &mut Self::Allocation {
        self.entry(*address).or_default()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        allocations::AuctionResults,
        orders::{
            bids::{
                tests::{
                    random_bid_submission, random_collateralized_non_revealed_bid,
                    random_collateralized_revealed_bid, random_undercollateralized_bid,
                },
                Bids, ValidatedBids,
            },
            Order, PlacedOrders,
        },
        utils::get_key,
    };

    use super::*;
    use alloy_primitives::U256;

    #[test]
    fn test_update_purchase_amount_bidder() {
        let mut bidder_allocation: BidderAllocation = BidderAllocation::default();
        assert_eq!(bidder_allocation.purchase_amount, U256::ZERO);

        let increase_amount: U256 = U256::from(rand::random::<u64>());
        bidder_allocation.update_purchase_amount(increase_amount);
        assert_eq!(bidder_allocation.purchase_amount, increase_amount);
    }

    #[test]
    fn test_update_collateral_amount() {
        let mut bidder_allocation: BidderAllocation = BidderAllocation::default();
        assert_eq!(bidder_allocation.collateral_amount, U256::ZERO);

        let increase_amount: U256 = U256::from(rand::random::<u64>());
        bidder_allocation.update_collateral_amount(increase_amount);
        assert_eq!(bidder_allocation.collateral_amount, increase_amount);
    }

    #[test]
    fn test_update_repurchase_obligation() {
        let mut bidder_allocation: BidderAllocation = BidderAllocation::default();
        assert_eq!(
            bidder_allocation.repurchase_obligation.repurchase_amount,
            U256::ZERO
        );
        assert_eq!(
            bidder_allocation.repurchase_obligation.collateral_amount,
            U256::ZERO
        );

        let repurchase_increase_amount: U256 = U256::from(rand::random::<u64>());
        let collateral_increase_amount: U256 = U256::from(rand::random::<u64>());
        bidder_allocation
            .update_repurchase_obligation(repurchase_increase_amount, collateral_increase_amount);
        assert_eq!(
            bidder_allocation.repurchase_obligation.repurchase_amount,
            repurchase_increase_amount
        );
        assert_eq!(
            bidder_allocation.repurchase_obligation.collateral_amount,
            collateral_increase_amount
        );
    }

    #[test]
    fn test_bidder_add_from_order() {
        let mut bidder_allocations: BidderAllocations = BidderAllocations::new();

        // Define two bids that originate from the same bidder
        let bid_a: Bid = Bid::from_order_submission(&random_bid_submission());
        let mut bid_b: Bid = Bid::from_order_submission(&random_bid_submission());
        bid_b.bidder = bid_a.bidder;

        // Defines an allocation from an order
        bidder_allocations.add_from_order(&bid_a);

        let allocation_a: &BidderAllocation = bidder_allocations.get(&bid_a.bidder).unwrap();
        assert_eq!(allocation_a.collateral_amount, bid_a.collateral_amount);
        assert_eq!(allocation_a.purchase_amount, U256::ZERO);
        assert_eq!(
            allocation_a.repurchase_obligation,
            RepurchaseObligation::default()
        );

        // Correspondingly updates the allocation from adding another order
        bidder_allocations.add_from_order(&bid_b);

        let allocation_b: &BidderAllocation = bidder_allocations.get(&bid_a.bidder).unwrap();
        assert_eq!(
            allocation_b.collateral_amount,
            bid_a.collateral_amount + bid_b.collateral_amount
        );
        assert_eq!(allocation_b.purchase_amount, U256::ZERO);
        assert_eq!(
            allocation_b.repurchase_obligation,
            RepurchaseObligation::default()
        );
    }

    #[test]
    fn test_bidder_get_allocation() {
        let mut auction_results: AuctionResults = AuctionResults::new(&Address::random());
        let bidder_address: Address = Address::random();

        // Get a new bidder allocation
        let bidder_allocation: &mut BidderAllocation = auction_results
            .bidder_allocations
            .get_allocation(&bidder_address);
        assert_eq!(bidder_allocation.purchase_amount, U256::ZERO);

        // Modify the allocation
        let update_amount: U256 = U256::from(rand::random::<u64>());
        bidder_allocation.update_purchase_amount(update_amount);

        // Get the same allocation and check if it's modified
        let same_allocation: &mut BidderAllocation = auction_results
            .bidder_allocations
            .get_allocation(&bidder_address);
        assert_eq!(same_allocation.purchase_amount, update_amount);

        // Check that a new address creates a new allocation
        let another_address: Address = Address::random();
        let another_allocation: &mut BidderAllocation = auction_results
            .bidder_allocations
            .get_allocation(&another_address);
        assert_eq!(another_allocation.purchase_amount, U256::ZERO);
    }

    #[test]
    fn test_validate_bids() {
        let tokens: Tokens = random_tokens();

        let mut bidder_allocations: BidderAllocations = BidderAllocations::new();
        let revealed_bid: Bid =
            random_collateralized_revealed_bid(&tokens.purchasePrice, &tokens.collateralPrice);
        let undercollateralized_bid: Bid =
            random_undercollateralized_bid(&tokens.purchasePrice, &tokens.collateralPrice);
        let non_revealed_bid: Bid =
            random_collateralized_non_revealed_bid(&tokens.purchasePrice, &tokens.collateralPrice);

        let placed_bids: Bids = Bids::from([
            (
                get_key(&revealed_bid.bidder, &revealed_bid.id),
                revealed_bid.clone(),
            ),
            (
                get_key(&non_revealed_bid.bidder, &non_revealed_bid.id),
                non_revealed_bid.clone(),
            ),
            (
                get_key(&undercollateralized_bid.bidder, &undercollateralized_bid.id),
                undercollateralized_bid.clone(),
            ),
        ]);

        let validated_bids: ValidatedBids =
            placed_bids.into_validated_orders(&tokens, &mut bidder_allocations);

        // Revealed bid
        assert_eq!(validated_bids.len(), 1);
        assert_eq!(validated_bids[0], revealed_bid);

        // Non revealed bid is added to allocations
        assert_eq!(
            bidder_allocations
                .get(&non_revealed_bid.bidder)
                .unwrap()
                .collateral_amount,
            non_revealed_bid.collateral_amount
        );
        assert_eq!(
            bidder_allocations
                .get(&non_revealed_bid.bidder)
                .unwrap()
                .purchase_amount,
            U256::ZERO
        );
        assert_eq!(
            bidder_allocations
                .get(&non_revealed_bid.bidder)
                .unwrap()
                .repurchase_obligation,
            RepurchaseObligation::default()
        );

        // Uncollateralized bid is added to allocations
        assert_eq!(
            bidder_allocations
                .get(&undercollateralized_bid.bidder)
                .unwrap()
                .collateral_amount,
            undercollateralized_bid.collateral_amount
        );
        assert_eq!(
            bidder_allocations
                .get(&undercollateralized_bid.bidder)
                .unwrap()
                .purchase_amount,
            U256::ZERO
        );
        assert_eq!(
            bidder_allocations
                .get(&undercollateralized_bid.bidder)
                .unwrap()
                .repurchase_obligation,
            RepurchaseObligation::default()
        );
    }

    #[test]
    fn test_bidder_into_exit_leaves() {
        let tokens: Tokens = random_tokens();

        // Empty bidder allocation pushes no new leaf
        let mut exit_leaves: ExitLeaves = ExitLeaves::new();
        let bidder_address: Address = Address::random();
        let bidder_allocation: BidderAllocation = BidderAllocation::default();
        bidder_allocation.into_exit_leaves(bidder_address, &tokens, &mut exit_leaves);
        assert_eq!(exit_leaves.len(), 0);

        // Proper bidder purchase allocation pushes corresponding leaf
        let mut exit_leaves: ExitLeaves = ExitLeaves::new();
        let bidder_address: Address = Address::random();
        let bidder_purchase_amount: U256 = U256::from(rand::random::<u128>());
        let mut bidder_allocation: BidderAllocation = BidderAllocation::default();
        bidder_allocation.update_purchase_amount(bidder_purchase_amount);
        bidder_allocation.into_exit_leaves(bidder_address, &tokens, &mut exit_leaves);
        assert_eq!(exit_leaves.len(), 1);
        assert_eq!(
            exit_leaves[0],
            ExitLeaf::TokenWithdrawal(ExitLeafTokenWithdrawal {
                recipient: bidder_address,
                token: tokens.purchaseToken,
                amount: bidder_purchase_amount,
            }),
        );

        // Proper bidder collateral allocation pushes corresponding leaf
        let mut exit_leaves: ExitLeaves = ExitLeaves::new();
        let bidder_address: Address = Address::random();
        let bidder_collateral_amount: U256 = U256::from(rand::random::<u128>());
        let mut bidder_allocation: BidderAllocation = BidderAllocation::default();
        bidder_allocation.update_collateral_amount(bidder_collateral_amount);
        bidder_allocation.into_exit_leaves(bidder_address, &tokens, &mut exit_leaves);
        assert_eq!(exit_leaves.len(), 1);
        assert_eq!(
            exit_leaves[0],
            ExitLeaf::TokenWithdrawal(ExitLeafTokenWithdrawal {
                recipient: bidder_address,
                token: tokens.collateralToken,
                amount: bidder_collateral_amount,
            }),
        );

        // Proper bidder repurchase obligation allocation pushes corresponding leaf
        let mut exit_leaves: ExitLeaves = ExitLeaves::new();
        let bidder_address: Address = Address::random();
        let bidder_purchase_amount: U256 = U256::from(rand::random::<u128>());
        let bidder_collateral_amount: U256 = U256::from(rand::random::<u128>());
        let mut bidder_allocation: BidderAllocation = BidderAllocation::default();
        bidder_allocation
            .update_repurchase_obligation(bidder_purchase_amount, bidder_collateral_amount);
        bidder_allocation.into_exit_leaves(bidder_address, &tokens, &mut exit_leaves);
        assert_eq!(exit_leaves.len(), 1);
        assert_eq!(
            exit_leaves[0],
            ExitLeaf::RepurchaseObligation(ExitLeafRepurchaseObligation {
                debtor: bidder_address,
                repurchaseAmount: bidder_purchase_amount,
                collateralAmount: bidder_collateral_amount,
            }),
        );
    }

    // TEST HELPER FUNCTIONS
    /// Creates a new set of random tokens.
    fn random_tokens() -> Tokens {
        Tokens {
            purchaseToken: Address::random(),
            purchasePrice: U256::from(rand::random::<u64>()),
            collateralToken: Address::random(),
            collateralPrice: U256::from(rand::random::<u64>()),
        }
    }
}
