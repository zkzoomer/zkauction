use alloy_primitives::U256;

use crate::types::{
    allocations::AuctionResults, bids::ValidatedBids, offers::ValidatedOffers, tokens::Tokens,
};

/// Matches bids and offers and updates the auction results correspondingly.
///
/// # Arguments
///
/// * `tokens` - A reference to the `Tokens` involved in the auction.
/// * `clearing_rate` - The single price that clears the auction.
/// * `validated_bids` - A mutable reference to the `ValidatedBids` for the auction.
/// * `validated_offers` - A mutable reference to the `ValidatedOffers` for the auction.
/// * `auction_results` - A reference to the `AuctionResults` to be updated.
pub fn auction_match(
    tokens: &Tokens,
    clearing_rate: U256,
    validated_bids: &mut ValidatedBids,
    validated_offers: &mut ValidatedOffers,
    auction_results: &AuctionResults,
) {
}

#[cfg(test)]
mod tests {}
