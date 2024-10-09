use super::utils::{add_to_hash_chain, get_key, get_price_hash};
use super::ChainableOrders;
use crate::constants::MAX_OFFER_PRICE;
use alloy_primitives::{aliases::U96, Address, B256, U256};
use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an offer to lend an amount of money for a specific interest rate.
#[derive(PartialEq, Eq, Debug)]
pub struct Offer {
    /// Unique identifier for the offer, combined with `offeror` to form a complete key.
    id: U96,
    /// Ethereum address of the offeror (lender).
    offeror: Address,
    /// Keccak-256 hash of the offer price and a nonce, enabling the blind auction process.
    offer_price_hash: B256,
    /// The actual offer price revealed during the reveal phase, initially zero.
    offer_price_revealed: U256,
    /// Maximum amount of purchase tokens that can be lent.
    amount: U256,
    /// Address of the ERC20 token to be lent (purchase token).
    purchase_token: Address,
    /// Indicates whether the offer has been revealed in the reveal phase.
    is_revealed: bool,
}

impl Offer {
    /// Creates a new offer from an offer submission.
    ///
    /// # Arguments
    ///
    /// * `offer_submission` - The offer submission.
    pub fn from_offer_submission(offer_submission: &OfferSubmission) -> Self {
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

    /// Updates the offer with a new offer submission.
    ///
    /// # Arguments
    ///
    /// * `self` - The offer being updated.
    /// * `offer_submission` - The new offer submission.
    pub fn update_with_offer_submission(&mut self, offer_submission: &OfferSubmission) {
        self.amount = offer_submission.amount;
        self.offer_price_hash = offer_submission.offerPriceHash;
    }

    /// Updates the offer with a revealed price if the reveal is valid.
    ///
    /// # Arguments
    ///
    /// * `self` - The offer being updated.
    /// * `hash_function` - A function that computes a 32-byte hash from a byte slice.
    /// * `offer_reveal` - The reveal information containing the price and nonce.
    pub fn update_from_offer_reveal<F: Fn(&[u8]) -> B256>(
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
pub type Offers = HashMap<B256, Offer>;

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

impl ChainableOrders for OfferSubmissions {
    type T = Offer;
    /// # Behavior
    ///
    /// - If an offer with the same key already exists, it updates the amount and offer price hash.
    /// - If no offer exists for the key, it creates a new `Offer` instance with the provided details.
    fn hash_chain<F>(&self, hash_function: &F, start_value: B256, offers: &mut Offers) -> B256
    where
        F: Fn(&[u8]) -> B256,
    {
        self.iter()
            .fold(start_value, |acc: B256, item: &OfferSubmission| {
                save_or_update_offer(offers, item);
                // Add value to hash chain
                add_to_hash_chain(hash_function, item, &acc)
            })
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

impl ChainableOrders for OfferReveals {
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
                    offer.update_from_offer_reveal(hash_function, item);
                }
                // Add value to hash chain
                add_to_hash_chain(hash_function, item, &acc)
            })
    }
}

/// Saves a new offer, updates an existing one, or deletes it from the offers collection.
///
/// # Arguments
///
/// * `offers` - A mutable reference to the `Offers` collection (HashMap) to modify.
/// * `offer_submission` - A reference to the `OfferSubmission` containing the offer details.
///
/// # Behavior
///
/// - If the offer's amount is zero, the offer is removed from the collection.
/// - If an offer with the same key already exists, it is updated with the new submission details.
/// - If no offer exists for the key, a new `Offer` instance is created and inserted.
fn save_or_update_offer(offers: &mut Offers, offer_submission: &OfferSubmission) {
    let key: B256 = get_key(&offer_submission.offeror, &offer_submission.id);
    if offer_submission.amount.is_zero() {
        offers.remove(&key);
    } else {
        offers
            .entry(key)
            .and_modify(|existing_offer: &mut Offer| {
                existing_offer.update_with_offer_submission(offer_submission);
            })
            .or_insert_with(|| Offer::from_offer_submission(offer_submission));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::utils::test::calculate_expected_hash_chain_output;
    use alloy_primitives::keccak256;

    #[test]
    fn test_offer_from_offer_submission() {
        let offer_submission: OfferSubmission = random_offer_submission();

        let offer: Offer = Offer::from_offer_submission(&offer_submission);
        assert_eq!(offer.offeror, offer_submission.offeror);
        assert_eq!(offer.id, offer_submission.id);
        assert_eq!(offer.offer_price_hash, offer_submission.offerPriceHash);
        assert_eq!(offer.amount, offer_submission.amount);
        assert_eq!(offer.purchase_token, offer_submission.purchaseToken);
    }

    #[test]
    fn test_offer_update_from_offer_submission() {
        let offer_submission: OfferSubmission = random_offer_submission();

        let mut offer: Offer = Offer::from_offer_submission(&offer_submission);
        let new_offer_submission: OfferSubmission = random_offer_submission();

        offer.update_with_offer_submission(&new_offer_submission);
        assert_eq!(offer.amount, new_offer_submission.amount);
        assert_eq!(offer.offer_price_hash, new_offer_submission.offerPriceHash);
    }

    #[test]
    fn test_offer_update_from_offer_reveal() {
        // Valid reveal
        let price: U256 = U256::from(rand::random::<u64>() % crate::constants::MAX_OFFER_PRICE);
        let nonce: U256 = U256::from(rand::random::<u128>());
        let offer_submission: OfferSubmission = valid_random_offer_submission(&price, &nonce);
        let mut offer: Offer = Offer::from_offer_submission(&offer_submission);
        offer.update_from_offer_reveal(
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
        let mut offer = Offer::from_offer_submission(&offer_submission);
        offer.update_from_offer_reveal(
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
        let mut offer = Offer::from_offer_submission(&offer_submission);
        offer.update_from_offer_reveal(
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
    fn test_save_or_update_offer() {
        let mut offers: Offers = Offers::new();
        let mut offer_submission: OfferSubmission = random_offer_submission();

        // Saves the offer if new
        save_or_update_offer(&mut offers, &offer_submission);

        let offer: Offer = Offer::from_offer_submission(&offer_submission);
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
        save_or_update_offer(&mut offers, &offer_submission);

        let offer: Offer = Offer::from_offer_submission(&offer_submission);
        assert_eq!(offers.len(), 1);
        offer_eq(
            &offer,
            offers
                .get(&get_key(&offer_submission.offeror, &offer_submission.id))
                .unwrap(),
        );

        // Deletes the offer if amount is zero
        offer_submission.amount = U256::ZERO;
        save_or_update_offer(&mut offers, &offer_submission);
        assert_eq!(offers.len(), 0);
    }

    #[test]
    fn test_offer_submissions_hash_chain() {
        // Random values
        let start_value: B256 = B256::random();
        let mut expected_offers: Offers = Offers::new();
        let offer_submissions: OfferSubmissions = (0..42)
            .map(|_| {
                let offer_submission: OfferSubmission = random_offer_submission();
                save_or_update_offer(&mut expected_offers, &offer_submission);
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
    fn test_offer_reveals_hash_chain() {
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
                save_or_update_offer(&mut expected_offers, &offer_submission);
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
                offer.update_from_offer_reveal(&|x: &[u8]| keccak256(x), offer_reveal);
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
