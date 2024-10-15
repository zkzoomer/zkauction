//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can have an
//! EVM-Compatible proof generated which can be verified on-chain.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --system groth16
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --system plonk
//! ```

use alloy_sol_types::SolType;
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use sp1_sdk::{HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey};
use std::path::PathBuf;
use zkauction_lib::orders::PublicValuesStruct;

// Adjust this path based on the actual location of input.rs
#[path = "../lib/input.rs"]
mod input;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const ZK_AUCTION_ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

/// The arguments for the EVM command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct EVMArgs {
    #[clap(long)]
    acc_bids_hash: String,
    #[clap(long)]
    acc_offers_hash: String,
    #[clap(long)]
    tokens_hash: String,
    #[clap(long, default_value = "0")]
    auction_result_root: String,
    #[clap(long, value_enum, default_value = "groth16")]
    system: ProofSystem,
}

/// Enum representing the available proof systems
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystem {
    Plonk,
    Groth16, // wen??
}

/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SP1zkAuctionProofFixture {
    prover_address: String,
    acc_bids_hash: String,
    acc_offers_hash: String,
    token_prices_hash: String,
    auction_result_root: String,
    vkey: String,
    public_values: String,
    proof: String,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    /* // Parse the command line arguments.
    let args = EVMArgs::parse(); */

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    let (
        _prover_address,
        _bids_submissions,
        _offers_submissions,
        _bid_reveals,
        _offer_reveals,
        _tokens_prices,
    ) = input::set_inputs(&mut stdin);

    let proof_system = ProofSystem::Plonk;

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the program.
    let (pk, vk) = client.setup(ZK_AUCTION_ELF);
    println!("Proof System: {:?}", proof_system);

    // Generate the proof based on the selected proof system.
    println!("Generating proof...");
    let start_time = std::time::Instant::now();
    let proof = match proof_system {
        ProofSystem::Plonk => client.prove(&pk, stdin).plonk().run(),
        ProofSystem::Groth16 => client.prove(&pk, stdin).groth16().run(),
    }
    .expect("failed to generate proof");
    let proving_time = start_time.elapsed();
    println!("Proving time: {:?}", proving_time);
    create_proof_fixture(&proof, &vk, proof_system);
}

/// Create a fixture for the given proof.
fn create_proof_fixture(
    proof: &SP1ProofWithPublicValues,
    vk: &SP1VerifyingKey,
    system: ProofSystem,
) {
    // Deserialize the public values.
    let bytes = proof.public_values.as_slice();

    let PublicValuesStruct {
        proverAddress,
        accBidsHash,
        accOffersHash,
        tokenPricesHash,
        auctionResultRoot,
    } = PublicValuesStruct::abi_decode(bytes, false).unwrap();

    // Create the testing fixture so we can test things end-to-end.
    let fixture = SP1zkAuctionProofFixture {
        prover_address: proverAddress.to_string(),
        acc_bids_hash: accBidsHash.to_string(),
        acc_offers_hash: accOffersHash.to_string(),
        token_prices_hash: tokenPricesHash.to_string(),
        auction_result_root: auctionResultRoot.to_string(),
        vkey: vk.bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    // The verification key is used to verify that the proof corresponds to the execution of the
    // program on the given input.
    //
    // Note that the verification key stays the same regardless of the input.
    println!("Verification Key: {}", fixture.vkey);

    // The public values are the values which are publicly committed to by the zkVM.
    //
    // If you need to expose the inputs or outputs of your program, you should commit them in
    // the public values.
    println!("Public Values: {}", fixture.public_values);

    // The proof proves to the verifier that the program was executed with some inputs that led to
    // the give public values.
    println!("Proof Bytes: {}", fixture.proof);

    // Save the fixture to a file.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(format!("{:?}-fixture.json", system).to_lowercase()),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}
