pub mod precompiles;
pub mod types;
pub mod utils;

use alloy_primitives::B256;
pub use types::{
    BidReveal, BidSubmission, OfferReveal, OfferSubmission, PublicValuesStruct, TokenInformation,
};
use utils::{hash_chain, hash_unrolled};

/// Executes the auction process and computes the public values.
///
/// This function takes the auction data (bids, offers, revealed information, and token details)
/// and a hash function to compute the necessary hashes for the auction's public values.
///
/// # Arguments
///
/// * `hash_function` - A closure that takes a vector of bytes and returns a 32-byte array hash.
/// * `bids` - A vector of bid submissions.
/// * `offers` - A vector of offer submissions.
/// * `revealed_bids` - A vector of revealed bid information.
/// * `revealed_offers` - A vector of revealed offer information.
/// * `tokens` - A vector of token information for the assets involved in the auction.
///
/// # Returns
///
/// Returns a `PublicValuesStruct` containing the computed hashes and auction result root.
pub fn run_auction<F: Fn(&[u8]) -> B256>(
    hash_function: &F,
    bids: Vec<BidSubmission>,
    offers: Vec<OfferSubmission>,
    revealed_bids: Vec<BidReveal>,
    revealed_offers: Vec<OfferReveal>,
    tokens: Vec<TokenInformation>,
) -> (B256, B256, B256, B256) {
    // Compute the hash chain for the bids
    let mut acc_bids_hash: B256 = hash_chain(&sp1_keccak256, &bids, &B256::ZERO);
    acc_bids_hash = hash_chain(&sp1_keccak256, &revealed_bids, &acc_bids_hash);

    // Compute the hash chain for the offers
    let mut acc_offers_hash: B256 = hash_chain(&sp1_keccak256, &offers, &B256::ZERO);
    acc_offers_hash = hash_chain(&sp1_keccak256, &revealed_offers, &acc_offers_hash);

    // Compute the hash of the information of the tokens involved in the auction
    let tokens_hash: B256 = hash_unrolled(&sp1_keccak256, &tokens);

    // TODO: Compute the auction result root
    let auction_result_root = B256::ZERO;

    // Create and return the PublicValuesStruct
    (
        acc_bids_hash,
        acc_offers_hash,
        tokens_hash,
        auction_result_root,
    )
}
