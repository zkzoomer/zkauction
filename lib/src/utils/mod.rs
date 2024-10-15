//! This module contains a lean incremental Merkle tree implementation which follows
//! [Semaphore's implementation](https://hackmd.io/@vplasencia/S1whLBN16)
use alloy_primitives::{keccak256, B256};

/// A lean incremental Merkle tree is an append-only merkle which minimizes the number of hash calculations
///
/// This structure represents an append-only Merkle tree that minimizes the number of hash calculations.
/// It stores nodes at each level of the tree, allowing for efficient updates and proof generation.
pub struct LeanIncrementalMerkleTree {
    /// Stores the nodes of the tree. Each inner `Vec` represents a level in the tree.
    /// The first `Vec` (index 0) contains the leaves, and the last `Vec` contains the root.
    nodes: Vec<Vec<B256>>,
}

impl Default for LeanIncrementalMerkleTree {
    /// Creates a new, empty `LeanIncrementalMerkleTree`.
    fn default() -> Self {
        Self::new(&[])
    }
}

/// Represents a Merkle proof for the LeanIncrementalMerkleTree
///
/// This struct contains all the necessary information to verify the inclusion
/// of a specific `leaf` in the Merkle tree defined by the `root`.
pub struct LeanIMTMerkleProof {
    /// The root hash of the Merkle tree.
    pub root: B256,
    /// The leaf hash for which the proof is generated.
    pub leaf: B256,
    /// The index of the leaf in the tree.
    pub index: usize,
    /// The sibling hashes needed to reconstruct the path to the root.
    pub siblings: Vec<B256>,
}

impl LeanIncrementalMerkleTree {
    /// Create a new lean incremental Merkle tree containing the provided `leaves`
    ///
    /// # Arguments
    ///
    /// * `leaves` - A vector of leaf hashes to initialize the tree with.
    ///
    /// # Returns
    ///
    /// A new `LeanIncrementalMerkleTree` instance.
    pub fn new(leaves: &[B256]) -> Self {
        let mut tree: LeanIncrementalMerkleTree = Self {
            nodes: vec![vec![]],
        };
        if !leaves.is_empty() {
            tree.insert_many(leaves);
        }
        tree
    }

    /// Returns the root hash of the Merkle tree.
    ///
    /// If the tree is empty, returns a zero `B256` value.
    pub fn root(&self) -> B256 {
        *self.nodes[self.depth()].first().unwrap_or(&B256::ZERO)
    }

    /// Returns the depth of the Merkle tree.
    ///
    /// The depth is the number of levels in the tree minus one (this definition excludes the leaf level).
    pub fn depth(&self) -> usize {
        self.nodes.len() - 1
    }

    /// Returns a vector containing all the leaves of the Merkle tree.
    pub fn leaves(&self) -> Vec<B256> {
        self.nodes[0].clone()
    }

    /// Returns the size (number of leaves) of the Merkle tree.
    pub fn size(&self) -> usize {
        self.nodes[0].len()
    }

    /// Finds the index of a given `leaf` in the Merkle tree.
    ///
    /// # Arguments
    ///
    /// * `leaf` - The leaf hash to search for.
    ///
    /// # Returns
    ///
    /// An `Option<usize>` containing the index if the leaf is found, or `None` if it's not present.
    pub fn index_of(&self, leaf: &B256) -> Option<usize> {
        self.nodes[0].iter().position(|&x| x == *leaf)
    }

    /// Checks if the Merkle tree contains a specific `leaf`.
    ///
    /// # Arguments
    ///
    /// * `leaf` - The leaf hash to check for.
    ///
    /// # Returns
    ///
    /// `true` if the leaf is present in the tree, `false` otherwise.
    pub fn has(&self, leaf: &B256) -> bool {
        self.nodes[0].contains(leaf)
    }

    /// Inserts multiple leaves into the Merkle tree.
    ///
    /// This method updates the tree structure efficiently by only recalculating
    /// the necessary nodes.
    ///
    /// # Arguments
    ///
    /// * `leaves` - A vector of leaf hashes to insert into the tree.
    fn insert_many(&mut self, leaves: &[B256]) {
        let mut start_index: usize = self.size() >> 1;
        self.nodes[0].extend_from_slice(leaves);

        let new_levels: usize =
            ((self.size() as f64).log2().ceil() as usize).saturating_sub(self.depth());
        self.nodes.extend((0..new_levels).map(|_| vec![]));

        for level in 0..self.depth() {
            let num_nodes: usize = (self.nodes[level].len() + 1) / 2;

            for index in start_index..num_nodes {
                let left_node: B256 = self.nodes[level][index * 2];
                let right_node: Option<&B256> = self.nodes[level].get(index * 2 + 1);

                let parent_node: B256 = match right_node {
                    Some(right_node) => keccak256([&left_node, right_node].concat()),
                    None => left_node,
                };

                if index >= self.nodes[level + 1].len() {
                    self.nodes[level + 1].push(parent_node);
                } else {
                    self.nodes[level + 1][index] = parent_node;
                }
            }

            start_index >>= 1;
        }
    }

