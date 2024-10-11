pub mod lean_imt;

use crate::types::{bids::ValidatedBids, offers::ValidatedOffers};
use alloy_primitives::U256;

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
pub fn compute_clearing_rate(bids: &ValidatedBids, offers: &ValidatedOffers) -> U256 {
    if bids.len() < 2 || offers.len() < 2 {
        // TODO: What happens here? what's the clearing rate then?
        return U256::ZERO;
    }

    (bids[1].bid_price_revealed + offers[1].offer_price_revealed) / U256::from(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::bids::Bid;
    use crate::types::offers::Offer;
    use crate::types::ValidatedOrders;
    use alloy_primitives::{aliases::U96, Address, B256};

    #[test]
    fn test_compute_clearing_rate() {
        // Create sample bids
        let mut validated_bids: ValidatedBids = vec![
            random_validated_bid(),
            random_validated_bid(),
            random_validated_bid(),
        ];
        validated_bids.sort_orders();
        // Creat sample offers
        let mut validated_offers: ValidatedOffers = vec![
            random_validated_offer(),
            random_validated_offer(),
            random_validated_offer(),
        ];
        validated_offers.sort_orders();
        // Compute the clearing rate
        let clearing_rate = compute_clearing_rate(&validated_bids, &validated_offers);

        // Expected clearing rate: (90 + 80) / 2 = 85
        assert_eq!(
            clearing_rate,
            (validated_bids[1].bid_price_revealed + validated_offers[1].offer_price_revealed)
                / U256::from(2)
        );

        // Test with fewer bids and offers
        let validated_bids: ValidatedBids = vec![random_validated_bid()];
        let validated_offers: ValidatedOffers = vec![random_validated_offer()];
        let clearing_rate = compute_clearing_rate(&validated_bids, &validated_offers);
        assert_eq!(clearing_rate, U256::ZERO);
    }

    // HELPER FUNCTIONS
    /// Creates a random validated Bid.
    fn random_validated_bid() -> Bid {
        Bid {
            id: U96::from(rand::random::<u64>()),
            bidder: Address::random(),
            bid_price_hash: B256::random(),
            bid_price_revealed: U256::from(rand::random::<u64>() % crate::constants::MAX_BID_PRICE),
            amount: U256::from(rand::random::<u128>()),
            collateral_amount: U256::from(rand::random::<u128>()),
            is_rollover: false,
            rollover_pair_off_term_repo_servicer: Address::ZERO,
            is_revealed: true,
        }
    }

    /// Creates a random validated Offer.
    fn random_validated_offer() -> Offer {
        Offer {
            id: U96::from(rand::random::<u64>()),
            offeror: Address::random(),
            offer_price_hash: B256::random(),
            offer_price_revealed: U256::from(
                rand::random::<u64>() % crate::constants::MAX_OFFER_PRICE,
            ),
            amount: U256::from(rand::random::<u128>()),
            is_revealed: true,
        }
    }
}
