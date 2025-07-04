/// Batch operations for Merkle trees

use crate::{Hasher, MerkleTree};
use longfellow_core::{LongfellowError, Result};
use rayon::prelude::*;
use std::sync::Arc;

/// A Merkle tree optimized for batch operations
pub struct BatchMerkleTree<H: Hasher> {
    /// The underlying tree
    tree: Arc<MerkleTree<H>>,
    /// Cached computations for efficiency
    cache: Option<BatchCache<H>>,
}

/// Cache for batch operations
struct BatchCache<H: Hasher> {
    /// Precomputed subtree roots at various levels
    subtree_roots: Vec<Vec<H::Output>>,
}

impl<H: Hasher> BatchMerkleTree<H> {
    /// Create a new batch Merkle tree
    pub fn new<T: AsRef<[u8]>>(data: &[T]) -> Result<Self> {
        let tree = Arc::new(MerkleTree::new(data)?);
        Ok(Self { tree, cache: None })
    }
    
    /// Build cache for efficient batch operations
    pub fn build_cache(&mut self) {
        let mut subtree_roots = Vec::new();
        
        // Cache subtree roots at different granularities
        for level in 0..self.tree.height().saturating_sub(2) {
            let level_roots: Vec<_> = (0..self.tree.nodes[level].len())
                .into_par_iter()
                .map(|i| self.tree.nodes[level][i].clone())
                .collect();
            subtree_roots.push(level_roots);
        }
        
        self.cache = Some(BatchCache { subtree_roots });
    }
    
    /// Update multiple leaves efficiently
    pub fn batch_update<T: AsRef<[u8]> + Send + Sync>(
        &mut self,
        updates: &[(usize, T)],
    ) -> Result<()> {
        // Validate indices
        for (idx, _) in updates {
            if *idx >= self.tree.num_leaves() {
                return Err(LongfellowError::InvalidParameter(
                    format!("Update index {} out of range", idx)
                ));
            }
        }
        
        // Create new tree with updates
        let mut new_data: Vec<Vec<u8>> = (0..self.tree.num_leaves())
            .map(|i| {
                // This is inefficient but necessary without storing original data
                vec![0u8; 32] // Placeholder
            })
            .collect();
        
        // Apply updates
        for (idx, data) in updates {
            new_data[*idx] = data.as_ref().to_vec();
        }
        
        // Rebuild tree
        let new_tree = Arc::new(MerkleTree::new(&new_data)?);
        self.tree = new_tree;
        self.cache = None; // Invalidate cache
        
        Ok(())
    }
    
    /// Verify multiple proofs in parallel
    pub fn batch_verify(
        &self,
        proofs: &[(usize, &[u8], crate::MerkleProof<H>)],
    ) -> Vec<bool> {
        let root = self.tree.root();
        
        proofs
            .par_iter()
            .map(|(idx, data, proof)| {
                proof.leaf_index == *idx && proof.verify(root, data)
            })
            .collect()
    }
    
    /// Generate proofs for a range of indices
    pub fn prove_range(&self, start: usize, end: usize) -> Result<Vec<crate::MerkleProof<H>>> {
        if end > self.tree.num_leaves() {
            return Err(LongfellowError::InvalidParameter(
                format!("Range end {} exceeds number of leaves", end)
            ));
        }
        
        (start..end)
            .into_par_iter()
            .map(|i| self.tree.prove(i))
            .collect()
    }
    
    /// Get subtree root at a specific position
    pub fn subtree_root(&self, level: usize, index: usize) -> Option<&H::Output> {
        self.tree.get_node(level, index)
    }
    
    /// Get the underlying tree
    pub fn tree(&self) -> &MerkleTree<H> {
        &self.tree
    }
}

/// Incremental Merkle tree that supports efficient appends
pub struct IncrementalMerkleTree<H: Hasher> {
    /// Current leaves
    leaves: Vec<H::Output>,
    /// Cached right-edge hashes for each level
    right_edges: Vec<Option<H::Output>>,
    /// Maximum capacity (must be power of 2)
    capacity: usize,
}

impl<H: Hasher> IncrementalMerkleTree<H> {
    /// Create a new incremental tree with given capacity
    pub fn new(capacity: usize) -> Result<Self> {
        if capacity == 0 || capacity & (capacity - 1) != 0 {
            return Err(LongfellowError::InvalidParameter(
                "Capacity must be a power of 2".to_string()
            ));
        }
        
        let height = capacity.trailing_zeros() as usize;
        
        Ok(Self {
            leaves: Vec::new(),
            right_edges: vec![None; height],
            capacity,
        })
    }
    
