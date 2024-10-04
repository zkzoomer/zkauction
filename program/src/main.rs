// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use zkauction_lib::precompiles::sp1_keccak256;
use zkauction_lib::{
    run_auction, BidReveal, BidSubmission, OfferReveal, OfferSubmission, PublicValuesStruct,
    TokenInformation,
};

/// The main function of the program, reads the auction inputs, computes the auction results commitment,
/// and commits to the public values of the proof.
pub fn main() {
    // Reading inputs to the program. Behind the scenes, this compiles down to a custom system call
    // which handles reading inputs from the prover.
    // Read placed orders
    let bids: Vec<BidSubmission> = sp1_zkvm::io::read::<Vec<BidSubmission>>();
    let offers: Vec<OfferSubmission> = sp1_zkvm::io::read::<Vec<OfferSubmission>>();
    // Read revealed prices
    let revealed_bids: Vec<BidReveal> = sp1_zkvm::io::read::<Vec<BidReveal>>();
    let revealed_offers: Vec<OfferReveal> = sp1_zkvm::io::read::<Vec<OfferReveal>>();
    // Read token information at the time of proof verification
    let tokens: Vec<TokenInformation> = sp1_zkvm::io::read::<Vec<TokenInformation>>();

    // Compute public values encoding the auction and its results
    let (acc_bids_hash, acc_offers_hash, tokens_hash, auction_result_root) = run_auction(
        &sp1_keccak256,
        bids,
        offers,
        revealed_bids,
        revealed_offers,
        tokens,
    );

    // Encode the public values of the program.
    let bytes = PublicValuesStruct::abi_encode(&PublicValuesStruct {
        accBidsHash: acc_bids_hash,
        accOffersHash: acc_offers_hash,
        tokensHash: tokens_hash,
        auctionResultRoot: auction_result_root,
    });

    // Commit to the public values of the program. The final proof will have a commitment to all the
    // bytes that were committed to.
    sp1_zkvm::io::commit_slice(&bytes);
}