    /// Generates a `LeanIMTMerkleProof` Merkle proof for a leaf at the given `index`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the leaf for which to generate the proof.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the corresponding `LeanIMTMerkleProof` or an error message
    /// if the index is out of bounds.
    pub fn generate_proof(&self, index: usize) -> Result<LeanIMTMerkleProof, String> {
        if index >= self.size() {
            return Err(format!(
                "The leaf at index '{}' does not exist in this tree",
                index
            ));
        }

        let leaf: B256 = self.nodes[0][index];
        let mut siblings: Vec<B256> = Vec::new();
        let mut path: Vec<bool> = Vec::new();
        let mut current_index: usize = index;

        for level in 0..self.depth() {
            let is_right_node: bool = current_index & 1 == 1;
            let sibling_index: usize = if is_right_node {
                current_index - 1
            } else {
                current_index + 1
            };

            if let Some(sibling) = self.nodes[level].get(sibling_index) {
                path.push(is_right_node);
                siblings.push(*sibling);
            }

            current_index >>= 1;
        }

        path.reverse();
        let proof_index: usize = path.iter().fold(0, |acc, &bit| (acc << 1) | bit as usize);

        Ok(LeanIMTMerkleProof {
            root: self.root(),
            leaf,
            index: proof_index,
            siblings,
        })
    }

    /// Verifies a `LeanIMTMerkleProof` Merkle proof.
    ///
    /// This method checks if the provided proof correctly demonstrates that the
    /// leaf is part of the Merkle tree with the given `root` specified in the proof.
    /// The method **does not** check if the proof is valid for *current* tree, it only
    /// verifies the proof itself as being valid.
    ///
    /// # Arguments
    ///
    /// * `proof` - The `LeanIMTMerkleProof` to verify.
    ///
    ///
    /// `true` if the proof is valid, `false` otherwise.
    pub fn verify_proof(&self, proof: &LeanIMTMerkleProof) -> bool {
        let mut node: B256 = proof.leaf;

        for (i, &sibling) in proof.siblings.iter().enumerate() {
            if (proof.index >> i) & 1 == 1 {
                node = keccak256([&sibling, &node].concat());
            } else {
                node = keccak256([&node, &sibling].concat());
            }
        }

        proof.root == node
    }
}

#[cfg(test)]
mod test {
    use super::LeanIncrementalMerkleTree;
    use alloy_primitives::{keccak256, B256};

    #[test]
    fn test_initializes_empty_tree() {
        let tree: LeanIncrementalMerkleTree = LeanIncrementalMerkleTree::new(&[]);
        assert_eq!(tree.root(), B256::default());
    }

    #[test]
    fn test_initializes_tree_with_leaves() {
        let leaves: Vec<B256> = (0..5).map(|_| B256::random()).collect();
        let tree: LeanIncrementalMerkleTree = LeanIncrementalMerkleTree::new(&leaves);

        let manual_root: B256 = {
            let left_node = keccak256(
                [
                    &keccak256([&leaves[0], &leaves[1]].concat()),
                    &keccak256([&leaves[2], &leaves[3]].concat()),
                ]
                .concat(),
            );
            keccak256([&left_node, &leaves[4]].concat())
        };

        assert_eq!(tree.root(), manual_root);
    }

    #[test]
    fn test_leaves() {
        let leaves: Vec<B256> = (0..rand::random::<u16>()).map(|_| B256::random()).collect();
        let tree: LeanIncrementalMerkleTree = LeanIncrementalMerkleTree::new(&leaves);

        assert_eq!(tree.leaves(), leaves);
    }

    #[test]
    fn test_index_of() {
        let element: B256 = B256::random();
        let mut leaves: Vec<B256> = (0..rand::random::<u16>()).map(|_| B256::random()).collect();
        let insert_index = rand::random::<usize>() % leaves.len();
        leaves.insert(insert_index, element);
        let tree: LeanIncrementalMerkleTree = LeanIncrementalMerkleTree::new(&leaves);

        assert_eq!(tree.index_of(&element).unwrap(), insert_index);
    }

    #[test]
    fn test_has() {
        let element: B256 = B256::random();
        let mut leaves: Vec<B256> = (0..rand::random::<u16>()).map(|_| B256::random()).collect();
        let insert_index = rand::random::<usize>() % leaves.len();
        leaves.insert(insert_index, element);
        let tree: LeanIncrementalMerkleTree = LeanIncrementalMerkleTree::new(&leaves);

        assert!(tree.has(&element));
        assert!(!tree.has(&B256::random()));
    }

    #[test]
    fn test_generate_verify_proof() {
        let size: u16 = rand::random::<u16>();
        let leaves: Vec<B256> = (0..size).map(|_| B256::random()).collect();

        let tree: LeanIncrementalMerkleTree = LeanIncrementalMerkleTree::new(&leaves);
        let proof = tree
            .generate_proof(rand::random::<usize>() % size as usize)
            .unwrap();

        assert!(tree.verify_proof(&proof));
    }
}
