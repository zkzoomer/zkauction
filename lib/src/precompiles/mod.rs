#[cfg(test)]
mod tests;

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

/// Computes the Keccak-256 hash of the input bytes optimizing for the RISC Zero zkVM.
///
/// # Arguments
///
/// * `bytes` - A slice of bytes to be hashed.
///
/// # Returns
///
/// A 32-byte array containing the Keccak-256 hash.
pub fn risc0_keccak256(_bytes: &[u8]) -> [u8; 32] {
    unimplemented!()
}
