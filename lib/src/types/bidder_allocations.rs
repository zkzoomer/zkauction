use super::exit_tree::{ExitLeafRepurchaseObligation, ExitLeafTokenWithdrawal, ExitLeaves};
use super::tokens::Tokens;
use super::{allocations::Allocation, exit_tree::ExitLeaf};
use alloy_primitives::{Address, U256};
use std::collections::BTreeMap;

#[derive(PartialEq)]
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

/// A map of bidder addresses to their respective allocations.
pub type BidderAllocations = BTreeMap<Address, BidderAllocation>;

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

#[cfg(test)]
mod test {
    use crate::types::allocations::Allocations;

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
    fn test_get_or_create_bidder_allocation() {
        let mut allocations = Allocations::new(&Address::random());
        let bidder_address = Address::random();

        // Get a new bidder allocation
        let bidder_allocation = allocations.get_or_create_bidder_allocation(&bidder_address);
        assert_eq!(bidder_allocation.purchase_amount, U256::ZERO);

        // Modify the allocation
        let update_amount: U256 = U256::from(rand::random::<u64>());
        bidder_allocation.update_purchase_amount(update_amount);

        // Get the same allocation and check if it's modified
        let same_allocation = allocations.get_or_create_bidder_allocation(&bidder_address);
        assert_eq!(same_allocation.purchase_amount, update_amount);

        // Check that a new address creates a new allocation
        let another_address = Address::random();
        let another_allocation = allocations.get_or_create_bidder_allocation(&another_address);
        assert_eq!(another_allocation.purchase_amount, U256::ZERO);
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
