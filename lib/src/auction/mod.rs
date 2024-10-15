use alloy_primitives::U256;

use crate::orders::{bids::ValidatedBids, offers::ValidatedOffers};

/// Computes the clearing rate as the average of the second most competitive bid and the second most competitive offer.
///
/// # Arguments
///
/// * `bids` - The validated bids.
/// * `offers` - The validated offers.
///
/// # Returns
///
/// The clearing rate as a U256.
pub fn compute_clearing_price(bids: &ValidatedBids, offers: &ValidatedOffers) -> (U256, U256) {
    // Bids and offers are inversely ordered: the most competitive bid/offer is located at the last index of the array
    (
        U256::ZERO,
        (bids[bids.len() - 2].bid_price_revealed + offers[offers.len() - 2].offer_price_revealed)
            / U256::from(2),
    )
}

/// Trait for assigning bids and offers to auction results.
pub trait Assignable {
    /// Assigns bids or offers up to a maximum assignable amount at a clearing rate.
    ///
    /// # Arguments
    ///
    /// * `self` - The bids or offers to assign.
    /// * `max_assignable` - The maximum amount that can be assigned.
    /// * `clearing_rate` - The clearing rate at which to assign the orders.
    fn assign(self, max_assignable: &U256, clearing_rate: &U256);
}

impl Assignable for ValidatedBids {
    fn assign(self, max_assignable: &U256, clearing_rate: &U256) {}
}

impl Assignable for ValidatedOffers {
    fn assign(self, max_assignable: &U256, clearing_rate: &U256) {}
}

#[cfg(test)]
mod tests {
    use crate::orders::{
        bids::{tests::random_revealed_bid, Bid},
        offers::{tests::random_revealed_offer, Offer},
    };

    use super::*;
    use alloy_primitives::{Address, U256};

    #[test]
    fn test_compute_clearing_price() {
        unimplemented!()
    }

    #[test]
    fn test_assign_validated_bids() {
        unimplemented!()
    }

    #[test]
    fn test_assign_validated_offers() {
        unimplemented!()
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
