use alloy_primitives::U256;

use crate::{allocations::bidder_allocations::BidderAllocations, orders::bids::ValidatedBids};

use super::Assignable;

impl Assignable for ValidatedBids {
    type Allocations = BidderAllocations;

    fn assign(
        self,
        max_assignable: &U256,
        clearing_price: &U256,
        day_count: &U256,
        allocations: &mut BidderAllocations,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assign_bids() {
        unimplemented!()
    }
}
