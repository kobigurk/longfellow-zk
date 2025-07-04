/// Merkle tree implementation for column commitments

use longfellow_algebra::traits::Field;
use longfellow_core::{LongfellowError, Result};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;

/// Merkle tree for committing to columns
pub struct MerkleTree {
    /// Tree nodes (level -> nodes at that level)
    nodes: Vec<Vec<[u8; 32]>>,
    
    /// Number of leaves
    num_leaves: usize,
}

impl MerkleTree {
    /// Create a new Merkle tree from field element columns
    pub fn new<F: Field>(columns: &[Vec<F>]) -> Result<Self> {
        if columns.is_empty() {
            return Err(LongfellowError::InvalidParameter(
                "Cannot create Merkle tree with no columns".to_string()
            ));
        }
        
        let num_leaves = columns.len();
        let mut nodes = Vec::new();
        
        // Compute leaf hashes
        let mut leaf_hashes = Vec::with_capacity(num_leaves);
        for column in columns {
            leaf_hashes.push(hash_column(column));
        }
        
        // Pad to next power of 2
        let tree_size = num_leaves.next_power_of_two();
        leaf_hashes.resize(tree_size, [0u8; 32]);
        
        nodes.push(leaf_hashes);
        
        // Build tree bottom-up
        let mut current_size = tree_size;
        while current_size > 1 {
            current_size /= 2;
            let mut level = Vec::with_capacity(current_size);
            
            let prev_level = nodes.last().unwrap();
            for i in 0..current_size {
                let left = &prev_level[2 * i];
                let right = &prev_level[2 * i + 1];
                level.push(hash_pair(left, right));
            }
            
            nodes.push(level);
        }
        
        Ok(Self { nodes, num_leaves })
    }
    
    /// Get the root hash
    pub fn root(&self) -> [u8; 32] {
        self.nodes.last().unwrap()[0]
    }
    
    /// Generate a proof for opening a column
    pub fn prove(&self, index: usize) -> Result<Vec<[u8; 32]>> {
        if index >= self.num_leaves {
            return Err(LongfellowError::InvalidParameter(
                format!("Column index {} out of range", index)
            ));
        }
        
        let mut proof = Vec::new();
        let mut current_index = index;
        
        // Traverse from leaf to root
        for level in 0..self.nodes.len() - 1 {
            let sibling_index = current_index ^ 1;
            let sibling_hash = self.nodes[level][sibling_index];
            proof.push(sibling_hash);
            current_index /= 2;
        }
        
        Ok(proof)
    }
    
    /// Verify a Merkle proof
    pub fn verify<F: Field>(
        root: &[u8; 32],
        index: usize,
        column: &[F],
        proof: &[[u8; 32]],
    ) -> bool {
        let mut current_hash = hash_column(column);
        let mut current_index = index;
        
        for sibling in proof {
            if current_index & 1 == 0 {
                // Current node is left child
                current_hash = hash_pair(&current_hash, sibling);
            } else {
                // Current node is right child
                current_hash = hash_pair(sibling, &current_hash);
            }
            current_index /= 2;
        }
        
        current_hash == *root
    }
}

/// Hash a column of field elements
fn hash_column<F: Field>(column: &[F]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(b"LigeroColumn");
    hasher.update(&(column.len() as u64).to_le_bytes());
    
    for elem in column {
        let bytes = elem.to_bytes_le();
        hasher.update(&bytes);
    }
    
    hasher.finalize().into()
}

/// Hash two nodes together
fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(b"LigeroNode");
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

/// Multi-Merkle tree for efficient batch operations
pub struct MultiMerkleTree {
    /// Individual trees for each set of columns
    trees: Vec<MerkleTree>,
    
    /// Mapping from global column index to (tree_index, local_index)
    index_map: HashMap<usize, (usize, usize)>,
}

impl MultiMerkleTree {
    /// Create a multi-tree from multiple sets of columns
    pub fn new<F: Field>(column_sets: &[Vec<Vec<F>>]) -> Result<Self> {
        let mut trees = Vec::new();
        let mut index_map = HashMap::new();
        let mut global_index = 0;
        
        for (tree_idx, columns) in column_sets.iter().enumerate() {
            let tree = MerkleTree::new(columns)?;
            
            for local_idx in 0..columns.len() {
                index_map.insert(global_index, (tree_idx, local_idx));
                global_index += 1;
            }
            
            trees.push(tree);
        }
        
        Ok(Self { trees, index_map })
    }
    
    /// Get all root hashes
    pub fn roots(&self) -> Vec<[u8; 32]> {
        self.trees.iter().map(|t| t.root()).collect()
    }
    
    /// Generate a proof for a global column index
    pub fn prove(&self, global_index: usize) -> Result<(usize, Vec<[u8; 32]>)> {
        let (tree_idx, local_idx) = self.index_map.get(&global_index)
            .ok_or_else(|| LongfellowError::InvalidParameter(
                format!("Invalid global column index: {}", global_index)
            ))?;
        
        let proof = self.trees[*tree_idx].prove(*local_idx)?;
        Ok((*tree_idx, proof))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_merkle_tree() {
        // Create test columns
        let columns: Vec<Vec<Fp128>> = (0..8)
            .map(|i| {
                (0..10)
                    .map(|j| Fp128::from((i * 10 + j) as u64))
                    .collect()
            })
            .collect();
        
        let tree = MerkleTree::new(&columns).unwrap();
        let root = tree.root();
        
        // Test proof generation and verification
        for i in 0..8 {
            let proof = tree.prove(i).unwrap();
            assert!(MerkleTree::verify(&root, i, &columns[i], &proof));
            
            // Test invalid proof
            let mut bad_column = columns[i].clone();
            bad_column[0] += Fp128::one();
            assert!(!MerkleTree::verify(&root, i, &bad_column, &proof));
        }
    }
    
    #[test]
    fn test_multi_merkle_tree() {
        // Create multiple sets of columns
        let set1: Vec<Vec<Fp128>> = (0..4)
            .map(|i| vec![Fp128::from(i as u64); 5])
            .collect();
            
        let set2: Vec<Vec<Fp128>> = (0..4)
            .map(|i| vec![Fp128::from((i + 10) as u64); 5])
            .collect();
        
        let column_sets = vec![set1, set2];
        let multi_tree = MultiMerkleTree::new(&column_sets).unwrap();
        
        let roots = multi_tree.roots();
        assert_eq!(roots.len(), 2);
        
        // Test proving global indices
        let (tree_idx, proof) = multi_tree.prove(5).unwrap(); // Should be in second tree
        assert_eq!(tree_idx, 1);
    }
}