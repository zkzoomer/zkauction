#[cfg(test)]
mod tests;

mod lean_imt;

use alloy_primitives::B256;
use alloy_sol_types::SolValue;

/// Computes a hash chain over a sequence of items using a provided hash function.
///
/// # Arguments
///
/// * `hash_function` - A function that computes a 32-byte hash from a byte slice.
/// * `items` - A slice of items to be hashed, each implementing the `SolValue` trait.
/// * `start_value` - The initial 32-byte value to start the hash chain.
///
/// # Returns
///
/// A 32-byte array representing the final hash in the chain.
///
/// # Type Parameters
///
/// * `F` - The type of the hash function.
/// * `S` - The type of items in the slice, which must implement `SolValue`.
pub fn hash_chain<F, S>(hash_function: &F, items: &[S], start_value: &B256) -> B256
where
    F: Fn(&[u8]) -> B256,
    S: SolValue,
{
    items.iter().fold(*start_value, |acc: B256, item: &S| {
        let encoded_item: Vec<u8> = item.abi_encode_packed();
        let input: Vec<u8> = [&acc[..], &encoded_item].concat();
        hash_function(&input)
    })
}

/// Computes a hash of multiple items concatenated together.
///
/// # Arguments
///
/// * `hash_function` - A function that computes a 32-byte hash from a byte slice.
/// * `items` - A slice of items to be hashed, each implementing the `SolValue` trait.
///
/// # Returns
///
/// A 32-byte array representing the hash of the concatenated items.
///
/// # Type Parameters
///
/// * `F` - The type of the hash function.
/// * `S` - The type of items
pub fn hash_unrolled<F: Fn(&[u8]) -> B256, S: SolValue>(hash_function: &F, items: &[S]) -> B256 {
    hash_function(&items.abi_encode_packed())
}

/// Computes the root of a lean incremental Merkle tree from a list of leaves.
///
/// This function implements a bottom-up approach to calculate the Merkle root:
/// it iteratively combines pairs of hashes at each level until a single root hash is obtained.
/// When a node lacks a right counterpart, it adopts the left child's value.
/// The tree's depth dynamically adjusts to the count of leaves, enhancing efficiency
/// by minimizing the number of hash computations.
/// For a better understanding, refer to the [visual explanation](https://hackmd.io/@vplasencia/S1whLBN16).
///
/// # Arguments
///
/// * `hash_function` - A function that computes a 32-byte hash from a byte slice.
/// * `leaves` - A slice of 32-byte arrays representing the leaves of the tree.
///
/// # Returns
///
/// A 32-byte array representing the root of the Merkle tree. If the input `leaves` is empty, it returns a zero byte array.
///
/// # Type Parameters
///
/// * `F` - The type of the hash function.
pub fn hash_exit_root<F>(hash_function: &F, leaves: &[B256]) -> B256
where
    F: Fn(&[u8]) -> B256,
{
    if leaves.is_empty() {
        return B256::ZERO;
    }

    let mut current_level = leaves.to_vec();

    while current_level.len() > 1 {
        current_level = current_level
            .chunks(2)
            .map(|chunk: &[B256]| {
                if chunk.len() == 2 {
                    let input: Vec<u8> = [&chunk[0][..], &chunk[1][..]].concat();
                    hash_function(&input)
                } else {
                    chunk[0]
                }
            })
            .collect();
    }

    current_level[0]
}
