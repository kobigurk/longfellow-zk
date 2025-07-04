/// Merkle proof structures and verification

use crate::{Hasher, MerkleTree};
use longfellow_core::{LongfellowError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

/// A proof for a single leaf in a Merkle tree
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleProof<H: Hasher> {
    /// Index of the leaf being proven
    pub leaf_index: usize,
    /// Sibling hashes from leaf to root
    pub siblings: Vec<H::Output>,
    /// Phantom data for hasher type
    #[serde(skip)]
    pub _hasher: PhantomData<H>,
}

impl<H: Hasher> MerkleProof<H> {
    /// Verify this proof against a root hash
    pub fn verify(&self, root: &H::Output, leaf_data: &[u8]) -> bool {
        let mut current_hash = H::hash_leaf(leaf_data);
        let mut current_index = self.leaf_index;
        
        for sibling in &self.siblings {
            if current_index & 1 == 0 {
                // Current node is left child
                current_hash = H::hash_pair(&current_hash, sibling);
            } else {
                // Current node is right child
                current_hash = H::hash_pair(sibling, &current_hash);
            }
            current_index /= 2;
        }
        
        &current_hash == root
    }
    
    /// Get the size of this proof in bytes
    pub fn size_bytes(&self) -> usize {
        self.siblings.len() * std::mem::size_of::<H::Output>()
    }
}

/// A batch proof for multiple leaves (more efficient than individual proofs)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiProof<H: Hasher> {
    /// Indices of leaves being proven
    pub leaf_indices: Vec<usize>,
    /// All required hashes (deduplicated)
    pub hashes: HashMap<(usize, usize), H::Output>, // (level, index) -> hash
    /// Phantom data for hasher type
    #[serde(skip)]
    pub _hasher: PhantomData<H>,
}

impl<H: Hasher> MultiProof<H> {
    /// Create a multi-proof for given indices
    pub fn create(tree: &MerkleTree<H>, indices: &[usize]) -> Result<Self> {
        let mut required_hashes = HashMap::new();
        let mut computed_nodes = HashSet::new();
        
        // Mark all nodes on paths from leaves to root
        for &leaf_idx in indices {
            let mut idx = leaf_idx;
            for level in 0..tree.height() {
                computed_nodes.insert((level, idx));
                idx /= 2;
            }
        }
        
        // Collect all required sibling hashes
        for &leaf_idx in indices {
            let mut idx = leaf_idx;
            for level in 0..tree.height() - 1 {
                let sibling_idx = idx ^ 1;
                
                // Only include if sibling is not in computed set
                if !computed_nodes.contains(&(level, sibling_idx)) {
                    if let Some(hash) = tree.get_node(level, sibling_idx) {
                        required_hashes.insert((level, sibling_idx), hash.clone());
                    }
                }
                
                idx /= 2;
            }
        }
        
        Ok(Self {
            leaf_indices: indices.to_vec(),
            hashes: required_hashes,
            _hasher: PhantomData,
        })
    }
    
    /// Verify this multi-proof
    pub fn verify(&self, root: &H::Output, leaf_data: &[(usize, &[u8])]) -> bool {
        // Create map of leaf indices to data
        let leaf_map: HashMap<usize, &[u8]> = leaf_data.iter()
            .map(|&(idx, data)| (idx, data))
            .collect();
        
        // Check all provided leaves are in the proof
        for &idx in &self.leaf_indices {
            if !leaf_map.contains_key(&idx) {
                return false;
            }
        }
        
        // Compute all node values bottom-up
        let mut computed: HashMap<(usize, usize), H::Output> = HashMap::new();
        
        // Start with leaf hashes
        for &idx in &self.leaf_indices {
            if let Some(&data) = leaf_map.get(&idx) {
                computed.insert((0, idx), H::hash_leaf(data));
            }
        }
        
        // Work up the tree
        let max_level = (self.leaf_indices.iter().max().unwrap_or(&0) + 1).next_power_of_two().trailing_zeros() as usize;
        
        for level in 0..max_level {
            let mut indices_at_level: HashSet<usize> = HashSet::new();
            
            // Collect all indices we need to compute at next level
            for (&(l, idx), _) in &computed {
                if l == level {
                    indices_at_level.insert(idx / 2);
                }
            }
            
            // Compute parent nodes
            for &parent_idx in &indices_at_level {
                let left_idx = parent_idx * 2;
                let right_idx = parent_idx * 2 + 1;
                
                let left_hash = computed.get(&(level, left_idx))
                    .or_else(|| self.hashes.get(&(level, left_idx)));
                let right_hash = computed.get(&(level, right_idx))
                    .or_else(|| self.hashes.get(&(level, right_idx)));
                
                if let (Some(left), Some(right)) = (left_hash, right_hash) {
                    computed.insert((level + 1, parent_idx), H::hash_pair(left, right));
                }
            }
        }
        
        // Check if we computed the correct root
        computed.get(&(max_level, 0))
            .map(|computed_root| computed_root == root)
            .unwrap_or(false)
    }
    
