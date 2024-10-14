use super::{
    bidder_allocations::BidderAllocations,
    exit_tree::{ExitLeaf, ExitLeafTokenWithdrawal, ExitLeaves},
    offeror_allocations::OfferorAllocations,
    tokens::Tokens,
};
use alloy_primitives::{Address, U256};

/// Represents the allocation for the prover, which is credited with all the accrued fees
pub struct ProverAllocation {
    /// The Ethereum address of the prover
    prover_address: Address,
    /// The amount of purchase tokens, result of accrued fees, that are to be credited to the prover
    purchase_amount: U256,
}

impl ProverAllocation {
    /// Creates a new ProverAllocation with the given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the prover.
    fn new(prover_address: &Address) -> Self {
        Self {
            prover_address: *prover_address,
            purchase_amount: U256::ZERO,
        }
    }

    /// Updates the prover allocation purchase amount by adding the given amount.
    ///
    /// # Arguments
    ///
    /// * `self` - The prover allocation to update.
    /// * `amount` - The amount to increase it by.
    pub fn update_purchase_amount(&mut self, amount: U256) {
        self.purchase_amount = self.purchase_amount.saturating_add(amount);
    }

    /// Converts the ProverAllocation into exit leaves
    ///
    /// # Arguments
    ///
    /// * `self` - The prover allocation to convert.
    /// * `tokens` - A reference to the `Tokens` struct containing token information.
    /// * `exit_leaves` - A mutable reference to the vector of exit leaves to update.
    fn into_exit_leaves(self, tokens: &Tokens, exit_leaves: &mut ExitLeaves) {
        if self.purchase_amount != U256::ZERO {
            exit_leaves.push(ExitLeaf::TokenWithdrawal(ExitLeafTokenWithdrawal {
                recipient: self.prover_address,
                token: tokens.purchaseToken,
                amount: self.purchase_amount,
            }));
        }
    }
}

/// Trait for converting allocations into exit leaves
pub trait Allocation {
    /// Converts the allocation mapping into exit leaves and adds them to the `exit_leaves` vector.
    ///
    /// # Arguments
    ///
    /// * `self` - The allocation to convert.
    /// * `address` - The address associated with this allocation.
    /// * `tokens` - A reference to the `Tokens` struct containing token information.
    /// * `exit_leaves` - A mutable reference to the vector of exit leaves to update.
    fn into_exit_leaves(self, address: Address, tokens: &Tokens, exit_leaves: &mut ExitLeaves);
}

/// Trait for fetching allocations and defining them from invalid orders
pub trait Allocations {
    type Allocation;
    type Order;

    /// Returns a mutable reference to the allocation for the given address
    ///
    /// # Arguments
    ///
    /// * `self` - The allocations instance
    /// * `address` - The address being queried
    fn get_allocation(&mut self, address: &Address) -> &mut Self::Allocation;

    /// Creates or updates an allocation from an order
    ///
    /// # Arguments
    ///
    /// * `self` - The allocations instance
    /// * `order` - The order being added to the allocations
    fn add_from_order(&mut self, order: &Self::Order);
}

/// Represents the results of the auction
pub struct AuctionResults {
    /// The prover's allocation
    pub prover_allocation: ProverAllocation,
    /// The allocations for each of the bidders in the auction
    pub bidder_allocations: BidderAllocations,
    /// The allocations for each of the offerors in the auction
    pub offeror_allocations: OfferorAllocations,
}

impl AuctionResults {
    /// Creates a new AuctionResults instance with the given prover address
    ///
    /// # Arguments
    ///
    /// * `prover_address` - The address of the prover.
    pub fn new(prover_address: &Address) -> Self {
        AuctionResults {
            prover_allocation: ProverAllocation::new(prover_address),
            bidder_allocations: BidderAllocations::new(),
            offeror_allocations: OfferorAllocations::new(),
        }
    }

