pub mod bids;
pub mod offers;
pub mod tokens;
pub mod utils;
use alloy_primitives::B256;
use alloy_sol_types::sol;
use std::collections::HashMap;

sol! {
    /// The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct PublicValuesStruct {
        /// Reconstructed hash chain of all bids placed and revealed onchain
        bytes32 accBidsHash;
        /// Reconstructed hash chain of all offers placed and revealed onchain
        bytes32 accOffersHash;
        /// Hashed together information on the tokens involved
        bytes32 tokensHash;
        /// The root of the auction results tree
        bytes32 auctionResultRoot;
    }
}

pub trait ChainableOrders {
    type T;
    /// Computes an orders hash chain while updating the orders in the provided `orders` mapping with the revealed price information.
    ///
    /// # Arguments
    ///
    /// * `self` - The `T` instance containing all orders placed onchain.
    /// * `hash_function` - The hash function to use for the hash chain.
    /// * `start_value` - The initial 32-byte value to start the hash chain.
    /// * `orders` - A mutable reference to the `T` HashMap where all orders will be updated.
    fn hash_chain<F>(
        &self,
        hash_function: &F,
        start_value: B256,
        orders: &mut HashMap<B256, Self::T>,
    ) -> B256
    where
        F: Fn(&[u8]) -> B256;
}

pub trait UnrollableStructs {
    fn hash_together<F: Fn(&[u8]) -> B256>(&self, hash_function: &F) -> B256;
}
