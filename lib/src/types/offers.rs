use super::exit_tree::ExitLeafWithdrawal;
use super::tokens::TokenMap;
use super::utils::{add_to_hash_chain, get_key, get_price_hash};
use super::{ChainableSubmissions, Order, PlacedOrders, ValidatedOrders};
use crate::constants::MAX_OFFER_PRICE;
use alloy_primitives::{aliases::U96, Address, B256, U256};
use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents an offer to lend an amount of money for a specific interest rate.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Offer {
    /// Unique identifier for the offer, combined with `offeror` to form a complete key.
    pub id: U96,
    /// Ethereum address of the offeror (lender).
    pub offeror: Address,
    /// Keccak-256 hash of the offer price and a nonce, enabling the blind auction process.
    pub offer_price_hash: B256,
    /// The actual offer price revealed during the reveal phase, initially zero.
    pub offer_price_revealed: U256,
    /// Maximum amount of purchase tokens that can be lent.
    pub amount: U256,
    /// Address of the ERC20 token to be lent (purchase token).
    pub purchase_token: Address,
    /// Indicates whether the offer has been revealed in the reveal phase.
    pub is_revealed: bool,
}

impl Order for Offer {
    type OrderSubmission = OfferSubmission;
    type OrderReveal = OfferReveal;

    fn from_order_submission(offer_submission: &OfferSubmission) -> Self {
        Self {
            id: offer_submission.id,
            offeror: offer_submission.offeror,
            offer_price_hash: offer_submission.offerPriceHash,
            offer_price_revealed: U256::ZERO,
            amount: offer_submission.amount,
            purchase_token: offer_submission.purchaseToken,
            is_revealed: false,
        }
    }

    fn update_from_order_submission(&mut self, offer_submission: &OfferSubmission) {
        self.amount = offer_submission.amount;
        self.offer_price_hash = offer_submission.offerPriceHash;
    }

    fn update_from_order_reveal<F: Fn(&[u8]) -> B256>(
        &mut self,
        hash_function: &F,
        offer_reveal: &OfferReveal,
    ) {
        if get_price_hash(hash_function, &offer_reveal.price, &offer_reveal.nonce)
            == self.offer_price_hash
            && offer_reveal.price <= U256::from(MAX_OFFER_PRICE)
        {
            self.offer_price_revealed = offer_reveal.price;
            self.is_revealed = true;
        }
    }

    fn is_valid(&self, _token_map: &TokenMap) -> bool {
        self.is_revealed
    }

    fn to_exit_leaf(&self) -> ExitLeafWithdrawal {
        ExitLeafWithdrawal {
            recipient: self.offeror,
            token: self.purchase_token,
            amount: self.amount,
        }
    }
}

/// A collection of all offers, indexed by their unique keys.
///
/// # Key
/// The key is a `B256` (32-byte) value, created by concatenating:
/// - The offeror's Ethereum address (20 bytes)
/// - The offer's unique ID (12 bytes)
///
/// # Value
/// The value is a `Offer` struct, containing all details of the offer.
pub type Offers = BTreeMap<B256, Offer>;

impl PlacedOrders for Offers {
    type OrderSubmission = OfferSubmission;
    type Order = Offer;

    /// # Behavior
    ///
    /// - If the offer's amount is zero, the offer is removed from the collection.
    /// - If an offer with the same key already exists, it is updated with the new submission details.
    /// - If no offer exists for the key, a new `Offer` instance is created and inserted.
    fn save_or_update_order(&mut self, order_submission: &OfferSubmission) {
        let key: B256 = get_key(&order_submission.offeror, &order_submission.id);
        if order_submission.amount.is_zero() {
            self.remove(&key);
        } else {
            self.entry(key)
                .and_modify(|existing_offer: &mut Offer| {
                    existing_offer.update_from_order_submission(order_submission);
                })
                .or_insert_with(|| Offer::from_order_submission(order_submission));
        }
    }
}

sol! {
    /// An `OfferSubmission` represents an offer submission to lend an amount of money for a specific interest rate
    #[derive(Serialize, Deserialize)]
    struct OfferSubmission {
        /// The address of the offeror
        address offeror;
        /// Defines, alongside the `offeror`, a unique identifier for the offer
        uint96 id;
        /// Hash of the offered price as a percentage of the initial loaned amount vs amount returned at maturity. This stores 9 decimal places
        bytes32 offerPriceHash;
        /// The maximum amount of purchase tokens that can be lent
        uint256 amount;
        /// The address of the ERC20 purchase token
        address purchaseToken;
    }
}

/// Represents the history of all offer submissions made onchain.
pub type OfferSubmissions = Vec<OfferSubmission>;

