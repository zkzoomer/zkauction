use alloy_primitives::{aliases::U96, Address, B256, U256};
use alloy_sol_types::SolValue;

/// Generates a unique identifying key for an order by combining an address and an ID.
///
/// # Arguments
///
/// * `address` - The Ethereum address of the bidder or offeror.
/// * `id` - The 96-bit unique identifier for the bid or offer.
///
/// # Returns
///
/// A `B256` value representing the unique key.
pub fn get_key(address: &Address, id: &U96) -> B256 {
    let mut key = [0u8; 32];
    key[0..20].copy_from_slice(address.as_slice());
    key[20..32].copy_from_slice(&id.to_be_bytes::<12>());
    B256::from(key)
}

/// Calculates the price hash by hashing together the revealed price and nonce.
///
/// # Arguments
///
/// * `price` - The price that was revealed.
/// * `nonce` - A random value used to prevent rainbow table attacks.
///
/// # Returns
///
/// A `B256` value representing the price hash, which is the Keccak-256 hash of the price and nonce.
pub fn get_price_hash<F: Fn(&[u8]) -> B256>(hash_function: &F, price: &U256, nonce: &U256) -> B256 {
    hash_function(
        &[
            &price.to_be_bytes::<32>()[..],
            &nonce.to_be_bytes::<32>()[..],
        ]
        .concat(),
    )
}

/// Adds an item to a hash chain by combining it with the previous accumulator value.
///
/// # Arguments
///
/// * `hash_function` - A function that computes a 32-byte hash from a byte slice.
/// * `item` - The item to be added to the hash chain. Must implement the `SolValue` trait.
/// * `acc` - The current accumulator value (previous hash in the chain).
///
/// # Returns
///
/// A new `B256` hash representing the updated state of the hash chain.
///
/// # Type Parameters
///
/// * `F`: The type of the hash function closure.
/// * `S`: The type of the item being added to the hash chain, which must implement `SolValue`.
pub fn add_to_hash_chain<F, S>(hash_function: &F, item: &S, acc: &B256) -> B256
where
    F: Fn(&[u8]) -> B256,
    S: SolValue,
{
    let encoded_item: Vec<u8> = item.abi_encode_packed();
    let input: Vec<u8> = [&acc[..], &encoded_item].concat();
    hash_function(&input)
}

// TEST HELPER FUNCTIONS
pub mod test {
    use alloy_primitives::{keccak256, B256};
    use alloy_sol_types::{sol, SolValue};

    pub fn calculate_expected_hash_chain_output(
        start_value: &B256,
        elements: &[impl SolValue],
    ) -> B256 {
        sol! { struct ChainedStruct { bytes32 startValue; bytes newBytes; } }
        let mut expected_output: B256 = *start_value;
        for offer in elements.iter() {
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
}
