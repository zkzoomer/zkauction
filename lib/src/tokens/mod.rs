use alloy_primitives::B256;
use alloy_sol_types::{sol, SolValue};
use serde::{Deserialize, Serialize};

sol! {
    /// A `TokenPrice` represents a given ERC-20 token address and its oracle price at proof verification time
    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Tokens {
        /// The purchase token address
        address purchaseToken;
        /// The oracle price of the purchase token at proof verification time
        uint256 purchasePrice;
        /// The collateral token address
        address collateralToken;
        /// The oracle price of the collateral token at proof verification time
        uint256 collateralPrice;
    }
}

/// Trait for Solidity structs that can be hashed via first calling `abi.encodePacked`.
pub trait HashableStruct: SolValue {
    /// Computes a single hash value from the struct's fields by first calling `abi.encodePacked`.
    ///
    /// # Arguments
    ///
    /// * `self` - The struct to hash.
    /// * `hash_function` - A function that computes a 32-byte hash from a byte slice.
    fn hash<F: Fn(&[u8]) -> B256>(&self, hash_function: &F) -> B256 {
        hash_function(&self.abi_encode_packed())
    }
}

impl HashableStruct for Tokens {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{keccak256, Address, B256, U256};
    use alloy_sol_types::SolValue;

    #[test]
    fn test_hash() {
        let tokens: Tokens = random_tokens();

        // Recreates the onchain process
        let mut encoded_tokens: Vec<u8> = Vec::new();
        encoded_tokens.extend_from_slice(&tokens.purchaseToken.abi_encode_packed());
        encoded_tokens.extend_from_slice(&tokens.purchasePrice.abi_encode_packed());
        encoded_tokens.extend_from_slice(&tokens.collateralToken.abi_encode_packed());
        encoded_tokens.extend_from_slice(&tokens.collateralPrice.abi_encode_packed());
        let expected_output: B256 = keccak256(&encoded_tokens);

        // Testing with `sp1_keccak256`
        let sp1_output: B256 = tokens.hash(&|x: &[u8]| keccak256(x));
        assert_eq!(sp1_output, expected_output);

        // Testing with `risc0_keccak256`
        // let risc0_output: B256 = hash_unrolled(&risc0_keccak256, &tokens);
        // assert_eq!(risc0_output, expected_output);
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
