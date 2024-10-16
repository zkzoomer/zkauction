use alloy_primitives::U256;

use crate::{
    allocations::{
        bidder_allocations::{BidderAllocation, BidderAllocations},
        Allocations,
    },
    orders::bids::{Bid, ValidatedBids},
};

use super::{
    calculate_repurchase_price, find_first_index_for_price, AssignableOrder, AssignableOrders,
};

impl AssignableOrder for Bid {
    type Allocations = BidderAllocations;

    fn fully_assign(
        &self,
        clearing_price: &U256,
        day_count: &U256,
        bidder_allocations: &mut BidderAllocations,
    ) -> U256 {
        let repurchase_amount: U256 =
            calculate_repurchase_price(&self.amount, clearing_price, day_count);

        let bidder_allocation: &mut BidderAllocation =
            bidder_allocations.get_allocation(&self.bidder);
        bidder_allocation.update_purchase_amount(self.amount);
        bidder_allocation.update_repurchase_obligation(repurchase_amount, self.collateral_amount);

        self.amount
    }

    fn partially_assign(
        &self,
        clearing_price: &U256,
        day_count: &U256,
        assigned_amount: &U256,
        bidder_allocations: &mut BidderAllocations,
    ) -> U256 {
        let repurchase_amount: U256 =
            calculate_repurchase_price(assigned_amount, clearing_price, day_count);

        let bidder_allocation: &mut BidderAllocation =
            bidder_allocations.get_allocation(&self.bidder);
        bidder_allocation.update_purchase_amount(*assigned_amount);
        bidder_allocation.update_repurchase_obligation(repurchase_amount, self.collateral_amount);

        *assigned_amount
    }

    fn unlock(&self, bidder_allocations: &mut BidderAllocations) {
        bidder_allocations.add_from_order(self);
    }
}

impl AssignableOrders for ValidatedBids {
    type Allocations = BidderAllocations;

    fn assign(
        self,
        max_assignable: &U256,
        clearing_price: &U256,
        day_count: &U256,
        allocations: &mut BidderAllocations,
    ) {
        // Process revealed bids
        let mut total_assigned_bids: U256 = U256::ZERO;
        let mut inner_index: usize;
        let mut i: usize;
        let mut j: usize = self.len();

        while j > 0 {
            i = j - 1;

            // First, find the sub-range that contains the current price.
            let (k, mut price_group_amount) =
                find_first_index_for_price(&self[i].bid_price_revealed, &self, &i);
            // NOTE: priceGroupAmount gets changed later on in this function and is used as the "remaining" priceGroupAmount during partial assignment.

            if self[i].bid_price_revealed >= *clearing_price
                && total_assigned_bids < *max_assignable
                && price_group_amount <= (*max_assignable - total_assigned_bids)
            {
                // FULL ASSIGNMENT
                inner_index = 0;

                while i - inner_index >= k {
                    total_assigned_bids +=
                        self[i - inner_index].fully_assign(clearing_price, day_count, allocations);

                    if i == inner_index {
                        break;
                    }

                    inner_index += 1;
                }

                if inner_index > 0 {
                    j -= inner_index - 1;
                }
            } else if self[i].bid_price_revealed >= *clearing_price
                && total_assigned_bids < *max_assignable
            {
                // PARTIAL ASSIGNMENT
                // Partial assignment for the entire price group
                inner_index = 0;

                while i - inner_index >= k {
                    if i - inner_index == k {
                        // Last iteration of loop. Assign remaining amount left to assign.
                        total_assigned_bids += self[i - inner_index].partially_assign(
                            clearing_price,
                            day_count,
                            &(max_assignable - total_assigned_bids),
                            allocations,
                        );
                        price_group_amount -= max_assignable - total_assigned_bids;
                    } else {
                        // Assign an amount based upon the partial assignment ratio.
                        let assigned_amount: U256 = (self[i - inner_index].amount
                            * (max_assignable - total_assigned_bids))
                            / price_group_amount;

                        total_assigned_bids += self[i - inner_index].partially_assign(
                            clearing_price,
                            day_count,
                            &assigned_amount,
                            allocations,
                        );
                        price_group_amount -= self[i - inner_index].amount;
                    }

                    inner_index += 1;
                }

                if inner_index > 0 {
                    j -= inner_index - 1;
                }
            } else {
                // NO ASSIGNMENT
                self[i].unlock(allocations);
            }

            j -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_assign_bids() {
        unimplemented!()
    }
}
