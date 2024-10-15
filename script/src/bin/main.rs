//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use alloy_primitives::keccak256;
use alloy_sol_types::SolType;
use clap::Parser;
use sp1_sdk::{ProverClient, SP1Stdin};
use zkauction_lib::orders::PublicValuesStruct;

// Adjust this path based on the actual location of input.rs
#[path = "../lib/input.rs"]
mod input;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const ZK_AUCTION_ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,

    #[clap(long)]
    prove: bool,

    #[clap(long, default_value = "20")]
    n: u32,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    let (_prover_address, bid_submissions, offer_submissions, bid_reveals, offer_reveals, tokens) =
        input::set_inputs(&mut stdin);

    if args.execute {
        // Execute the program
        let (output, report) = client.execute(ZK_AUCTION_ELF, stdin).run().unwrap();
        println!("Program executed successfully.");

        // Read the output.
        let decoded = PublicValuesStruct::abi_decode(output.as_slice(), true).unwrap();
        let PublicValuesStruct {
            proverAddress: prover_address,
            accBidsHash: acc_bids_hash,
            accOffersHash: acc_offers_hash,
            tokenPricesHash: token_prices_hash,
            auctionResultRoot: auction_result_root,
        } = decoded;
        println!("proverAddress: {}", prover_address);
        println!("accBidsHash: {}", acc_bids_hash);
        println!("accOffersHash: {}", acc_offers_hash);
        println!("tokenPricesHash: {}", token_prices_hash);
        println!("auctionResultRoot: {}", auction_result_root);

        let (
            expected_acc_bids_hash,
            expected_acc_offers_hash,
            expected_token_prices_hash,
            expected_auction_result_root,
        ) = zkauction_lib::run_auction(
            &|x: &[u8]| keccak256(x),
            &prover_address,
            &bid_submissions,
            &offer_submissions,
            &bid_reveals,
            &offer_reveals,
            &tokens,
        );
        assert_eq!(acc_bids_hash, expected_acc_bids_hash);
        assert_eq!(acc_offers_hash, expected_acc_offers_hash);
        assert_eq!(token_prices_hash, expected_token_prices_hash);
        assert_eq!(auction_result_root, expected_auction_result_root);
        println!("Values are correct!");

        // Record the number of cycles executed.
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(ZK_AUCTION_ELF);

        // Generate the proof
        let proof = client
            .prove(&pk, stdin)
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