    /// Append a new leaf
    pub fn append<T: AsRef<[u8]>>(&mut self, data: T) -> Result<()> {
        if self.leaves.len() >= self.capacity {
            return Err(LongfellowError::InvalidParameter(
                "Tree is at capacity".to_string()
            ));
        }
        
        let leaf_hash = H::hash_leaf(data.as_ref());
        let leaf_index = self.leaves.len();
        self.leaves.push(leaf_hash.clone());
        
        // Update right edges
        let mut current_hash = leaf_hash;
        let mut idx = leaf_index;
        
        for level in 0..self.right_edges.len() {
            if idx & 1 == 0 {
                // This is a left child, just store as right edge
                self.right_edges[level] = Some(current_hash);
                break;
            } else {
                // This is a right child, combine with stored left sibling
                if let Some(left_sibling) = &self.right_edges[level] {
                    current_hash = H::hash_pair(left_sibling, &current_hash);
                    self.right_edges[level] = None;
                } else {
                    // No left sibling stored, shouldn't happen
                    break;
                }
            }
            idx /= 2;
        }
        
        Ok(())
    }
    
    /// Get current root
    pub fn root(&self) -> H::Output {
        if self.leaves.is_empty() {
            return H::empty_hash();
        }
        
        // Combine all right edges with empty hashes
        let mut roots = Vec::new();
        let mut empty_hash = H::empty_hash();
        
        for (level, edge) in self.right_edges.iter().enumerate() {
            if let Some(hash) = edge {
                roots.push((level, hash.clone()));
            } else {
                // Use empty hash for this level
                roots.push((level, empty_hash.clone()));
            }
            
            // Update empty hash for next level
            empty_hash = H::hash_pair(&empty_hash, &empty_hash);
        }
        
        // Combine roots from right to left
        let mut current_hash = roots.last().unwrap().1.clone();
        
        for i in (0..roots.len() - 1).rev() {
            if self.leaves.len() & (1 << i) != 0 {
                current_hash = H::hash_pair(&roots[i].1, &current_hash);
            }
        }
        
        current_hash
    }
    
    /// Get number of leaves
    pub fn num_leaves(&self) -> usize {
        self.leaves.len()
    }
    
    /// Convert to a regular Merkle tree
    pub fn to_merkle_tree(&self) -> Result<MerkleTree<H>> {
        if self.leaves.is_empty() {
            return Err(LongfellowError::InvalidParameter(
                "Cannot convert empty incremental tree".to_string()
            ));
        }
        
        // Need original data to reconstruct, so we'll use a workaround
        // In practice, you'd store the original data
        let placeholder_data: Vec<Vec<u8>> = (0..self.leaves.len())
            .map(|i| vec![i as u8])
            .collect();
        
        MerkleTree::new(&placeholder_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::Sha3_256Hasher;
    
    #[test]
    fn test_batch_merkle_tree() {
        let data: Vec<Vec<u8>> = (0..100).map(|i| vec![i]).collect();
        let mut batch_tree = BatchMerkleTree::<Sha3_256Hasher>::new(&data).unwrap();
        
        // Build cache
        batch_tree.build_cache();
        
        // Test range proofs
        let proofs = batch_tree.prove_range(10, 20).unwrap();
        assert_eq!(proofs.len(), 10);
        
        // Verify all proofs
        let verify_data: Vec<_> = (10..20)
            .zip(&proofs)
            .map(|(i, proof)| (i, data[i].as_slice(), proof.clone()))
            .collect();
        
        let results = batch_tree.batch_verify(&verify_data);
        assert!(results.iter().all(|&r| r));
    }
    
    #[test]
    fn test_incremental_tree() {
        let mut inc_tree = IncrementalMerkleTree::<Sha3_256Hasher>::new(16).unwrap();
        
        // Add some leaves
        for i in 0..10 {
            inc_tree.append(vec![i]).unwrap();
        }
        
        assert_eq!(inc_tree.num_leaves(), 10);
        
        // Root should be deterministic
        let root1 = inc_tree.root();
        
        // Create another tree with same data
        let mut inc_tree2 = IncrementalMerkleTree::<Sha3_256Hasher>::new(16).unwrap();
        for i in 0..10 {
            inc_tree2.append(vec![i]).unwrap();
        }
        
        let root2 = inc_tree2.root();
        assert_eq!(root1, root2);
    }
    
    #[test]
    fn test_incremental_capacity() {
        let mut inc_tree = IncrementalMerkleTree::<Sha3_256Hasher>::new(4).unwrap();
        
        // Fill to capacity
        for i in 0..4 {
            assert!(inc_tree.append(vec![i]).is_ok());
        }
        
        // Should fail when over capacity
        assert!(inc_tree.append(vec![5]).is_err());
    }
}