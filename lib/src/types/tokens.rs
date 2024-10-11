use crate::types::HashableStruct;
use alloy_sol_types::sol;
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

impl HashableStruct for Tokens {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{keccak256, Address, B256, U256};
    use alloy_sol_types::SolValue;

    #[test]
    fn test_hash() {
        let token_prices: Tokens = Tokens {
            purchaseToken: Address::random(),
            purchasePrice: U256::from(rand::random::<u128>()),
            collateralToken: Address::random(),
            collateralPrice: U256::from(rand::random::<u128>()),
        };

        // Recreates the onchain process
        let mut encoded_token_prices: Vec<u8> = Vec::new();
        encoded_token_prices.extend_from_slice(&token_prices.purchaseToken.abi_encode_packed());
        encoded_token_prices.extend_from_slice(&token_prices.purchasePrice.abi_encode_packed());
        encoded_token_prices.extend_from_slice(&token_prices.collateralToken.abi_encode_packed());
        encoded_token_prices.extend_from_slice(&token_prices.collateralPrice.abi_encode_packed());
        let expected_output: B256 = keccak256(&encoded_token_prices);

        // Testing with `sp1_keccak256`
        let sp1_output: B256 = token_prices.hash(&|x: &[u8]| keccak256(x));
        assert_eq!(sp1_output, expected_output);

        // Testing with `risc0_keccak256`
        // let risc0_output: B256 = hash_unrolled(&risc0_keccak256, &tokens);
        // assert_eq!(risc0_output, expected_output);
    }
}
