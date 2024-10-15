use alloy_primitives::B256;
use tiny_keccak::{Hasher, Keccak};

/// Computes the Keccak-256 hash of the input bytes using [SP1's Keccak precompile](https://docs.succinct.xyz/writing-programs/precompiles.html).
///
/// # Arguments
///
/// * `bytes` - A slice of bytes to be hashed.
///
/// # Returns
///
/// A 32-byte array containing the Keccak-256 hash.
pub fn sp1_keccak256(bytes: &[u8]) -> B256 {
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::keccak256;

    #[test]
    fn test_sp1_keccak256() {
        let input: [u8; 32] = [1u8; 32];
        let expected_output = keccak256(input);

        let output: B256 = sp1_keccak256(&input);
        assert_eq!(output, expected_output);
    }
}
