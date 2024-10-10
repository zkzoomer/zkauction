pub mod bids;
pub mod exit_tree;
pub mod offers;
pub mod tokens;
pub mod utils;
use alloy_primitives::B256;
use alloy_sol_types::sol;
use exit_tree::{ExitLeaf, ExitLeafWithdrawal, ExitLeaves};
use std::collections::HashMap;

sol! {
    /// The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct PublicValuesStruct {
        /// Address of the prover
        address proverAddress;
        /// Reconstructed hash chain of all bids placed and revealed onchain
        bytes32 accBidsHash;
        /// Reconstructed hash chain of all offers placed and revealed onchain
        bytes32 accOffersHash;
        /// Hashed together information on the tokens involved
        bytes32 tokenPricesHash;
        /// The root of the auction results tree
        bytes32 auctionResultRoot;
    }
}

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
    /// * `orders` - A mutable reference to the `T` HashMap where all orders will be updated.
    fn hash_chain<F>(
        &self,
        hash_function: &F,
        start_value: B256,
        orders: &mut HashMap<B256, Self::T>,
    ) -> B256
    where
        F: Fn(&[u8]) -> B256;
}

/// Trait for types that can be "unrolled" at once into a single `abi.encodePacked`-able value.
pub trait UnrollableStructs {
    /// Computes a single hash value from the implementing type's fields.
    ///
    /// # Arguments
    ///
    /// * `hash_function` - A function that computes a 32-byte hash from a byte slice.
    fn hash_together<F: Fn(&[u8]) -> B256>(&self, hash_function: &F) -> B256;
}

/// Trait for placed orders mappings.
pub trait PlacedOrders: IntoIterator<Item = (B256, Self::Order)> + Sized {
    type OrderSubmission;
    type Order: Order;

    /// Saves a new order, updates an existing one, or deletes it from the orders collection.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the `Orders` collection (HashMap) to modify.
    /// * `order_submission` - A reference to the `OrderSubmission` containing the order details.
    fn save_or_update_order(&mut self, order_submission: &Self::OrderSubmission);

    /// Validates orders and returns a vector of valid orders.
    ///
    /// # Arguments
    ///
    /// * `orders` - The orders mapping to validate.
    /// * `exit_leaves` - The exit leaves to add invalid orders to.
    fn into_validated_orders(self, exit_leaves: &mut ExitLeaves) -> Vec<Self::Order> {
        let mut valid_orders = Vec::new();

        for (_, order) in self.into_iter() {
            if order.is_valid() {
                valid_orders.push(order);
            } else {
                exit_leaves.push(ExitLeaf::Withdrawal(order.to_exit_leaf()));
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
    fn is_valid(&self) -> bool;

    /// Converts the order to an exit leaf.
    ///
    /// # Arguments
    ///
    /// * `self` - The order being converted.
    fn to_exit_leaf(&self) -> ExitLeafWithdrawal;
}

/// Type alias for orders mapping.
pub type Orders<T> = HashMap<B256, T>;

pub trait ValidatedOrders {
    type Order;

    /// Appropriately sorts the orders by revealed price.
    ///
    /// # Arguments
    ///
    /// * `self` - The orders being sorted.
    fn sort_orders(&mut self);
}
