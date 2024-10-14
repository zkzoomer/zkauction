// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_primitives::Address;
use alloy_sol_types::SolType;
use zkauction_lib::precompiles::sp1_keccak256;
use zkauction_lib::types::bids::{BidReveals, BidSubmissions};
use zkauction_lib::types::offers::{OfferReveals, OfferSubmissions};
use zkauction_lib::types::tokens::Tokens;
use zkauction_lib::{run_auction, types::PublicValuesStruct};

/// The main function of the program, reads the auction inputs, computes the auction results commitment,
/// and commits to the public values of the proof.
pub fn main() {
    // Reading inputs to the program. Behind the scenes, this compiles down to a custom system call
    // which handles reading inputs from the prover.
    // Read the address of the prover
    let prover_address: Address = sp1_zkvm::io::read::<Address>();
    // Read placed orders
    let bid_submissions: BidSubmissions = sp1_zkvm::io::read::<BidSubmissions>();
    let offer_submissions: OfferSubmissions = sp1_zkvm::io::read::<OfferSubmissions>();
    // Read revealed prices
    let bid_reveals: BidReveals = sp1_zkvm::io::read::<BidReveals>();
    let offer_reveals: OfferReveals = sp1_zkvm::io::read::<OfferReveals>();
    // Read token information at the time of proof verification
    let tokens: Tokens = sp1_zkvm::io::read::<Tokens>();

    // Compute public values encoding the auction and its results
    let (acc_bids_hash, acc_offers_hash, token_prices_hash, auction_result_root) = run_auction(
        &sp1_keccak256,
        &prover_address,
        &bid_submissions,
        &offer_submissions,
        &bid_reveals,
        &offer_reveals,
        &tokens,
    );

    // Encode the public values of the program.
    let bytes = PublicValuesStruct::abi_encode(&PublicValuesStruct {
        proverAddress: prover_address,
        accBidsHash: acc_bids_hash,
        accOffersHash: acc_offers_hash,
        tokenPricesHash: token_prices_hash,
        auctionResultRoot: auction_result_root,
    });

    // Commit to the public values of the program. The final proof will have a commitment to all the
    // bytes that were committed to.
    sp1_zkvm::io::commit_slice(&bytes);
}