impl ChainableSubmissions for OfferSubmissions {
    type T = Offer;
    /// # Behavior
    ///
    /// - If an offer with the same key already exists, it updates the amount and offer price hash.
    /// - If no offer exists for the key, it creates a new `Offer` instance with the provided details.
    fn hash_chain<F>(&self, hash_function: &F, start_value: B256, offers: &mut Offers) -> B256
    where
        F: Fn(&[u8]) -> B256,
    {
        self.iter().fold(
            start_value,
            |acc: B256, offer_submission: &OfferSubmission| {
                offers.save_or_update_order(offer_submission);
                add_to_hash_chain(hash_function, offer_submission, &acc)
            },
        )
    }
}

sol! {
    /// An `OfferReveal` represents the offer reveal process that was carried out onchain
    #[derive(Serialize, Deserialize)]
    struct OfferReveal {
        /// The ID of the offer that was revealed
        uint256 orderId;
        /// The price of the offer that was revealed
        uint256 price;
        /// Nonce value that was used to generate the offer price hash
        uint256 nonce;
    }
}

/// Represents the history of all offer reveals made onchain.
pub type OfferReveals = Vec<OfferReveal>;

impl ChainableSubmissions for OfferReveals {
    type T = Offer;
    /// # Behavior
    ///
    /// - If a offer with the matching `orderId` is found and the calculated price hash
    ///   matches the stored hash:
    ///   - Updates the `offer_price_revealed` with the revealed price.
    ///   - Sets `is_revealed` to `true`.
    /// - If no matching offer is found or the price hash doesn't match, no changes are made.
    fn hash_chain<F>(&self, hash_function: &F, start_value: B256, offers: &mut Offers) -> B256
    where
        F: Fn(&[u8]) -> B256,
    {
        self.iter()
            .fold(start_value, |acc: B256, item: &OfferReveal| {
                // Set offer price if it was revealed properly
                if let Some(offer) = offers.get_mut::<B256>(&item.orderId.into()) {
                    offer.update_from_order_reveal(hash_function, item);
                }
                // Add value to hash chain
                add_to_hash_chain(hash_function, item, &acc)
            })
    }
}

/// A collection of all validated offers.
pub type ValidatedOffers = Vec<Offer>;

impl ValidatedOrders for ValidatedOffers {
    type Order = Offer;

    fn sort_orders(&mut self) {
        self.sort_by(|a: &Offer, b: &Offer| a.offer_price_revealed.cmp(&b.offer_price_revealed));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        exit_tree::{ExitLeaf, ExitLeaves},
        utils::test::calculate_expected_hash_chain_output,
    };
    use alloy_primitives::keccak256;

    #[test]
    fn test_offer_from_order_submission() {
        let offer_submission: OfferSubmission = random_offer_submission();

        let offer: Offer = Offer::from_order_submission(&offer_submission);
        assert_eq!(offer.offeror, offer_submission.offeror);
        assert_eq!(offer.id, offer_submission.id);
        assert_eq!(offer.offer_price_hash, offer_submission.offerPriceHash);
        assert_eq!(offer.amount, offer_submission.amount);
        assert_eq!(offer.purchase_token, offer_submission.purchaseToken);
    }

    #[test]
    fn test_offer_update_from_order_submission() {
        let offer_submission: OfferSubmission = random_offer_submission();

        let mut offer: Offer = Offer::from_order_submission(&offer_submission);
        let new_order_submission: OfferSubmission = random_offer_submission();

        offer.update_from_order_submission(&new_order_submission);
        assert_eq!(offer.amount, new_order_submission.amount);
        assert_eq!(offer.offer_price_hash, new_order_submission.offerPriceHash);
    }

