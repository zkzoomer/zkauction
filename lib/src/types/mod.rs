pub mod allocations;
pub mod bidder_allocations;
pub mod bids;
pub mod exit_tree;
pub mod offeror_allocations;
pub mod offers;
pub mod tokens;
pub mod utils;
use allocations::Allocations;
use alloy_primitives::B256;
use alloy_sol_types::{sol, SolValue};
use exit_tree::ExitLeafTokenWithdrawal;
use std::collections::BTreeMap;
use tokens::Tokens;

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

/// Trait for Solidity structs that can be hashed via first calling `abi.encodePacked`.
pub trait HashableStruct: SolValue {
    /// Computes a single hash value from the struct's fields by first calling `abi.encodePacked`.
    ///
    /// # Arguments
    ///
    /// * `self` - The struct to hash.
    /// * `hash_function` - A function that computes a 32-byte hash from a byte slice.
    fn hash<F: Fn(&[u8]) -> B256>(&self, hash_function: &F) -> B256 {
        hash_function(&self.abi_encode_packed())
    }
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

pub trait ValidatedOrders {
    type Order;

    /// Appropriately sorts the orders by revealed price.
    ///
    /// # Arguments
    ///
    /// * `self` - The orders being sorted.
    fn sort_orders(&mut self);
}