    /// Get the size of this proof in bytes
    pub fn size_bytes(&self) -> usize {
        self.hashes.len() * std::mem::size_of::<H::Output>()
    }
    
    /// Convert to individual proofs (less efficient but simpler interface)
    pub fn to_individual_proofs(&self, tree: &MerkleTree<H>) -> Result<Vec<MerkleProof<H>>> {
        self.leaf_indices.iter()
            .map(|&idx| tree.prove(idx))
            .collect()
    }
}

/// Compressed proof using bit vectors for efficiency
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressedProof<H: Hasher> {
    /// Bit vector indicating which nodes are included
    pub included_nodes: Vec<u8>,
    /// The actual hash values in order
    pub hashes: Vec<H::Output>,
    /// Tree height
    pub height: usize,
    /// Phantom data
    #[serde(skip)]
    pub _hasher: PhantomData<H>,
}

impl<H: Hasher> CompressedProof<H> {
    /// Create a compressed proof from a multi-proof
    pub fn from_multiproof(multiproof: &MultiProof<H>, tree_height: usize) -> Self {
        let total_nodes = (1 << tree_height) - 1;
        let mut included_nodes = vec![0u8; (total_nodes + 7) / 8];
        let mut hashes = Vec::new();
        
        // Mark included nodes and collect hashes in order
        for level in 0..tree_height {
            let level_start = (1 << level) - 1;
            let level_size = 1 << level;
            
            for idx in 0..level_size {
                if multiproof.hashes.contains_key(&(level, idx)) {
                    let global_idx = level_start + idx;
                    included_nodes[global_idx / 8] |= 1 << (global_idx % 8);
                    hashes.push(multiproof.hashes[&(level, idx)].clone());
                }
            }
        }
        
        Self {
            included_nodes,
            hashes,
            height: tree_height,
            _hasher: PhantomData,
        }
    }
    
    /// Get the size of this proof in bytes
    pub fn size_bytes(&self) -> usize {
        self.included_nodes.len() + self.hashes.len() * std::mem::size_of::<H::Output>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::Sha3_256Hasher;
    
    #[test]
    fn test_single_proof() {
        let data = vec![b"a", b"b", b"c", b"d"];
        let tree = MerkleTree::<Sha3_256Hasher>::new(&data).unwrap();
        
        for i in 0..4 {
            let proof = tree.prove(i).unwrap();
            assert!(proof.verify(tree.root(), data[i]));
            
            // Wrong data should fail
            assert!(!proof.verify(tree.root(), b"wrong"));
        }
    }
    
    #[test]
    fn test_multi_proof() {
        let data: Vec<Vec<u8>> = (0..8).map(|i| vec![i]).collect();
        let tree = MerkleTree::<Sha3_256Hasher>::new(&data).unwrap();
        
        // Create multi-proof for indices 1, 3, 5
        let indices = vec![1, 3, 5];
        let multiproof = tree.prove_batch(&indices).unwrap();
        
        // Prepare leaf data
        let leaf_data: Vec<(usize, &[u8])> = indices.iter()
            .map(|&i| (i, data[i].as_slice()))
            .collect();
        
        assert!(multiproof.verify(tree.root(), &leaf_data));
        
        // Size should be less than individual proofs
        let individual_size: usize = indices.iter()
            .map(|&i| tree.prove(i).unwrap().size_bytes())
            .sum();
        
        assert!(multiproof.size_bytes() < individual_size);
    }
    
    #[test]
    fn test_compressed_proof() {
        let data: Vec<Vec<u8>> = (0..16).map(|i| vec![i]).collect();
        let tree = MerkleTree::<Sha3_256Hasher>::new(&data).unwrap();
        
        let indices = vec![0, 1, 4, 7, 15];
        let multiproof = tree.prove_batch(&indices).unwrap();
        
        let compressed = CompressedProof::from_multiproof(&multiproof, tree.height());
        
        // Compressed should be smaller than regular multiproof
        assert!(compressed.size_bytes() <= multiproof.size_bytes());
    }
}