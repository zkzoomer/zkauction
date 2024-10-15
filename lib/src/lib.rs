pub mod allocations;
pub mod auction;
pub mod constants;
pub mod exit_tree;
pub mod orders;
pub mod precompiles;
pub mod tokens;
pub mod utils;

use allocations::AuctionResults;
use alloy_primitives::{Address, B256};
use auction::{compute_clearing_price, Assignable};
use exit_tree::{ExitLeaves, ExitTree};
use orders::{
    bids::{BidReveals, BidSubmissions, Bids, ValidatedBids},
    offers::{OfferReveals, OfferSubmissions, Offers, ValidatedOffers},
    ChainableSubmissions, PlacedOrders, ValidatedOrders,
};
use tokens::{HashableStruct, Tokens};

/// Executes the auction process and computes the public values.
///
/// This function takes the auction data (bids, offers, revealed information, and token details)
/// and a hash function to compute the necessary hashes for the auction's public values.
///
/// # Arguments
///
/// * `hash_function` - A function that computes a 32-byte hash from a byte slice.
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
    prover_address: &Address,
    bid_submissions: &BidSubmissions,
    offer_submissions: &OfferSubmissions,
    bid_reveals: &BidReveals,
    offer_reveals: &OfferReveals,
    tokens: &Tokens,
) -> (B256, B256, B256, B256) {
    // Compute the hash chain for the bids
    let mut bids: Bids = Bids::new();
    let mut acc_bids_hash: B256 = bid_submissions.hash_chain(hash_function, B256::ZERO, &mut bids);
    acc_bids_hash = bid_reveals.hash_chain(hash_function, acc_bids_hash, &mut bids);

    // Compute the hash chain for the offers
    let mut offers: Offers = Offers::new();
    let mut acc_offers_hash: B256 =
        offer_submissions.hash_chain(hash_function, B256::ZERO, &mut offers);
    acc_offers_hash = offer_reveals.hash_chain(hash_function, acc_offers_hash, &mut offers);

    // Compute the hash of the information of the tokens involved in the auction
    let tokens_hash: B256 = tokens.hash(hash_function);

    // Define the auction results
    let mut auction_results: AuctionResults = AuctionResults::new(prover_address);

    // Get validated bids and offers
    let mut validated_bids: ValidatedBids =
        bids.into_validated_orders(tokens, &mut auction_results.bidder_allocations);
    let mut validated_offers: ValidatedOffers =
        offers.into_validated_orders(tokens, &mut auction_results.offeror_allocations);

    // Sort validated bids by *ascending* price. Orders right on the price edge will be partially filled.
    validated_bids.sort_orders();
    // Sort validated offers by *ascending* price. Orders right on the price edge will be partially filled.
    validated_offers.sort_orders();

    // Calculate a clearing price and assign bids and offers only if both bids and offers exist and market intersects
    if !validated_bids.is_empty()
        && !validated_offers.is_empty()
        && validated_bids.last().unwrap().bid_price_revealed
            >= validated_offers.first().unwrap().offer_price_revealed
    {
        let (max_assignable, clearing_rate) =
            compute_clearing_price(&validated_bids, &validated_offers);

        // Assign bids and offers
        validated_bids.assign(&max_assignable, &clearing_rate);
        validated_offers.assign(&max_assignable, &clearing_rate);
    } else {
        // Dump all validated bids and offers to their corresponding allocations
        validated_bids.unlock_outstanding_orders(&mut auction_results.bidder_allocations);
        validated_offers.unlock_outstanding_orders(&mut auction_results.offeror_allocations);
    }

    // Define the exit leaves
    let mut exit_leaves: ExitLeaves = ExitLeaves::new();
    // Add all auction results to exit leaves
    auction_results.into_exit_leaves(tokens, &mut exit_leaves);

    // Compute the auction result root
    let auction_result_root: B256 = exit_leaves.hash_exit_root(hash_function);

    // Create and return the PublicValuesStruct
    (
        acc_bids_hash,
        acc_offers_hash,
        tokens_hash,
        auction_result_root,
    )
}
