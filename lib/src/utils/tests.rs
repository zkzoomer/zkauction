use crate::precompiles::sp1_keccak256;
use crate::types::{OfferSubmission, TokenInformation};
use crate::utils::lean_imt::LeanIncrementalMerkleTree;
use crate::utils::{hash_chain, hash_exit_root, hash_unrolled};
use alloy_primitives::{aliases::U96, keccak256, Address, B256, U256};
use alloy_sol_types::{sol, SolValue};

#[test]
fn test_hash_chain() {
    // Setup
    let start_value: B256 = B256::random();
    let offers: Vec<OfferSubmission> = (0..42)
        .map(|_| OfferSubmission {
            offeror: Address::random(),
            id: U96::from(rand::random::<u64>()),
            offerPriceHash: B256::random(),
            amount: U256::from(rand::random::<u128>()),
            purchaseToken: Address::random(),
        })
        .collect();

    // Execute
    let expected_output: B256 = calculate_expected_hash_chain_output(&start_value, &offers);
    let sp1_output: B256 = hash_chain(&sp1_keccak256, &offers, &start_value);

    // Assert
    assert_eq!(sp1_output, expected_output);

    // TODO: Testing with `risc0_keccak256`
    //let risc0_output: B256 = hash_unrolled(&risc0_keccak256, &encoded_tokens);
    //assert_eq!(risc0_output, expected_output);
}

#[test]
fn test_hash_unrolled() {
    let tokens: Vec<TokenInformation> = (0..2)
        .map(|_| TokenInformation {
            tokenAddress: Address::random(),
            price: U256::from(rand::random::<u128>()),
        })
        .collect();

    // Recreates the onchain process
    let mut encoded_tokens: Vec<u8> = Vec::new();
    for token in &tokens {
        let serialized_token: Vec<u8> = token.abi_encode_packed();
        encoded_tokens.extend_from_slice(&serialized_token);
    }
    let expected_output: B256 = keccak256(&encoded_tokens);

    // Testing with `sp1_keccak256`
    let sp1_output: B256 = hash_unrolled(&sp1_keccak256, &tokens);
    assert_eq!(
        sp1_output, expected_output,
        "SP1 output does not match expected output"
    );

    // TODO: Testing with `risc0_keccak256`
    //let risc0_output: B256 = hash_unrolled(&risc0_keccak256, &encoded_tokens);
    //assert_eq!(risc0_output, expected_output);
}

#[test]
fn test_hash_exit_root() {
    // Setup
    let leaves: Vec<B256> = (0..11).map(|_| B256::random()).collect();

    // Calculate expected result
    let expected_tree = LeanIncrementalMerkleTree::new(&leaves);
    let expected_output: B256 = expected_tree.root();

    // Testing with `sp1_keccak256`
    let sp1_output: B256 = hash_exit_root(&sp1_keccak256, &leaves);
    assert_eq!(sp1_output, expected_output);

    // TODO: Test with risc0_keccak256 once implemented
    //let risc0_output = hash_exit_root(&risc0_keccak256, &exit_root);
    //assert_eq!(risc0_output, expected_output);
}

// HELPER FUNCTIONS
fn calculate_expected_hash_chain_output(start_value: &B256, offers: &[OfferSubmission]) -> B256 {
    sol! { struct ChainedStruct { bytes32 startValue; bytes newBytes; } }
    let mut expected_output: B256 = *start_value;
    for offer in offers {
        let new_bytes: Vec<u8> = offer.abi_encode_packed();
        expected_output = keccak256(
            ChainedStruct {
                startValue: expected_output,
                newBytes: new_bytes.into(),
            }
            .abi_encode_packed(),
        );
    }
    expected_output
}
