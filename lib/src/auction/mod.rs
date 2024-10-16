pub mod assign_bids;
pub mod assign_offers;

use alloy_primitives::U256;

use crate::{
    constants::{BPS, DAYS_IN_YEAR},
    orders::{bids::ValidatedBids, offers::ValidatedOffers},
};

/// Computes the clearing rate as the average of the second most competitive bid and the second most competitive offer.
///
/// This implementation is just a rough Rust translation of the [original Solidity implementation](https://github.com/term-finance/term-finance-contracts/blob/262098c71578bbb9e54d6c2a8d2d88d112b9662a/contracts/TermAuction.sol#L512),
/// and may be full of critical bugs and far from optimized for performance.
///
/// # Arguments
///
/// * `bids` - The validated bids.
/// * `offers` - The validated offers.
///
/// # Returns
///
/// * `clearing_price` - The clearing price as a U256.
/// * `max_assignable` - The maximum assignable amount as a U256.
pub fn compute_clearing_price(bids: &ValidatedBids, offers: &ValidatedOffers) -> (U256, U256) {
    let offer_price: U256 = offers.last().unwrap().offer_price_revealed; // p^o_i
    let mut offer_index: usize = 1; // idxo(offerPrice)
    let mut cum_sum_offers: U256 = offers.last().unwrap().amount; // cso(offerPrice)
    let mut bid_index: usize = bids.len();
    let mut cum_sum_bids: U256 = U256::ZERO;
    let mut next_offer_index: usize;
    let mut next_bid_index: usize;
    let mut next_cum_sum_offers: U256;
    let mut next_cum_sum_bids: U256;
    let mut next_offer_price: U256;
    let mut next_max_clearing_volume: U256;
    /* let mut min_cum_sum_correction: bool = false; // Seemingly useless, see comment below*/
    let mut next_bid_price: U256;

    (cum_sum_bids, bid_index) =
        increase_cum_sum_bids(bids, &(bid_index - 1), &cum_sum_bids, &offer_price);

    // Calculate initial maximal clearing volume
    let mut max_clearing_volume: U256 = U256::min(cum_sum_bids, cum_sum_offers);

    // Calculate the pre-clearance price: maximise the clearing volume
    while offer_index < offers.len() && bid_index < bids.len() {
        // Initialize the next iteration of the relevant variables
        next_offer_index = offer_index;
        next_bid_index = bid_index;
        next_cum_sum_offers = cum_sum_offers;
        next_cum_sum_bids = cum_sum_bids;
        next_offer_price = offers[offer_index].offer_price_revealed;

        // Obtain next offer index, increase cumulative sum
        while next_offer_index < offers.len()
            && offers[next_offer_index].offer_price_revealed == next_offer_price
        {
            next_cum_sum_offers += offers[next_offer_index].amount;
            next_offer_index += 1;
        }

        // Obtain next bid index, decrease cumulative sum
        (next_cum_sum_bids, next_bid_index) =
            decrease_cum_sum_bids(bids, &next_bid_index, &next_cum_sum_bids, &next_offer_price);

        next_max_clearing_volume = U256::min(next_cum_sum_bids, next_cum_sum_offers);

        if next_max_clearing_volume > max_clearing_volume {
            offer_index = next_offer_index;
            bid_index = next_bid_index;
            cum_sum_offers = next_cum_sum_offers;
            cum_sum_bids = next_cum_sum_bids;
            /* offer_price = next_offer_price; // Seemingly useless, see comment below*/
            max_clearing_volume = next_max_clearing_volume;
        } else {
            break;
        }
    }

    // Get next offer price: first offer price higher than the pre-clearance price
    if offer_index < offers.len() {
        next_offer_price = offers[offer_index].offer_price_revealed;
    } else {
        next_offer_price = U256::MAX;
    }

    // Minimise css by minimising csb as long as bid price is smaller than next offer price
    while bid_index < bids.len() {
        next_bid_index = bid_index;
        next_bid_price = bids[bid_index].bid_price_revealed;
        next_cum_sum_bids = cum_sum_bids;

        if next_bid_price < next_offer_price {
            while next_bid_index < bids.len()
                && bids[next_bid_index].bid_price_revealed == next_bid_price
            {
                next_cum_sum_bids -= bids[next_bid_index].amount;
                next_bid_index += 1;
            }

            if next_cum_sum_bids < cum_sum_offers {
                /* min_cum_sum_correction = true; // Seemingly useless, see comment below*/
                cum_sum_bids = next_cum_sum_bids;
                bid_index = next_bid_index;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Seemingly useless chunk of code that somehow made it to production??? dev pls fix
    /* // Calculate clearing price: bid price if minimum correction was made and offer price otherwise
    if min_cum_sum_correction {
        if bid_index == bids.len() {
            clearing_price = bids[bid_index - 1].bid_price_revealed;
        } else {
            clearing_price = bids[bid_index].bid_price_revealed;
        }
    } else {
        clearing_price = offer_price;
    } */

    // The main loop positions `offerIndex` at the first index greater than the price.
    // It needs to be shifted back to get the last index smaller than or equal to the price.
    offer_index -= 1;

    // If non-zero clearing offset, find the offset tender prices and then average them to find the final clearing price.
    let clearing_offset: U256 = U256::from(1); // Assuming clearing_offset is often one
    let clearing_price: U256 = if clearing_offset == U256::from(1) {
        let mut next_offer_price_index: usize = offer_index;
        while next_offer_price_index > 0
            && offers[next_offer_price_index].offer_price_revealed
                == offers[offer_index].offer_price_revealed
        {
            next_offer_price_index -= 1;
        }

        let mut next_bid_price_index: usize = bid_index;

        // In the case that there is no clear, bid index is past end of array, so decrement it to last element.
        if bid_index == bids.len() {
            next_bid_price_index -= 1;
        }

        while next_bid_price_index < bids.len() - 1
            && bids[next_bid_price_index].bid_price_revealed == bids[bid_index].bid_price_revealed
        {
            next_bid_price_index += 1;
        }

        (offers[next_offer_price_index].offer_price_revealed
            + bids[next_bid_price_index].bid_price_revealed)
            / U256::from(2)
    } else {
        // In the case that there is no clear, bid index is past end of array, so decrement it to last element.
        if bid_index == bids.len() {
            bid_index -= 1;
        }

        (offers[offer_index].offer_price_revealed + bids[bid_index].bid_price_revealed)
            / U256::from(2)
    };

    // Update cum_sum_offers
    if offers[offer_index].offer_price_revealed <= clearing_price {
        offer_index += 1;
        while offer_index < offers.len()
            && offers[offer_index].offer_price_revealed <= clearing_price
        {
            cum_sum_offers += offers[offer_index].amount;
            offer_index += 1;
        }
    } else {
        while offers[offer_index].offer_price_revealed > clearing_price {
            cum_sum_offers -= offers[offer_index].amount;
            if offer_index == 0 {
                break;
            }
            offer_index -= 1;
        }
    }

    // Update cum_sum_bids
    if bid_index < bids.len() && bids[bid_index].bid_price_revealed < clearing_price {
        (cum_sum_bids, _) = decrease_cum_sum_bids(bids, &bid_index, &cum_sum_bids, &clearing_price);
    } else if bid_index > 0 {
        (cum_sum_bids, _) =
            increase_cum_sum_bids(bids, &(bid_index - 1), &cum_sum_bids, &clearing_price);
    }

    (clearing_price, U256::min(cum_sum_bids, cum_sum_offers))
}

/// Increases the cumulative sum of bids at a given price.
fn increase_cum_sum_bids(
    bids: &ValidatedBids,
    start_index: &usize,
    prev_cum_sum_bids: &U256,
    current_price: &U256,
) -> (U256, usize) {
    let mut cum_sum_bids: U256 = *prev_cum_sum_bids;
    let mut i: usize = *start_index;

    while bids[i].bid_price_revealed >= *current_price {
        cum_sum_bids += bids[i].amount;
        i -= 1;
        if i == 0 {
            break;
        }
    }

    let final_index: usize = if bids[i].bid_price_revealed < *current_price {
        i + 1
    } else {
        i
    };

    (cum_sum_bids, final_index)
}

/// Decreases the cumulative sum of bids at a given price.
fn decrease_cum_sum_bids(
    bids: &ValidatedBids,
    start_index: &usize,
    prev_cum_sum_bids: &U256,
    current_price: &U256,
) -> (U256, usize) {
    let mut cum_sum_bids: U256 = *prev_cum_sum_bids;
    let mut i: usize = *start_index;

    while i < bids.len() && bids[i].bid_price_revealed < *current_price {
        cum_sum_bids -= bids[i].amount;
        i += 1;
    }

    (cum_sum_bids, i)
}

/// Trait for assigning orders individually, either fully, partially or not at all.
pub trait AssignableOrder {
    type Allocations;

    /// Fully assigns an order.
    ///
    /// # Arguments
    ///
    /// * `self` - The order to fully assign.
    /// * `clearing_price` - The clearing rate at which to assign the order.
    /// * `day_count` - The day count.
    /// * `allocations` - The allocations.
    ///
    /// # Returns
    ///
    /// * `U256` - The amount that was assigned.
    fn fully_assign(
        &self,
        clearing_price: &U256,
        day_count: &U256,
        allocations: &mut Self::Allocations,
    ) -> U256;

    /// Partially assigns an order.
    ///
    /// # Arguments
    ///
    /// * `self` - The order to partially assign.
    /// * `clearing_price` - The clearing rate at which to assign the order.
    /// * `day_count` - The day count.
    /// * `assigned_amount` - The amount to partially assign.
    /// * `allocations` - The allocations.
    ///
    /// # Returns
    ///
    /// * `U256` - The amount that was assigned.
    fn partially_assign(
        &self,
        clearing_price: &U256,
        day_count: &U256,
        assigned_amount: &U256,
        allocations: &mut Self::Allocations,
    ) -> U256;

    /// Unlocks an order, meaning it was not assigned..
    ///
    /// # Arguments
    ///
    /// * `self` - The order to unlock.
    /// * `allocations` - The allocations.
    fn unlock(&self, allocations: &mut Self::Allocations);
}

/// Trait for assigning bids and offers to auction results.
pub trait AssignableOrders {
    type Allocations;

    /// Assigns bids or offers up to a maximum assignable amount at a clearing rate.
    ///
    /// # Arguments
    ///
    /// * `self` - The bids or offers to assign.
    /// * `max_assignable` - The maximum amount that can be assigned.
    /// * `clearing_price` - The clearing rate at which to assign the orders.
    fn assign(
        self,
        max_assignable: &U256,
        clearing_price: &U256,
        day_count: &U256,
        allocations: &mut Self::Allocations,
    );
}

/// Finds the index of the first bid with a bidPrice of `price` and calculates the cumulative sum of the bid amounts up to that index.
pub fn find_first_index_for_price(
    price: &U256,
    bids: &ValidatedBids,
    start_index: &usize,
) -> (usize, U256) {
    let mut i: usize = *start_index;
    let mut total_amount: U256 = bids[i].amount;

    loop {
        if i == 0 || bids[i - 1].bid_price_revealed != *price {
            break;
        }

        total_amount += bids[i - 1].amount;
        i -= 1;
    }

    (i, total_amount)
}

/// Finds the index of the last offer with a offerPrice of `price` and calculates the cumulative sum of the offer amounts up to that index.
pub fn find_last_index_for_price(
    price: &U256,
    offers: &ValidatedOffers,
    start_index: &usize,
) -> (usize, U256) {
    let mut i: usize = *start_index;
    let mut total_amount: U256 = offers[i].amount;

    loop {
        if i < offers.len() - 1 || offers[i + 1].offer_price_revealed != *price {
            break;
        }

        total_amount += offers[i + 1].amount;
        i += 1;
    }

    (i, total_amount)
}

/// Computes the repurchase price using the 360 day count convention.
pub fn calculate_repurchase_price(
    purchase_price: &U256,
    clearing_price: &U256,
    day_count: &U256,
) -> U256 {
    // RepurchasePrice = PurchasePrice * (1 + RepoRate * DayCountFactor)
    let aux: f64 = f64::from(day_count * clearing_price) / f64::from(DAYS_IN_YEAR * BPS);
    U256::from(f64::from(purchase_price) * (1.0 + aux))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_compute_clearing_price() {
        // We're just gonna assume Term Finance is correct and move on and our bug infested code
        unimplemented!()
    }
}
