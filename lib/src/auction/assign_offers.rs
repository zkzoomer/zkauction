use alloy_primitives::U256;

use crate::{
    allocations::offeror_allocations::OfferorAllocations, orders::offers::ValidatedOffers,
};

use super::{find_last_index_for_price, Assignable};

impl Assignable for ValidatedOffers {
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
        let mut inner_index: usize = 0;
        let mut i: usize = 0;

        while i < self.len() {
            // First, find the sub-range that contains the current price.
            let (k, price_group_amount) =
                find_last_index_for_price(&self[i].offer_price_revealed, &self, &i);
            // NOTE: price_group_amount gets changed later on in this function and is used as the "remaining" price_group_amount during partial assignment.

            if self[i].offer_price_revealed <= *clearing_price
                && total_assigned_offers < *max_assignable
                && price_group_amount < (*max_assignable - total_assigned_offers)
            {
                // FULL ASSIGNMENT
            } else if self[i].offer_price_revealed <= *clearing_price
                && total_assigned_offers < *max_assignable
            {
                // PARTIAL ASSIGNMENT
            } else {
                // NO ASSIGNMENT
                // Purchase tokens are returned to the offeror
            }

            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assign_offers() {
        unimplemented!()
    }
}