    #[test]
    fn test_offer_update_from_order_reveal() {
        // Valid reveal
        let price: U256 = U256::from(rand::random::<u64>() % crate::constants::MAX_OFFER_PRICE);
        let nonce: U256 = U256::from(rand::random::<u128>());
        let offer_submission: OfferSubmission = valid_random_offer_submission(&price, &nonce);
        let mut offer: Offer = Offer::from_order_submission(&offer_submission);
        offer.update_from_order_reveal(
            &|x| keccak256(x),
            &OfferReveal {
                orderId: get_key(&offer_submission.offeror, &offer_submission.id).into(),
                price,
                nonce,
            },
        );
        assert_eq!(offer.offer_price_revealed, price);
        assert!(offer.is_revealed);

        // Invalid reveal
        let mut offer = Offer::from_order_submission(&offer_submission);
        offer.update_from_order_reveal(
            &|x: &[u8]| keccak256(x),
            &OfferReveal {
                orderId: get_key(&offer_submission.offeror, &offer_submission.id).into(),
                price: U256::from(rand::random::<u128>()),
                nonce: U256::from(rand::random::<u128>()),
            },
        );
        assert_eq!(offer.offer_price_revealed, U256::ZERO);
        assert!(!offer.is_revealed);

        // Invalid reveal with out of bounds price
        let price: U256 = U256::from(crate::constants::MAX_OFFER_PRICE + 1);
        let nonce: U256 = U256::from(rand::random::<u128>());
        let offer_submission: OfferSubmission = valid_random_offer_submission(&price, &nonce);
        let mut offer = Offer::from_order_submission(&offer_submission);
        offer.update_from_order_reveal(
            &|x: &[u8]| keccak256(x),
            &OfferReveal {
                orderId: get_key(&offer_submission.offeror, &offer_submission.id).into(),
                price,
                nonce,
            },
        );
        assert_eq!(offer.offer_price_revealed, U256::ZERO);
        assert!(!offer.is_revealed);
    }

    #[test]
    fn test_offer_is_valid() {
        let mut offer: Offer = random_revealed_offer();
        offer.is_revealed = true;
        assert!(offer.is_valid(&TokenMap::new()));

        offer.is_revealed = false;
        assert!(!offer.is_valid(&TokenMap::new()));
    }

    #[test]
    fn test_offer_to_exit_leaf() {
        let offer: Offer = random_revealed_offer();
        let exit_leaf = offer.to_exit_leaf();

        assert_eq!(exit_leaf.recipient, offer.offeror);
        assert_eq!(exit_leaf.token, offer.purchase_token);
        assert_eq!(exit_leaf.amount, offer.amount);
    }

    #[test]
    fn test_save_or_update_offer() {
        let mut offers: Offers = Offers::new();
        let mut offer_submission: OfferSubmission = random_offer_submission();

        // Saves the offer if new
        offers.save_or_update_order(&offer_submission);

        let offer: Offer = Offer::from_order_submission(&offer_submission);
        assert_eq!(offers.len(), 1);
        offer_eq(
            &offer,
            offers
                .get(&get_key(&offer_submission.offeror, &offer_submission.id))
                .unwrap(),
        );

        // Updates the offer if it already exists
        offer_submission.offerPriceHash = B256::random();
        offer_submission.amount = U256::from(rand::random::<u128>());
        offers.save_or_update_order(&offer_submission);

        let offer: Offer = Offer::from_order_submission(&offer_submission);
        assert_eq!(offers.len(), 1);
        offer_eq(
            &offer,
            offers
                .get(&get_key(&offer_submission.offeror, &offer_submission.id))
                .unwrap(),
        );

        // Deletes the offer if amount is zero
        offer_submission.amount = U256::ZERO;
        offers.save_or_update_order(&offer_submission);
        assert_eq!(offers.len(), 0);
    }

    #[test]
    fn test_order_submissions_hash_chain() {
        // Random values
        let start_value: B256 = B256::random();
        let mut expected_offers: Offers = Offers::new();
        let offer_submissions: OfferSubmissions = (0..42)
            .map(|_| {
                let offer_submission: OfferSubmission = random_offer_submission();
                expected_offers.save_or_update_order(&offer_submission);
                offer_submission
            })
            .collect();
        let expected_output: B256 =
            calculate_expected_hash_chain_output(&start_value, &offer_submissions);

        let mut offers: Offers = Offers::new();
        let output: B256 =
            offer_submissions.hash_chain(&|x: &[u8]| keccak256(x), start_value, &mut offers);

        assert_eq!(expected_output, output);
        assert_eq!(expected_offers, offers);
    }

    #[test]
    fn test_order_reveals_hash_chain() {
        // Random values
        let start_value: B256 = B256::random();
        let mut expected_offers: Offers = Offers::new();
        let mut offer_reveals: OfferReveals = OfferReveals::new();
        let offer_submissions: OfferSubmissions = (0..42)
            .map(|_| {
                let price: U256 =
                    U256::from(rand::random::<u64>() % crate::constants::MAX_OFFER_PRICE);
                let nonce: U256 = U256::from(rand::random::<u128>());
                let offer_submission: OfferSubmission =
                    valid_random_offer_submission(&price, &nonce);
                expected_offers.save_or_update_order(&offer_submission);
                offer_reveals.push(OfferReveal {
                    orderId: get_key(&offer_submission.offeror, &offer_submission.id).into(),
                    price,
                    nonce,
                });
                offer_submission
            })
            .collect();
        offer_reveals.iter().for_each(|offer_reveal: &OfferReveal| {
            if let Some(offer) = expected_offers.get_mut::<B256>(&offer_reveal.orderId.into()) {
                offer.update_from_order_reveal(&|x: &[u8]| keccak256(x), offer_reveal);
            }
        });
        let mut expected_output: B256 =
            calculate_expected_hash_chain_output(&start_value, &offer_submissions);
        expected_output = calculate_expected_hash_chain_output(&expected_output, &offer_reveals);

        let mut offers: Offers = Offers::new();
        let mut output: B256 =
            offer_submissions.hash_chain(&|x: &[u8]| keccak256(x), start_value, &mut offers);
        output = offer_reveals.hash_chain(&|x: &[u8]| keccak256(x), output, &mut offers);

        assert_eq!(expected_output, output);
        assert_eq!(expected_offers, offers);
    }

