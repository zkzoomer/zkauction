use alloy_primitives::U256;

use crate::{
    allocations::{offeror_allocations::OfferorAllocations, Allocations},
    orders::offers::{Offer, ValidatedOffers},
};

use super::{
    calculate_repurchase_price, find_last_index_for_price, AssignableOrder, AssignableOrders,
};

impl AssignableOrder for Offer {
    type Allocations = OfferorAllocations;

    fn fully_assign(
        &self,
        clearing_price: &U256,
        day_count: &U256,
        offeror_allocations: &mut OfferorAllocations,
    ) -> U256 {
        let repurchase_amount: U256 =
            calculate_repurchase_price(&self.amount, clearing_price, day_count);

        offeror_allocations
            .get_allocation(&self.offeror)
            .update_repo_amount(repurchase_amount);

        self.amount
    }

    fn partially_assign(
        &self,
        clearing_price: &U256,
        day_count: &U256,
        assigned_amount: &U256,
        offeror_allocations: &mut OfferorAllocations,
    ) -> U256 {
        let repurchase_amount: U256 =
            calculate_repurchase_price(assigned_amount, clearing_price, day_count);

        let offeror_allocation = offeror_allocations.get_allocation(&self.offeror);
        offeror_allocation.update_repo_amount(repurchase_amount);
        offeror_allocation.update_purchase_amount(self.amount - assigned_amount);

        *assigned_amount
    }

    fn unlock(&self, offeror_allocations: &mut OfferorAllocations) {
        offeror_allocations.add_from_order(self);
    }
}

impl AssignableOrders for ValidatedOffers {
    type Allocations = OfferorAllocations;

    fn assign(
        self,
        max_assignable: &U256,
        clearing_price: &U256,
        day_count: &U256,
        allocations: &mut OfferorAllocations,
    ) {
        // Process revealed offers
        let mut total_assigned_offers: U256 = U256::ZERO;
        let mut inner_index: usize;
        let mut i: usize = 0;

        while i < self.len() {
            // First, find the sub-range that contains the current price.
            let (k, mut price_group_amount) =
                find_last_index_for_price(&self[i].offer_price_revealed, &self, &i);
            // NOTE: price_group_amount gets changed later on in this function and is used as the "remaining" price_group_amount during partial assignment.

            if self[i].offer_price_revealed <= *clearing_price
                && total_assigned_offers < *max_assignable
                && price_group_amount < (*max_assignable - total_assigned_offers)
            {
                // FULL ASSIGNMENT
                inner_index = 0;

                while inner_index + i < k {
                    total_assigned_offers +=
                        self[inner_index + i].fully_assign(clearing_price, day_count, allocations);

                    inner_index += 1;
                }

                if inner_index > 0 {
                    i += inner_index - 1;
                }
            } else if self[i].offer_price_revealed <= *clearing_price
                && total_assigned_offers < *max_assignable
            {
                // PARTIAL ASSIGNMENT
                let mut inner_index: usize = 0;
                while inner_index + i < k {
                    if inner_index + i == k {
                        // Last iteration of loop. Assign remaining amount left to assign.
                        total_assigned_offers += self[inner_index + i].partially_assign(
                            clearing_price,
                            day_count,
                            &(max_assignable - total_assigned_offers),
                            allocations,
                        );
                        price_group_amount -= max_assignable - total_assigned_offers;
                    } else {
                        // Assign an amount based upon the partial assignment ratio.
                        let assigned_amount: U256 = if inner_index + i != k {
                            (self[inner_index + i].amount
                                * (max_assignable - total_assigned_offers))
                                / price_group_amount
                        } else {
                            max_assignable - total_assigned_offers
                        };

                        total_assigned_offers += self[inner_index + i].partially_assign(
                            clearing_price,
                            day_count,
                            &assigned_amount,
                            allocations,
                        );
                        price_group_amount -= self[inner_index + i].amount;
                    }
                    inner_index += 1;
                }
                if inner_index > 0 {
                    i += inner_index - 1;
                }
            } else {
                // NO ASSIGNMENT
                self[i].unlock(allocations);
            }

            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_assign_offers() {
        unimplemented!()
    }
}
