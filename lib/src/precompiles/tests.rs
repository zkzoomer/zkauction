use crate::precompiles::sp1_keccak256;
use alloy_primitives::{keccak256, B256};

#[test]
fn test_sp1_keccak256() {
    let input: [u8; 32] = [1u8; 32];
    let expected_output = keccak256(input);

    let output: B256 = sp1_keccak256(&input);
    assert_eq!(output, expected_output);
}

/* #[test]
fn test_risc0_keccak256() {
    let input: [u8; 32] = [1u8; 32];
    let expected_output: [u8; 32] =
        hex!("cebc8882fecbec7fb80d2cf4b312bec018884c2d66667c67a90508214bd8bafc");

    let output: [u8; 32] = risc0_keccak256(&input);
    assert_eq!(output, expected_output);
} */
