use alloy_primitives::U256;

use crate::types::{
    allocations::{Allocations, AuctionResults},
    bids::{Bid, ValidatedBids},
    offers::{Offer, ValidatedOffers},
    tokens::Tokens,
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
    clearing_rate: U256,
    mut validated_bids: ValidatedBids,
    mut validated_offers: ValidatedOffers,
    auction_results: &mut AuctionResults,
) {
    loop {
        if !can_match(&clearing_rate, &validated_bids, &validated_offers) {
            auction_finish(validated_bids, validated_offers, auction_results);
            break;
        }

        // Auction matching
    }
}

/// Returns true if the auction can proceed, i.e. there are bids and offers that can be matched at the given clearing rate.
///
/// # Arguments
///
/// * `clearing_rate` - The price at which the auction is being cleared.
/// * `validated_bids` - A reference to the `ValidatedBids` for the auction.
/// * `validated_offers` - A reference to the `ValidatedOffers` for the auction.
fn can_match(
    clearing_rate: &U256,
    validated_bids: &ValidatedBids,
    validated_offers: &ValidatedOffers,
) -> bool {
    validated_bids.last().map_or(false, |last_bid: &Bid| {
        validated_offers.last().map_or(false, |last_offer: &Offer| {
            last_bid.bid_price_revealed >= *clearing_rate
                && last_offer.offer_price_revealed <= *clearing_rate
        })
    })
}

/// Updates allocations with the tenors that were partially filled/left on the table.
///
/// # Arguments
///
/// * `tokens` - A reference to the `Tokens` involved in the auction.
/// * `validated_bids` - A mutable reference to the outstanding `ValidatedBids` for the auction.
/// * `validated_offers` - A mutable reference to the outstanding `ValidatedOffers` for the auction.
/// * `auction_results` - A reference to the `AuctionResults` to be updated.
fn auction_finish(
    validated_bids: ValidatedBids,
    validated_offers: ValidatedOffers,
    auction_results: &mut AuctionResults,
) {
    for bid in validated_bids.into_iter() {
        auction_results.bidder_allocations.add_from_order(&bid);
    }
    for offer in validated_offers.into_iter() {
        auction_results.offeror_allocations.add_from_order(&offer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        allocations::AuctionResults,
        bids::{tests::random_revealed_bid, Bid, ValidatedBids},
        offers::{tests::random_revealed_offer, Offer, ValidatedOffers},
    };
    use alloy_primitives::{Address, U256};

    #[test]
    fn test_auction_match() {
        unimplemented!()
    }

    #[test]
    fn test_can_match() {
        // Empty bids returns false
        let clearing_rate: U256 = U256::ZERO;
        let validated_bids: ValidatedBids = ValidatedBids::new();
        let validated_offers: ValidatedOffers = ValidatedOffers::from([random_revealed_offer()]);
        assert!(!can_match(
            &clearing_rate,
            &validated_bids,
            &validated_offers
        ));

        // Empty offers returns false
        let clearing_rate: U256 = U256::ZERO;
        let validated_bids: ValidatedBids = ValidatedBids::from([random_revealed_bid()]);
        let validated_offers: ValidatedOffers = ValidatedOffers::new();
        assert!(!can_match(
            &clearing_rate,
            &validated_bids,
            &validated_offers
        ));

        // Bid under clearing rate returns false
        let clearing_rate: U256 = U256::from(rand::random::<u64>());
        let validated_bids: ValidatedBids =
            ValidatedBids::from([random_bid_from_price(clearing_rate - U256::from(1))]);
        let validated_offers: ValidatedOffers =
            ValidatedOffers::from([random_offer_from_price(clearing_rate)]);
        assert!(!can_match(
            &clearing_rate,
            &validated_bids,
            &validated_offers
        ));

        // Offer over clearing rate returns true
        let clearing_rate: U256 = U256::from(rand::random::<u64>());
        let validated_bids: ValidatedBids =
            ValidatedBids::from([random_bid_from_price(clearing_rate)]);
        let validated_offers: ValidatedOffers =
            ValidatedOffers::from([random_offer_from_price(clearing_rate + U256::from(1))]);
        assert!(!can_match(
            &clearing_rate,
            &validated_bids,
            &validated_offers
        ));

        // Matchable bid and offer returns true
        let clearing_rate: U256 = U256::from(rand::random::<u64>());
        let validated_bids: ValidatedBids =
            ValidatedBids::from([random_bid_from_price(clearing_rate)]);
        let validated_offers: ValidatedOffers =
            ValidatedOffers::from([random_offer_from_price(clearing_rate)]);
        assert!(can_match(
            &clearing_rate,
            &validated_bids,
            &validated_offers
        ));
    }

    #[test]
    fn test_auction_finish() {
        let prover_address: Address = Address::random();
        let mut auction_results: AuctionResults = AuctionResults::new(&prover_address);
        let validated_bids: ValidatedBids = ValidatedBids::from([random_revealed_bid()]);
        let validated_offers: ValidatedOffers = ValidatedOffers::from([random_revealed_offer()]);
        auction_finish(validated_bids, validated_offers, &mut auction_results);

        // Allocations get assigned
        assert_eq!(auction_results.bidder_allocations.len(), 1);
        assert_eq!(auction_results.offeror_allocations.len(), 1);
    }

    // TEST HELPER FUNCTIONS
    /// Defines a random bid with the given bid price
    fn random_bid_from_price(bid_price: U256) -> Bid {
        let mut bid: Bid = random_revealed_bid();
        bid.bid_price_revealed = bid_price;
        bid
    }

    /// Defines a random offer with the given offer price
    fn random_offer_from_price(offer_price: U256) -> Offer {
        let mut offer: Offer = random_revealed_offer();
        offer.offer_price_revealed = offer_price;
        offer
    }
}
