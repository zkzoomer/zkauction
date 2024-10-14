use super::exit_tree::{ExitLeafRepoTokenWithdrawal, ExitLeafTokenWithdrawal, ExitLeaves};
use super::tokens::Tokens;
use super::{allocations::Allocation, exit_tree::ExitLeaf};
use alloy_primitives::{Address, U256};
use std::collections::BTreeMap;

/// Represents the allocation for an offeror in the auction.
pub struct OfferorAllocation {
    /// The amount of repo tokens assigned to the offeror, if any.
    repo_amount: U256,
    /// The amount of purchase tokens left on the table for the offeror, if any.
    purchase_amount: U256,
}

/// A map of offeror addresses to their respective allocations.
pub type OfferorAllocations = BTreeMap<Address, OfferorAllocation>;

impl Default for OfferorAllocation {
    /// Creates a default `OfferorAllocation` with zero amounts.
    fn default() -> Self {
        Self {
            repo_amount: U256::ZERO,
            purchase_amount: U256::ZERO,
        }
    }
}

impl OfferorAllocation {
    /// Updates the repo token amount for the offeror.
    ///
    /// # Arguments
    ///
    /// * `self` - The allocation to be updated.
    /// * `amount` - The amount to add to the current repo amount.
    pub fn update_repo_amount(&mut self, amount: U256) {
        self.repo_amount = self.repo_amount.saturating_add(amount);
    }

    /// Updates the purchase token amount for the offeror.
    ///
    /// # Arguments
    ///
    /// * `self` - The allocation to be updated.
    /// * `amount` - The amount to add to the current purchase amount.
    pub fn update_purchase_amount(&mut self, amount: U256) {
        self.purchase_amount = self.purchase_amount.saturating_add(amount);
    }
}

impl Allocation for OfferorAllocation {
    fn into_exit_leaves(self, address: Address, tokens: &Tokens, exit_leaves: &mut ExitLeaves) {
        if self.repo_amount != U256::ZERO {
            exit_leaves.push(ExitLeaf::RepoTokenWithdrawal(ExitLeafRepoTokenWithdrawal {
                recipient: address,
                amount: self.repo_amount,
            }));
        }

        if self.purchase_amount != U256::ZERO {
            exit_leaves.push(ExitLeaf::TokenWithdrawal(ExitLeafTokenWithdrawal {
                recipient: address,
                token: tokens.purchaseToken,
                amount: self.purchase_amount,
            }));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::types::allocations::Allocations;

    use super::*;
    use alloy_primitives::U256;

    #[test]
    fn test_update_repo_amount() {
        let mut offeror_allocation: OfferorAllocation = OfferorAllocation::default();
        assert_eq!(offeror_allocation.repo_amount, U256::ZERO);

        let increase_amount: U256 = U256::from(rand::random::<u64>());
        offeror_allocation.update_repo_amount(increase_amount);
        assert_eq!(offeror_allocation.repo_amount, increase_amount);
    }

    #[test]
    fn test_update_purchase_amount_offeror() {
        let mut offeror_allocation: OfferorAllocation = OfferorAllocation::default();
        assert_eq!(offeror_allocation.purchase_amount, U256::ZERO);

        let increase_amount: U256 = U256::from(rand::random::<u64>());
        offeror_allocation.update_purchase_amount(increase_amount);
        assert_eq!(offeror_allocation.purchase_amount, increase_amount);
    }

    #[test]
    fn test_get_or_create_offeror_allocation() {
        let mut allocations = Allocations::new(&Address::random());
        let offeror_address = Address::random();

        // Get a new offeror allocation
        let offeror_allocation = allocations.get_or_create_offeror_allocation(&offeror_address);
        assert_eq!(offeror_allocation.purchase_amount, U256::ZERO);

        // Modify the allocation
        let update_amount: U256 = U256::from(rand::random::<u64>());
        offeror_allocation.update_purchase_amount(update_amount);

        // Get the same allocation and check if it's modified
        let same_allocation = allocations.get_or_create_offeror_allocation(&offeror_address);
        assert_eq!(same_allocation.purchase_amount, update_amount);

        // Check that a new address creates a new allocation
        let another_address = Address::random();
        let another_allocation = allocations.get_or_create_offeror_allocation(&another_address);
        assert_eq!(another_allocation.purchase_amount, U256::ZERO);
    }

    #[test]
    fn test_offeror_into_exit_leaves() {
        let tokens: Tokens = random_tokens();

        // Empty offeror allocation pushes no new leaf
        let mut exit_leaves: ExitLeaves = ExitLeaves::new();
        let offeror_address = Address::random();
        let offeror_allocation: OfferorAllocation = OfferorAllocation::default();
        offeror_allocation.into_exit_leaves(offeror_address, &tokens, &mut exit_leaves);
        assert_eq!(exit_leaves.len(), 0);

        // Proper offeror repo allocation pushes corresponding leaf
        let mut exit_leaves: ExitLeaves = ExitLeaves::new();
        let offeror_address: Address = Address::random();
        let offeror_repo_amount: U256 = U256::from(rand::random::<u128>());
        let mut offeror_allocation: OfferorAllocation = OfferorAllocation::default();
        offeror_allocation.update_repo_amount(offeror_repo_amount);
        offeror_allocation.into_exit_leaves(offeror_address, &tokens, &mut exit_leaves);
        assert_eq!(exit_leaves.len(), 1);
        assert_eq!(
            exit_leaves[0],
            ExitLeaf::RepoTokenWithdrawal(ExitLeafRepoTokenWithdrawal {
                recipient: offeror_address,
                amount: offeror_repo_amount,
            }),
        );

        // Proper offeror purchase allocation pushes corresponding leaf
        let mut exit_leaves: ExitLeaves = ExitLeaves::new();
        let offeror_address: Address = Address::random();
        let offeror_purchase_amount: U256 = U256::from(rand::random::<u128>());
        let mut offeror_allocation: OfferorAllocation = OfferorAllocation::default();
        offeror_allocation.update_purchase_amount(offeror_purchase_amount);
        offeror_allocation.into_exit_leaves(offeror_address, &tokens, &mut exit_leaves);
        assert_eq!(exit_leaves.len(), 1);
        assert_eq!(
            exit_leaves[0],
            ExitLeaf::TokenWithdrawal(ExitLeafTokenWithdrawal {
                recipient: offeror_address,
                token: tokens.purchaseToken,
                amount: offeror_purchase_amount,
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
