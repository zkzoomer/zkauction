use alloy_primitives::{aliases::U96, Address, B256, U256};
use sp1_sdk::SP1Stdin;
use zkauction_lib::{BidReveal, BidSubmission, OfferReveal, OfferSubmission, TokenInformation};

type BidSubmissions = Vec<BidSubmission>;
type OfferSubmissions = Vec<OfferSubmission>;
type BidReveals = Vec<BidReveal>;
type OfferReveals = Vec<OfferReveal>;
type Tokens = Vec<TokenInformation>;

/// Reads the provided auction inputs and sets them in the provided stdin.
pub fn set_inputs(
    stdin: &mut SP1Stdin,
) -> (
    BidSubmissions,
    OfferSubmissions,
    BidReveals,
    OfferReveals,
    Tokens,
) {
    let num_offers: i32 = 10;
    let num_tokens: i32 = 2;

    let bids: BidSubmissions = (0..num_offers)
        .map(|_| BidSubmission {
            bidder: Address::random(),
            id: U96::from(rand::random::<u64>()),
            bidPriceHash: B256::random(),
            amount: U256::from(rand::random::<u128>()),
            collateralAmount: U256::from(rand::random::<u128>()),
            purchaseToken: Address::random(),
            collateralToken: Address::random(),
        })
        .collect();
    let offers: OfferSubmissions = (0..num_offers)
        .map(|_| OfferSubmission {
            offeror: Address::random(),
            id: U96::from(rand::random::<u64>()),
            offerPriceHash: B256::random(),
            amount: U256::from(rand::random::<u128>()),
            purchaseToken: Address::random(),
        })
        .collect();
    let revealed_bids: BidReveals = (0..num_offers)
        .map(|_| BidReveal {
            orderId: U256::from(rand::random::<u64>()),
            price: U256::from(rand::random::<u128>()),
            nonce: U256::from(rand::random::<u128>()),
        })
        .collect();
    let revealed_offers: OfferReveals = (0..num_offers)
        .map(|_| OfferReveal {
            orderId: U256::from(rand::random::<u64>()),
            price: U256::from(rand::random::<u128>()),
            nonce: U256::from(rand::random::<u128>()),
        })
        .collect();
    let tokens: Tokens = (0..num_tokens)
        .map(|_| TokenInformation {
            tokenAddress: Address::random(),
            price: U256::from(rand::random::<u128>()),
        })
        .collect();

    stdin.write(&bids);
    stdin.write(&offers);
    stdin.write(&revealed_bids);
    stdin.write(&revealed_offers);
    stdin.write(&tokens);

    (bids, offers, revealed_bids, revealed_offers, tokens)
}
