use crate::precompiles::sp1_keccak256;
use alloy_primitives::{keccak256, B256};

#[test]
fn test_sp1_keccak256() {
    let input: [u8; 32] = [1u8; 32];
    let expected_output = keccak256(input);

    let output: B256 = sp1_keccak256(&input);
    assert_eq!(output, expected_output);
}
