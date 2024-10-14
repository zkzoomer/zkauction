use super::{
    bidder_allocations::{BidderAllocation, BidderAllocations},
    exit_tree::{ExitLeaf, ExitLeafTokenWithdrawal, ExitLeaves},
    offeror_allocations::{OfferorAllocation, OfferorAllocations},
    tokens::Tokens,
};
use alloy_primitives::{Address, U256};

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

/// Represents all allocations in the system
pub struct Allocations {
    /// The prover's allocation
    prover_allocation: ProverAllocation,
    /// The allocations for each of the bidders in the auction
    bidder_allocations: BidderAllocations,
    /// The allocations for each of the offerors in the auction
    offeror_allocations: OfferorAllocations,
}

impl Allocations {
    /// Creates a new Allocations instance with the given prover address
    ///
    /// # Arguments
    ///
    /// * `prover_address` - The address of the prover.
    pub fn new(prover_address: &Address) -> Self {
        Allocations {
            prover_allocation: ProverAllocation::new(prover_address),
            bidder_allocations: BidderAllocations::new(),
            offeror_allocations: OfferorAllocations::new(),
        }
    }

    /// Returns a mutable reference to the prover allocation
    ///
    /// # Arguments
    ///
    /// * `self` - The allocations instance
    pub fn get_or_create_prover_allocation(&mut self) -> &mut ProverAllocation {
        &mut self.prover_allocation
    }

    /// Returns a mutable reference to the bidder allocation for the given address
    ///
    /// # Arguments
    ///
    /// * `self` - The allocations instance
    /// * `address` - The bidder's address
    pub fn get_or_create_bidder_allocation(&mut self, address: &Address) -> &mut BidderAllocation {
        self.bidder_allocations.entry(*address).or_default()
    }

    /// Returns a mutable reference to the offeror allocation for the given address
    ///
    /// # Arguments
    ///
    /// * `self` - The allocations instance
    /// * `address` - The offeror's address
    pub fn get_or_create_offeror_allocation(
        &mut self,
        address: &Address,
    ) -> &mut OfferorAllocation {
        self.offeror_allocations.entry(*address).or_default()
    }

    /// Converts all allocations into exit leaves
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
        exit_tree::{
            ExitLeaf, ExitLeafRepoTokenWithdrawal, ExitLeafRepurchaseObligation,
            ExitLeafTokenWithdrawal,
        },
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
    fn test_get_or_create_prover_allocation() {
        let prover_address: Address = Address::random();
        let mut allocations: Allocations = Allocations::new(&prover_address);

        // Get the prover allocation
        let prover_allocation: &mut ProverAllocation =
            allocations.get_or_create_prover_allocation();

        // Check that the prover address matches
        assert_eq!(prover_allocation.prover_address, prover_address);

        // Modify the allocation
        let update_amount: U256 = U256::from(rand::random::<u64>());
        prover_allocation.update_purchase_amount(update_amount);

        // Get the allocation again and check if it's the same (modified) instance
        let same_allocation: &mut ProverAllocation = allocations.get_or_create_prover_allocation();
        assert_eq!(same_allocation.purchase_amount, update_amount);
    }

    #[test]
    fn test_into_exit_leaves() {
        let tokens: Tokens = random_tokens();
        let mut exit_leaves: ExitLeaves = ExitLeaves::new();
        let mut allocations: Allocations = Allocations::new(&Address::random());

        // Add offeror allocation
        let offeror_address: Address = Address::random();
        let offeror_repo_amount: U256 = U256::from(100);
        let offeror_purchase_amount: U256 = U256::from(200);
        allocations
            .get_or_create_offeror_allocation(&offeror_address)
            .update_repo_amount(offeror_repo_amount);
        allocations
            .get_or_create_offeror_allocation(&offeror_address)
            .update_purchase_amount(offeror_purchase_amount);

        // Add bidder allocation
        let bidder_address: Address = Address::random();
        let bidder_purchase_amount: U256 = U256::from(300);
        let bidder_collateral_amount: U256 = U256::from(400);
        let bidder_repurchase_amount: U256 = U256::from(500);
        let bidder_repurchase_collateral: U256 = U256::from(600);
        allocations
            .get_or_create_bidder_allocation(&bidder_address)
            .update_purchase_amount(bidder_purchase_amount);
        allocations
            .get_or_create_bidder_allocation(&bidder_address)
            .update_collateral_amount(bidder_collateral_amount);
        allocations
            .get_or_create_bidder_allocation(&bidder_address)
            .update_repurchase_obligation(bidder_repurchase_amount, bidder_repurchase_collateral);

        // Convert allocations to exit leaves
        allocations.into_exit_leaves(&tokens, &mut exit_leaves);

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
