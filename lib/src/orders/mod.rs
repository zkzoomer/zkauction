pub mod bids;
pub mod offers;

use crate::{allocations::Allocations, exit_tree::ExitLeafTokenWithdrawal, tokens::Tokens};
use alloy_primitives::B256;
use std::collections::BTreeMap;

/// Trait for types that represent onchain chainable orders.
pub trait ChainableSubmissions {
    type T;
    /// Computes an orders hash chain while updating the orders in the provided `orders` mapping with the revealed price information.
    ///
    /// # Arguments
    ///
    /// * `self` - The `T` instance containing all orders placed onchain.
    /// * `hash_function` - A function that computes a 32-byte hash from a byte slice.
    /// * `start_value` - The initial 32-byte value to start the hash chain.
    /// * `orders` - A mutable reference to the `T` BTreeMap where all orders will be updated.
    fn hash_chain<F>(
        &self,
        hash_function: &F,
        start_value: B256,
        orders: &mut BTreeMap<B256, Self::T>,
    ) -> B256
    where
        F: Fn(&[u8]) -> B256;
}

/// Trait for placed orders mappings.
pub trait PlacedOrders: IntoIterator<Item = (B256, Self::Order)> + Sized {
    type OrderSubmission;
    type Allocation;
    type Order: Order;

    /// Saves a new order, updates an existing one, or deletes it from the orders collection.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the `Orders` collection (BTreeMap) to modify.
    /// * `order_submission` - A reference to the `OrderSubmission` containing the order details.
    fn save_or_update_order(&mut self, order_submission: &Self::OrderSubmission);

    /// Validates orders and returns a vector of valid orders, assigning invalid orders to the corresponding allocations.
    ///
    /// # Arguments
    ///
    /// * `orders` - The orders mapping to validate.
    /// * `tokens` - The tokens to check against.
    /// * `allocations` - The allocations to add invalid orders to.
    fn into_validated_orders(
        self,
        tokens: &Tokens,
        allocations: &mut dyn Allocations<Allocation = Self::Allocation, Order = Self::Order>,
    ) -> Vec<Self::Order> {
        let mut valid_orders = Vec::new();

        for (_, order) in self.into_iter() {
            if order.is_valid(tokens) {
                valid_orders.push(order);
            } else {
                allocations.add_from_order(&order);
            }
        }

        valid_orders
    }
}

/// Trait for orders.
pub trait Order {
    type OrderSubmission;
    type OrderReveal;

    /// Creates a new order from an order submission.
    ///
    /// # Arguments
    ///
    /// * `order_submission` - The order submission.
    fn from_order_submission(order_submission: &Self::OrderSubmission) -> Self;

    /// Updates the order with a new order submission.
    ///
    /// # Arguments
    ///
    /// * `self` - The order being updated.
    /// * `order_submission` - The new order submission.
    fn update_from_order_submission(&mut self, order_submission: &Self::OrderSubmission);

    /// Updates the order with revealed information if the reveal is valid.
    ///
    /// # Arguments
    ///
    /// * `self` - The order being updated.
    /// * `hash_function` - A function that computes a 32-byte hash from a byte slice.
    /// * `order_reveal` - The reveal information containing the price and nonce.
    fn update_from_order_reveal<F: Fn(&[u8]) -> B256>(
        &mut self,
        hash_function: &F,
        order_reveal: &Self::OrderReveal,
    );

    /// Returns true if the order is valid and can go to auction matching.
    ///
    /// # Arguments
    ///
    /// * `self` - The order being checked.
    /// * `tokens` - The tokens to check against.
    fn is_valid(&self, tokens: &Tokens) -> bool;

    /// Converts the order to an exit leaf.
    ///
    /// # Arguments
    ///
    /// * `self` - The order being converted.
    /// * `tokens` - The tokens being used in the auction.
    fn to_exit_leaf(&self, tokens: &Tokens) -> ExitLeafTokenWithdrawal;
}

/// Type alias for orders mapping.
pub type Orders<T> = BTreeMap<B256, T>;

pub trait ValidatedOrders: IntoIterator<Item = Self::Order> + Sized {
    type Allocation;
    type Order: Order;

    /// Appropriately sorts the orders by revealed price.
    ///
    /// # Arguments
    ///
    /// * `self` - The orders being sorted.
    fn sort_orders(&mut self);

    /// Dumps all outstanding validated orders into their corresponding allocations.
    ///
    /// # Arguments
    ///
    /// * `self` - The validated orders.
    /// * `allocations` - The allocations to add the orders to.
    fn unlock_outstanding_orders(
        self,
        allocations: &mut dyn Allocations<Allocation = Self::Allocation, Order = Self::Order>,
    ) {
        for order in self.into_iter() {
            allocations.add_from_order(&order);
        }
    }
}
