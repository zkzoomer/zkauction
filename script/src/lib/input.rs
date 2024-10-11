use alloy_primitives::{aliases::U96, Address, B256, U256};
use sp1_sdk::SP1Stdin;
use zkauction_lib::types::{
    bids::{BidReveal, BidReveals, BidSubmission, BidSubmissions},
    offers::{OfferReveal, OfferReveals, OfferSubmission, OfferSubmissions},
    tokens::Tokens,
};

/// Reads the provided auction inputs and sets them in the provided stdin.
pub fn set_inputs(
    stdin: &mut SP1Stdin,
) -> (
    Address,
    BidSubmissions,
    OfferSubmissions,
    BidReveals,
    OfferReveals,
    Tokens,
) {
    let num_offers: i32 = 1000;

    let prover_address = Address::random();
    let bid_submissions: BidSubmissions = (0..num_offers)
        .map(|_| BidSubmission {
            bidder: Address::random(),
            id: U96::from(rand::random::<u64>()),
            bidPriceHash: B256::random(),
            amount: U256::from(rand::random::<u128>()),
            collateralAmount: U256::from(rand::random::<u128>()),
        })
        .collect();
    let offer_submissions: OfferSubmissions = (0..num_offers)
        .map(|_| OfferSubmission {
            offeror: Address::random(),
            id: U96::from(rand::random::<u64>()),
            offerPriceHash: B256::random(),
            amount: U256::from(rand::random::<u128>()),
        })
        .collect();
    let bid_reveals: BidReveals = (0..num_offers)
        .map(|_| BidReveal {
            orderId: U256::from(rand::random::<u64>()),
            price: U256::from(rand::random::<u128>()),
            nonce: U256::from(rand::random::<u128>()),
        })
        .collect();
    let offer_reveals: OfferReveals = (0..num_offers)
        .map(|_| OfferReveal {
            orderId: U256::from(rand::random::<u64>()),
            price: U256::from(rand::random::<u128>()),
            nonce: U256::from(rand::random::<u128>()),
        })
        .collect();
    let tokens: Tokens = Tokens {
        purchaseToken: Address::random(),
        purchasePrice: U256::from(rand::random::<u64>()),
        collateralToken: Address::random(),
        collateralPrice: U256::from(rand::random::<u64>()),
    };

    stdin.write(&prover_address);
    stdin.write(&bid_submissions);
    stdin.write(&offer_submissions);
    stdin.write(&bid_reveals);
    stdin.write(&offer_reveals);
    stdin.write(&tokens);

    (
        prover_address,
        bid_submissions,
        offer_submissions,
        bid_reveals,
        offer_reveals,
        tokens,
    )
}
