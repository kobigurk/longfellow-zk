/// General-purpose Merkle tree implementation with multiple hash function support

use longfellow_core::{LongfellowError, Result};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use rayon::prelude::*;

pub mod hash;
pub mod proof;
pub mod batch;

pub use hash::{Hasher, HashFunction};
pub use proof::{MerkleProof, MultiProof};
pub use batch::BatchMerkleTree;

/// Generic Merkle tree implementation
#[derive(Clone, Debug)]
pub struct MerkleTree<H: Hasher> {
    /// Tree nodes organized by level (leaves at index 0)
    nodes: Vec<Vec<H::Output>>,
    /// Number of leaves
    num_leaves: usize,
    /// Phantom data for hasher type
    _hasher: PhantomData<H>,
}

impl<H: Hasher> MerkleTree<H> {
    /// Create a new Merkle tree from data
    pub fn new<T: AsRef<[u8]>>(data: &[T]) -> Result<Self> {
        if data.is_empty() {
            return Err(LongfellowError::InvalidParameter(
                "Cannot create Merkle tree with no data".to_string()
            ));
        }
        
        let num_leaves = data.len();
        let tree_size = num_leaves.next_power_of_two();
        
        // Compute leaf hashes
        let mut leaves: Vec<H::Output> = data
            .par_iter()
            .map(|item| H::hash_leaf(item.as_ref()))
            .collect();
        
        // Pad with empty hashes if needed
        let empty_hash = H::empty_hash();
        leaves.resize(tree_size, empty_hash);
        
        let mut nodes = vec![leaves];
        
        // Build tree bottom-up
        let mut level_size = tree_size;
        while level_size > 1 {
            level_size /= 2;
            let prev_level = &nodes[nodes.len() - 1];
            
            let level: Vec<H::Output> = (0..level_size)
                .into_par_iter()
                .map(|i| {
                    H::hash_pair(&prev_level[2 * i], &prev_level[2 * i + 1])
                })
                .collect();
            
            nodes.push(level);
        }
        
        Ok(Self {
            nodes,
            num_leaves,
            _hasher: PhantomData,
        })
    }
    
    /// Get the root hash
    pub fn root(&self) -> &H::Output {
        &self.nodes[self.nodes.len() - 1][0]
    }
    
    /// Get the number of leaves
    pub fn num_leaves(&self) -> usize {
        self.num_leaves
    }
    
    /// Get tree height
    pub fn height(&self) -> usize {
        self.nodes.len()
    }
    
    /// Generate a proof for a leaf at given index
    pub fn prove(&self, index: usize) -> Result<MerkleProof<H>> {
        if index >= self.num_leaves {
            return Err(LongfellowError::InvalidParameter(
                format!("Leaf index {} out of range", index)
            ));
        }
        
        let mut siblings = Vec::with_capacity(self.height() - 1);
        let mut current_index = index;
        
        // Collect siblings from leaf to root
        for level in 0..self.nodes.len() - 1 {
            let sibling_index = current_index ^ 1;
            siblings.push(self.nodes[level][sibling_index].clone());
            current_index /= 2;
        }
        
        Ok(MerkleProof {
            leaf_index: index,
            siblings,
            _hasher: PhantomData,
        })
    }
    
    /// Generate multiple proofs efficiently
    pub fn prove_batch(&self, indices: &[usize]) -> Result<MultiProof<H>> {
        // Validate indices
        for &index in indices {
            if index >= self.num_leaves {
                return Err(LongfellowError::InvalidParameter(
                    format!("Leaf index {} out of range", index)
                ));
            }
        }
        
        MultiProof::create(self, indices)
    }
    
    /// Verify a proof
    pub fn verify(
        root: &H::Output,
        leaf_index: usize,
        leaf_data: &[u8],
        proof: &MerkleProof<H>,
    ) -> bool {
        proof.verify(root, leaf_data)
    }
    
    /// Get a specific node in the tree
    pub fn get_node(&self, level: usize, index: usize) -> Option<&H::Output> {
        self.nodes.get(level).and_then(|l| l.get(index))
    }
    
    /// Get all leaves
    pub fn leaves(&self) -> &[H::Output] {
        &self.nodes[0][..self.num_leaves]
    }
}

/// Configuration for Merkle tree construction
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleConfig {
    /// Hash function to use
    pub hash_function: HashFunction,
    /// Whether to use parallel construction
    pub parallel: bool,
    /// Minimum leaves for parallel processing
    pub parallel_threshold: usize,
}

