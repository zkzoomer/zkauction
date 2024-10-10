use crate::types::UnrollableStructs;
use alloy_primitives::{Address, B256, U256};
use alloy_sol_types::{sol, SolValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

sol! {
    /// A `TokenPrice` represents a given ERC-20 token address and its oracle price at proof verification time
    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct TokenPrice {
        /// The address of the ERC-20 token
        address tokenAddress;
        /// The oracle price of the token at proof verification time
        uint256 price;
    }
}

/// A collection of all token information at proof verification time.
pub type TokenPrices = Vec<TokenPrice>;

impl UnrollableStructs for TokenPrices {
    /// Computes a hash of multiple items concatenated together.
    ///
    /// # Arguments
    ///
    /// * `self` - The slice of items to be hashed, each implementing the `SolValue` trait.
    /// * `hash_function` - A function that computes a 32-byte hash from a byte slice.
    ///
    /// # Returns
    ///
    /// A 32-byte array representing the hash of the concatenated items.
    ///
    /// # Type Parameters
    ///
    /// * `F` - The type of the hash function.
    /// * `S` - The type of items
    fn hash_together<F: Fn(&[u8]) -> B256>(&self, hash_function: &F) -> B256 {
        hash_function(&self.abi_encode_packed())
    }
}

/// A map of token addresses to their prices.
pub type TokenMap = BTreeMap<Address, U256>;

pub trait IntoTokenMap {
    fn to_token_map(&self) -> TokenMap;
}

impl IntoTokenMap for TokenPrices {
    fn to_token_map(&self) -> TokenMap {
        self.iter()
            .map(|token| (token.tokenAddress, token.price))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::keccak256;

    #[test]
    fn test_hash_together() {
        let token_prices: TokenPrices = (0..2)
            .map(|_| TokenPrice {
                tokenAddress: Address::random(),
                price: U256::from(rand::random::<u128>()),
            })
            .collect();

        // Recreates the onchain process
        let mut encoded_token_prices: Vec<u8> = Vec::new();
        for token in &token_prices {
            let serialized_token_prices: Vec<u8> = token.abi_encode_packed();
            encoded_token_prices.extend_from_slice(&serialized_token_prices);
        }
        let expected_output: B256 = keccak256(&encoded_token_prices);

        // Testing with `sp1_keccak256`
        let sp1_output: B256 = token_prices.hash_together(&|x: &[u8]| keccak256(x));
        assert_eq!(sp1_output, expected_output);

        // Testing with `risc0_keccak256`
        // let risc0_output: B256 = hash_unrolled(&risc0_keccak256, &tokens);
        // assert_eq!(risc0_output, expected_output);
    }

    #[test]
    fn test_into_token_map() {
        let mut expected_token_map: TokenMap = TokenMap::new();
        let token_prices: TokenPrices = (0..2)
            .map(|_| {
                let token_address: Address = Address::random();
                let price: U256 = U256::from(rand::random::<u128>());
                expected_token_map.insert(token_address, price);
                TokenPrice {
                    tokenAddress: token_address,
                    price,
                }
            })
            .collect();

        let token_map: TokenMap = token_prices.to_token_map();

        assert_eq!(token_map.len(), token_prices.len());
        assert_eq!(token_map, expected_token_map);
    }
}
