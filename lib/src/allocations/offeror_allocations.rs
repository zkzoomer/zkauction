use alloy_primitives::{Address, U256};
use std::collections::BTreeMap;

use crate::{
    exit_tree::{ExitLeaf, ExitLeafRepoTokenWithdrawal, ExitLeafTokenWithdrawal, ExitLeaves},
    orders::offers::Offer,
    tokens::Tokens,
};

use super::{Allocation, Allocations};

/// Represents the allocation for an offeror in the auction.
pub struct OfferorAllocation {
    /// The amount of repo tokens assigned to the offeror, if any.
    repo_amount: U256,
    /// The amount of purchase tokens left on the table for the offeror, if any.
    purchase_amount: U256,
}

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

/// A map of offeror addresses to their respective allocations.
pub type OfferorAllocations = BTreeMap<Address, OfferorAllocation>;

impl Allocations for OfferorAllocations {
    type Allocation = OfferorAllocation;
    type Order = Offer;

    fn add_from_order(&mut self, order: &Self::Order) {
        let offeror_allocation: &mut OfferorAllocation = self.get_allocation(&order.offeror);
        offeror_allocation.update_purchase_amount(order.amount);
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
            offers::{
                tests::{random_offer_submission, random_revealed_offer},
                Offers, ValidatedOffers,
            },
            Order, PlacedOrders,
        },
        utils::get_key,
    };

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
    fn test_offeror_add_from_order() {
        let mut offeror_allocations: OfferorAllocations = OfferorAllocations::new();

        // Define two offers that originate from the same offeror
        let offer_a: Offer = Offer::from_order_submission(&random_offer_submission());
        let mut offer_b: Offer = Offer::from_order_submission(&random_offer_submission());
        offer_b.offeror = offer_a.offeror;

        // Defines an allocation from an order
        offeror_allocations.add_from_order(&offer_a);

        let allocation_a: &OfferorAllocation = offeror_allocations.get(&offer_a.offeror).unwrap();
        assert_eq!(allocation_a.repo_amount, U256::ZERO);
        assert_eq!(allocation_a.purchase_amount, offer_a.amount);

        // Correspondingly updates the allocation from adding another order
        offeror_allocations.add_from_order(&offer_b);

        let allocation_b: &OfferorAllocation = offeror_allocations.get(&offer_a.offeror).unwrap();
        assert_eq!(allocation_b.repo_amount, U256::ZERO);
        assert_eq!(
            allocation_b.purchase_amount,
            offer_a.amount + offer_b.amount
        );
    }

    #[test]
    fn test_offeror_get_allocation() {
        let mut auction_results: AuctionResults = AuctionResults::new(&Address::random());
        let offeror_address: Address = Address::random();

        // Get a new offeror allocation
        let offeror_allocation: &mut OfferorAllocation = auction_results
            .offeror_allocations
            .get_allocation(&offeror_address);
        assert_eq!(offeror_allocation.purchase_amount, U256::ZERO);

        // Modify the allocation
        let update_amount: U256 = U256::from(rand::random::<u64>());
        offeror_allocation.update_purchase_amount(update_amount);

        // Get the same allocation and check if it's modified
        let same_allocation: &mut OfferorAllocation = auction_results
            .offeror_allocations
            .get_allocation(&offeror_address);
        assert_eq!(same_allocation.purchase_amount, update_amount);

        // Check that a new address creates a new allocation
        let another_address: Address = Address::random();
        let another_allocation: &mut OfferorAllocation = auction_results
            .offeror_allocations
            .get_allocation(&another_address);
        assert_eq!(another_allocation.purchase_amount, U256::ZERO);
    }

    #[test]
    fn test_validate_offers() {
        let tokens: Tokens = random_tokens();

        let mut offeror_allocations: OfferorAllocations = OfferorAllocations::new();
        let revealed_offer: Offer = random_revealed_offer();
        let non_revealed_offer: Offer = Offer::from_order_submission(&random_offer_submission());

        let placed_offers: Offers = Offers::from([
            (
                get_key(&revealed_offer.offeror, &revealed_offer.id),
                revealed_offer.clone(),
            ),
            (
                get_key(&non_revealed_offer.offeror, &non_revealed_offer.id),
                non_revealed_offer.clone(),
            ),
        ]);

        let validated_offers: ValidatedOffers =
            placed_offers.into_validated_orders(&tokens, &mut offeror_allocations);

        // Revealed offer
        assert_eq!(validated_offers.len(), 1);
        assert_eq!(validated_offers[0], revealed_offer);

        // Non revealed offer is added to allocations
        assert_eq!(
            offeror_allocations
                .get(&non_revealed_offer.offeror)
                .unwrap()
                .purchase_amount,
            non_revealed_offer.amount
        );
        assert_eq!(
            offeror_allocations
                .get(&non_revealed_offer.offeror)
                .unwrap()
                .repo_amount,
            U256::ZERO
        );
    }

    #[test]
    fn test_offeror_into_exit_leaves() {
        let tokens: Tokens = random_tokens();

        // Empty offeror allocation pushes no new leaf
        let mut exit_leaves: ExitLeaves = ExitLeaves::new();
        let offeror_address: Address = Address::random();
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
