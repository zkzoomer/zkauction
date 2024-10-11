use alloy_primitives::B256;
use alloy_sol_types::{sol, SolValue};

sol! {
    #[derive(PartialEq, Eq, Debug)]
    struct ExitLeafTokenWithdrawal {
        /// The recipient of the withdrawal
        address recipient;
        /// The token being withdrawn
        address token;
        /// The amount being withdrawn
        uint256 amount;
    }

    struct ExitLeafRepoTokenWithdrawal {
        /// The recipient of the withdrawal
        address recipient;
        /// The amount being withdrawn
        uint256 amount;
    }

    #[derive(PartialEq, Eq, Debug)]
    struct ExitLeafRepurchaseObligation {
        /// The debtor of the repurchase obligation
        address debtor;
        /// The amount being repurchased
        uint256 repurchaseAmount;
        /// The amount of collateral being repurchased
        uint256 collateralAmount;
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum ExitLeaf {
    Withdrawal(ExitLeafTokenWithdrawal),
    RepurchaseObligation(ExitLeafRepurchaseObligation),
}

impl ExitLeaf {
    pub fn hash<F: Fn(&[u8]) -> B256>(&self, hash_function: &F) -> B256 {
        match self {
            ExitLeaf::Withdrawal(withdrawal) => hash_function(&withdrawal.abi_encode_packed()),
            ExitLeaf::RepurchaseObligation(obligation) => {
                hash_function(&obligation.abi_encode_packed())
            }
        }
    }
}

pub type ExitLeaves = Vec<ExitLeaf>;

/// Defines a lean incremental Merkle tree.
pub trait ExitTree {
    /// Computes the root of a lean incremental Merkle tree from a list of leaves.
    ///
    /// This function implements a bottom-up approach to calculate the Merkle root:
    /// it iteratively combines pairs of hashes at each level until a single root hash is obtained.
    /// When a node lacks a right counterpart, it adopts the left child's value.
    /// The tree's depth dynamically adjusts to the count of leaves, enhancing efficiency
    /// by minimizing the number of hash computations.
    /// For a better understanding, refer to the [visual explanation](https://hackmd.io/@vplasencia/S1whLBN16).
    ///
    /// # Arguments
    ///
    /// * `self` - A slice of `SolValue` elements representing the leaves of the tree.
    /// * `hash_function` - A function that computes a 32-byte hash from a byte slice.
    ///
    /// # Returns
    ///
    /// A 32-byte array representing the root of the Merkle tree. If the input `leaves` is empty, it returns a zero byte array.
    fn hash_exit_root<F: Fn(&[u8]) -> B256>(&self, hash_function: &F) -> B256;
}

impl ExitTree for ExitLeaves {
    fn hash_exit_root<F: Fn(&[u8]) -> B256>(&self, hash_function: &F) -> B256 {
        if self.is_empty() {
            return B256::ZERO;
        }

        // Get the hash of each leaf
        let mut current_level: Vec<B256> = self
            .iter()
            .map(|leaf: &ExitLeaf| leaf.hash(hash_function))
            .collect();

        // Hash the leaves in pairs or keep the leaf if there's no pair until we get the root
        while current_level.len() > 1 {
            current_level = current_level
                .chunks(2)
                .map(|chunk: &[B256]| {
                    if chunk.len() == 2 {
                        let input: Vec<u8> = [&chunk[0][..], &chunk[1][..]].concat();
                        hash_function(&input)
                    } else {
                        chunk[0]
                    }
                })
                .collect();
        }

        current_level[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::precompiles::sp1_keccak256;
    use crate::utils::lean_imt::LeanIncrementalMerkleTree;
    use alloy_primitives::{keccak256, Address, B256, U256};
    use rand::{
        distributions::{Distribution, Standard},
        Rng,
    };

    #[test]
    fn test_hash_exit_root() {
        // Setup
        let mut leaves: Vec<B256> = Vec::new();
        let exit_leaves: ExitLeaves = (0..11)
            .map(|_| {
                let exit_leaf: ExitLeaf = rand::random();
                leaves.push(exit_leaf.hash(&|x: &[u8]| keccak256(x)));
                exit_leaf
            })
            .collect();

        // Calculate expected result
        let expected_tree: LeanIncrementalMerkleTree = LeanIncrementalMerkleTree::new(&leaves);
        let expected_output: B256 = expected_tree.root();

        // Testing with `sp1_keccak256`
        let sp1_output: B256 = exit_leaves.hash_exit_root(&sp1_keccak256);
        assert_eq!(sp1_output, expected_output);

        // TODO: Test with risc0_keccak256 once implemented
        //let risc0_output = hash_exit_root(&risc0_keccak256, &exit_root);
        //assert_eq!(risc0_output, expected_output);
    }

    // HELPER FUNCTIONS
    /// Creates a random `ExitLeaf`
    impl Distribution<ExitLeaf> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ExitLeaf {
            match rng.gen_range(0..=1) {
                0 => ExitLeaf::Withdrawal(ExitLeafTokenWithdrawal {
                    recipient: Address::random(),
                    token: Address::random(),
                    amount: U256::from(rand::random::<u128>()),
                }),
                1 => ExitLeaf::RepurchaseObligation(ExitLeafRepurchaseObligation {
                    debtor: Address::random(),
                    repurchaseAmount: U256::from(rand::random::<u128>()),
                    collateralAmount: U256::from(rand::random::<u128>()),
                }),
                _ => unreachable!(),
            }
        }
    }
}