    /// Converts all auction result allocations into exit leaves
    ///
    /// # Arguments
    ///
    /// * `self` - The allocations instance
    /// * `tokens` - A reference to the `Tokens` struct containing token information.
    /// * `exit_leaves` - A mutable reference to the vector of exit leaves to update.
    pub fn into_exit_leaves(self, tokens: &Tokens, exit_leaves: &mut ExitLeaves) {
        self.prover_allocation.into_exit_leaves(tokens, exit_leaves);

        for (address, bidder_allocation) in self.bidder_allocations.into_iter() {
            bidder_allocation.into_exit_leaves(address, tokens, exit_leaves);
        }

        for (address, offeror_allocation) in self.offeror_allocations.into_iter() {
            offeror_allocation.into_exit_leaves(address, tokens, exit_leaves);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{
        bidder_allocations::BidderAllocation,
        exit_tree::{
            ExitLeaf, ExitLeafRepoTokenWithdrawal, ExitLeafRepurchaseObligation,
            ExitLeafTokenWithdrawal,
        },
        offeror_allocations::OfferorAllocation,
        tokens::Tokens,
    };
    use alloy_primitives::{Address, U256};

    #[test]
    fn test_update_purchase_amount_prover() {
        let mut prover_allocation: ProverAllocation = ProverAllocation::new(&Address::random());
        assert_eq!(prover_allocation.purchase_amount, U256::ZERO);

        let increase_amount: U256 = U256::from(rand::random::<u64>());
        prover_allocation.update_purchase_amount(increase_amount);
        assert_eq!(prover_allocation.purchase_amount, increase_amount);
    }

    #[test]
    fn test_prover_into_exit_leaves() {
        let mut exit_leaves: ExitLeaves = ExitLeaves::new();
        let tokens: Tokens = random_tokens();

        // Empty prover allocation pushes no new leaf
        let prover_allocation: ProverAllocation = ProverAllocation::new(&Address::random());
        prover_allocation.into_exit_leaves(&tokens, &mut exit_leaves);
        assert_eq!(exit_leaves.len(), 0);

        // Proper prover allocation pushes corresponding leaf
        let prover_address: Address = Address::random();
        let prover_purchase_amount: U256 = U256::from(rand::random::<u128>());
        let mut prover_allocation: ProverAllocation = ProverAllocation::new(&prover_address);
        prover_allocation.update_purchase_amount(prover_purchase_amount);
        prover_allocation.into_exit_leaves(&tokens, &mut exit_leaves);
        assert_eq!(exit_leaves.len(), 1);
        assert_eq!(
            exit_leaves[0],
            ExitLeaf::TokenWithdrawal(ExitLeafTokenWithdrawal {
                recipient: prover_address,
                token: tokens.purchaseToken,
                amount: prover_purchase_amount,
            }),
        )
    }

    #[test]
    fn test_into_exit_leaves() {
        let tokens: Tokens = random_tokens();
        let mut exit_leaves: ExitLeaves = ExitLeaves::new();
        let mut auction_results: AuctionResults = AuctionResults::new(&Address::random());

        // Add offeror allocation
        let offeror_address: Address = Address::random();
        let offeror_repo_amount: U256 = U256::from(100);
        let offeror_purchase_amount: U256 = U256::from(200);
        let offeror_allocation: &mut OfferorAllocation = auction_results
            .offeror_allocations
            .get_allocation(&offeror_address);
        offeror_allocation.update_repo_amount(offeror_repo_amount);
        offeror_allocation.update_purchase_amount(offeror_purchase_amount);

        // Add bidder allocation
        let bidder_address: Address = Address::random();
        let bidder_purchase_amount: U256 = U256::from(300);
        let bidder_collateral_amount: U256 = U256::from(400);
        let bidder_repurchase_amount: U256 = U256::from(500);
        let bidder_repurchase_collateral: U256 = U256::from(600);
        let bidder_allocation: &mut BidderAllocation = auction_results
            .bidder_allocations
            .get_allocation(&bidder_address);
        bidder_allocation.update_purchase_amount(bidder_purchase_amount);
        bidder_allocation.update_collateral_amount(bidder_collateral_amount);
        bidder_allocation
            .update_repurchase_obligation(bidder_repurchase_amount, bidder_repurchase_collateral);

        // Convert allocations to exit leaves
        auction_results.into_exit_leaves(&tokens, &mut exit_leaves);

        // Check the number of exit leaves
        assert_eq!(exit_leaves.len(), 5);

        // Check offeror exit leaves
        assert!(exit_leaves.contains(&ExitLeaf::RepoTokenWithdrawal(
            ExitLeafRepoTokenWithdrawal {
                recipient: offeror_address,
                amount: offeror_repo_amount,
            }
        )));
        assert!(
            exit_leaves.contains(&ExitLeaf::TokenWithdrawal(ExitLeafTokenWithdrawal {
                recipient: offeror_address,
                token: tokens.purchaseToken,
                amount: offeror_purchase_amount,
            }))
        );

        // Check bidder exit leaves
        assert!(
            exit_leaves.contains(&ExitLeaf::TokenWithdrawal(ExitLeafTokenWithdrawal {
                recipient: bidder_address,
                token: tokens.purchaseToken,
                amount: bidder_purchase_amount,
            }))
        );
        assert!(
            exit_leaves.contains(&ExitLeaf::TokenWithdrawal(ExitLeafTokenWithdrawal {
                recipient: bidder_address,
                token: tokens.collateralToken,
                amount: bidder_collateral_amount,
            }))
        );
        assert!(exit_leaves.contains(&ExitLeaf::RepurchaseObligation(
            ExitLeafRepurchaseObligation {
                debtor: bidder_address,
                repurchaseAmount: bidder_repurchase_amount,
                collateralAmount: bidder_repurchase_collateral,
            }
        )));
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