    #[test]
    fn test_validate_offers() {
        let mut placed_offers: Offers = Offers::new();
        let mut exit_leaves: ExitLeaves = ExitLeaves::new();
        let revealed_offer: Offer = random_revealed_offer();
        let non_revealed_offer: Offer = random_non_revealed_offer();

        placed_offers.insert(
            get_key(&revealed_offer.offeror, &revealed_offer.id),
            revealed_offer.clone(),
        );
        placed_offers.insert(
            get_key(&non_revealed_offer.offeror, &non_revealed_offer.id),
            non_revealed_offer.clone(),
        );

        let validated_offers =
            placed_offers.into_validated_orders(&TokenMap::new(), &mut exit_leaves);

        assert_eq!(validated_offers.len(), 1);
        assert_eq!(exit_leaves.len(), 1);
        assert_eq!(validated_offers[0], revealed_offer);
        assert_eq!(
            exit_leaves[0],
            ExitLeaf::Withdrawal(non_revealed_offer.to_exit_leaf())
        );
    }

    #[test]
    fn test_validated_offers_sort_orders() {
        let mut offers: ValidatedOffers = vec![
            random_revealed_offer(),
            random_revealed_offer(),
            random_revealed_offer(),
        ];
        offers.sort_orders();
        assert!(offers[0].offer_price_revealed <= offers[1].offer_price_revealed);
        assert!(offers[1].offer_price_revealed <= offers[2].offer_price_revealed);
    }

    // HELPER FUNCTIONS
    /// Creates a new OfferSubmission with random values for testing purposes.
    fn random_offer_submission() -> OfferSubmission {
        OfferSubmission {
            offeror: Address::random(),
            id: U96::from(rand::random::<u64>()),
            offerPriceHash: B256::random(),
            amount: U256::from(rand::random::<u128>()),
            purchaseToken: Address::random(),
        }
    }

    /// Creates a random OfferSubmission with a valid offer price hash for the given price and nonce.
    fn valid_random_offer_submission(price: &U256, nonce: &U256) -> OfferSubmission {
        OfferSubmission {
            offeror: Address::random(),
            id: U96::from(rand::random::<u64>()),
            offerPriceHash: get_price_hash(&|x| keccak256(x), price, nonce),
            amount: U256::from(rand::random::<u128>()),
            purchaseToken: Address::random(),
        }
    }

    /// Creates a random revealed Offer.
    fn random_revealed_offer() -> Offer {
        Offer {
            id: U96::from(rand::random::<u64>()),
            offeror: Address::random(),
            offer_price_hash: B256::random(),
            offer_price_revealed: U256::from(
                rand::random::<u64>() % crate::constants::MAX_OFFER_PRICE,
            ),
            amount: U256::from(rand::random::<u128>()),
            purchase_token: Address::random(),
            is_revealed: true,
        }
    }

    /// Creates a random non-revealed Offer.
    fn random_non_revealed_offer() -> Offer {
        Offer {
            id: U96::from(rand::random::<u64>()),
            offeror: Address::random(),
            offer_price_hash: B256::random(),
            offer_price_revealed: U256::ZERO,
            amount: U256::from(rand::random::<u128>()),
            purchase_token: Address::random(),
            is_revealed: false,
        }
    }

    /// Compares two Offer structs for equality, asserting that all fields match.
    fn offer_eq(offer_expected: &Offer, offer: &Offer) {
        assert_eq!(offer_expected.offeror, offer.offeror);
        assert_eq!(offer_expected.id, offer.id);
        assert_eq!(offer_expected.offer_price_hash, offer.offer_price_hash);
        assert_eq!(
            offer_expected.offer_price_revealed,
            offer.offer_price_revealed
        );
        assert_eq!(offer_expected.amount, offer.amount);
        assert_eq!(offer_expected.purchase_token, offer.purchase_token);
        assert_eq!(offer_expected.is_revealed, offer.is_revealed);
    }
}