impl Default for MerkleConfig {
    fn default() -> Self {
        Self {
            hash_function: HashFunction::Sha3_256,
            parallel: true,
            parallel_threshold: 1000,
        }
    }
}

/// A forest of Merkle trees for managing large datasets
pub struct MerkleForest<H: Hasher> {
    /// Individual trees in the forest
    trees: Vec<MerkleTree<H>>,
    /// Tree size (number of leaves per tree)
    tree_size: usize,
}

impl<H: Hasher> MerkleForest<H> {
    /// Create a new forest with specified tree size
    pub fn new<T: AsRef<[u8]>>(data: &[T], tree_size: usize) -> Result<Self> {
        if tree_size == 0 {
            return Err(LongfellowError::InvalidParameter(
                "Tree size must be positive".to_string()
            ));
        }
        
        let trees: Result<Vec<_>> = data
            .chunks(tree_size)
            .map(|chunk| MerkleTree::new(chunk))
            .collect();
        
        Ok(Self {
            trees: trees?,
            tree_size,
        })
    }
    
    /// Get all root hashes
    pub fn roots(&self) -> Vec<&H::Output> {
        self.trees.iter().map(|t| t.root()).collect()
    }
    
    /// Generate proof for a global index
    pub fn prove(&self, global_index: usize) -> Result<(usize, MerkleProof<H>)> {
        let tree_index = global_index / self.tree_size;
        let local_index = global_index % self.tree_size;
        
        if tree_index >= self.trees.len() {
            return Err(LongfellowError::InvalidParameter(
                format!("Global index {} out of range", global_index)
            ));
        }
        
        let proof = self.trees[tree_index].prove(local_index)?;
        Ok((tree_index, proof))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::{Sha3_256Hasher, Blake3Hasher};
    
    #[test]
    fn test_merkle_tree_basic() {
        let data = vec![b"hello", b"world", b"foo", b"bar"];
        let tree = MerkleTree::<Sha3_256Hasher>::new(&data).unwrap();
        
        assert_eq!(tree.num_leaves(), 4);
        assert_eq!(tree.height(), 3); // log2(4) + 1
        
        // Generate and verify proofs
        for i in 0..4 {
            let proof = tree.prove(i).unwrap();
            assert!(MerkleTree::<Sha3_256Hasher>::verify(
                tree.root(),
                i,
                data[i],
                &proof
            ));
        }
    }
    
    #[test]
    fn test_merkle_tree_non_power_of_two() {
        let data = vec![b"a", b"b", b"c", b"d", b"e"];
        let tree = MerkleTree::<Blake3Hasher>::new(&data).unwrap();
        
        assert_eq!(tree.num_leaves(), 5);
        
        // Tree should be padded to 8 leaves
        assert_eq!(tree.nodes[0].len(), 8);
        
        // Verify all actual leaves
        for i in 0..5 {
            let proof = tree.prove(i).unwrap();
            assert!(MerkleTree::<Blake3Hasher>::verify(
                tree.root(),
                i,
                data[i],
                &proof
            ));
        }
    }
    
    #[test]
    fn test_merkle_forest() {
        let data: Vec<Vec<u8>> = (0..100)
            .map(|i| vec![i as u8; 32])
            .collect();
        
        let forest = MerkleForest::<Sha3_256Hasher>::new(&data, 10).unwrap();
        
        assert_eq!(forest.trees.len(), 10);
        assert_eq!(forest.roots().len(), 10);
        
        // Test global proof
        let (tree_idx, proof) = forest.prove(25).unwrap(); // Should be tree 2, leaf 5
        assert_eq!(tree_idx, 2);
        assert_eq!(proof.leaf_index, 5);
    }
    
    #[test]
    fn test_proof_tampering() {
        let data = vec![b"test1", b"test2", b"test3", b"test4"];
        let tree = MerkleTree::<Sha3_256Hasher>::new(&data).unwrap();
        
        let mut proof = tree.prove(0).unwrap();
        
        // Tamper with proof
        if !proof.siblings.is_empty() {
            proof.siblings[0] = [0u8; 32];
        }
        
        // Verification should fail
        assert!(!MerkleTree::<Sha3_256Hasher>::verify(
            tree.root(),
            0,
            data[0],
            &proof
        ));
    }
}