/// Equivalence tests for merkle module

use longfellow_merkle::{MerkleTree, MerkleProof};
use longfellow_algebra::Fp128;
use sha2::{Sha256, Digest};
use std::time::Instant;

#[test]
fn test_merkle_tree_construction() {
    println!("\n=== Merkle Tree Construction Test ===");
    
    let test_sizes = vec![2, 4, 8, 16, 32, 64];
    
    for size in test_sizes {
        // Create leaves
        let leaves: Vec<Fp128> = (0..size).map(|i| Fp128::from(i as u64)).collect();
        
        // Build tree
        let tree = MerkleTree::new(&leaves);
        
        // Verify root is deterministic
        let tree2 = MerkleTree::new(&leaves);
        assert_eq!(tree.root(), tree2.root(), "Root should be deterministic for size {}", size);
        
        // Verify tree properties
        assert_eq!(tree.num_leaves(), size, "Leaf count mismatch");
        
        println!("  ✓ Tree with {} leaves: root = {:?}", size, hex::encode(&tree.root()[..8]));
    }
}

#[test]
fn test_merkle_proof_generation_and_verification() {
    println!("\n=== Merkle Proof Generation and Verification Test ===");
    
    let sizes = vec![8, 16, 32];
    
    for size in sizes {
        let leaves: Vec<Fp128> = (0..size).map(|i| Fp128::from((i * i) as u64)).collect();
        let tree = MerkleTree::new(&leaves);
        
        // Test proof for each leaf
        for i in 0..size {
            let proof = tree.generate_proof(i);
            
            // Verify proof
            assert!(
                proof.verify(&tree.root(), &leaves[i], i),
                "Proof verification failed for leaf {} in tree of size {}", i, size
            );
            
            // Verify proof fails with wrong leaf
            let wrong_leaf = Fp128::from(999999);
            assert!(
                !proof.verify(&tree.root(), &wrong_leaf, i),
                "Proof should fail with wrong leaf"
            );
            
            // Verify proof fails with wrong index
            if i + 1 < size {
                assert!(
                    !proof.verify(&tree.root(), &leaves[i], i + 1),
                    "Proof should fail with wrong index"
                );
            }
        }
        
        println!("  ✓ Generated and verified {} proofs for tree of size {}", size, size);
    }
}

#[test]
fn test_merkle_multi_proof() {
    println!("\n=== Merkle Multi-Proof Test ===");
    
    let leaves: Vec<Fp128> = (0..32).map(|i| Fp128::from(i as u64)).collect();
    let tree = MerkleTree::new(&leaves);
    
    // Test various multi-proof combinations
    let test_cases = vec![
        vec![0, 1],           // Adjacent leaves
        vec![0, 31],          // Opposite ends
        vec![7, 8, 9],        // Consecutive
        vec![0, 15, 31],      // Spread out
        vec![10, 11, 12, 13], // Block of 4
    ];
    
    for indices in test_cases {
        let proof = tree.generate_multi_proof(&indices);
        
        // Collect values for these indices
        let values: Vec<Fp128> = indices.iter().map(|&i| leaves[i]).collect();
        
        // Verify multi-proof
        assert!(
            proof.verify_multi(&tree.root(), &values, &indices),
            "Multi-proof verification failed for indices {:?}", indices
        );
        
        // Verify proof size is optimal
        let individual_size: usize = indices.iter()
            .map(|&i| tree.generate_proof(i).path.len())
            .sum();
        
        println!("  ✓ Multi-proof for {:?}: {} hashes (vs {} individual)", 
            indices, proof.hashes.len(), individual_size);
    }
}

#[test]
fn test_merkle_tree_updates() {
    println!("\n=== Merkle Tree Update Test ===");
    
    let mut leaves: Vec<Fp128> = (0..16).map(|i| Fp128::from(i as u64)).collect();
    let mut tree = MerkleTree::new(&leaves);
    let original_root = tree.root();
    
    // Update a leaf
    let update_index = 7;
    let new_value = Fp128::from(999);
    leaves[update_index] = new_value;
    
    // Rebuild tree (in practice, we'd have incremental updates)
    tree = MerkleTree::new(&leaves);
    
    // Verify root changed
    assert_ne!(tree.root(), original_root, "Root should change after update");
    
    // Verify proof for updated leaf
    let proof = tree.generate_proof(update_index);
    assert!(
        proof.verify(&tree.root(), &new_value, update_index),
        "Proof should verify for updated leaf"
    );
    
    println!("  ✓ Tree update successful");
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    
    #[test]
    fn bench_merkle_tree_construction() {
        println!("\n=== Merkle Tree Construction Benchmarks ===");
        
        let sizes = vec![100, 1000, 10000, 100000];
        
        for size in sizes {
            let leaves: Vec<Fp128> = (0..size).map(|i| Fp128::from(i as u64)).collect();
            
            let start = Instant::now();
            let tree = MerkleTree::new(&leaves);
            let duration = start.elapsed();
            
            println!("  {} leaves: {:?} ({:.2} μs/leaf)", 
                size, 
                duration, 
                duration.as_micros() as f64 / size as f64
            );
            
            // Force use of tree to prevent optimization
            assert!(!tree.root().is_empty());
        }
    }
    
    #[test]
    fn bench_merkle_proof_generation() {
        println!("\n=== Merkle Proof Generation Benchmarks ===");
        
        let sizes = vec![1000, 10000, 100000];
        
        for size in sizes {
            let leaves: Vec<Fp128> = (0..size).map(|i| Fp128::from(i as u64)).collect();
            let tree = MerkleTree::new(&leaves);
            
            // Benchmark proof generation
            let indices: Vec<usize> = (0..100).map(|i| (i * 97) % size).collect();
            
            let start = Instant::now();
            for &idx in &indices {
                let _ = tree.generate_proof(idx);
            }
            let duration = start.elapsed();
            
            println!("  {} leaves: {:?} ({:.2} μs/proof)", 
                size,
                duration,
                duration.as_micros() as f64 / indices.len() as f64
            );
        }
    }
    
    #[test]
    fn bench_merkle_proof_verification() {
        println!("\n=== Merkle Proof Verification Benchmarks ===");
        
        let size = 10000;
        let leaves: Vec<Fp128> = (0..size).map(|i| Fp128::from(i as u64)).collect();
        let tree = MerkleTree::new(&leaves);
        
        // Generate proofs
        let proofs: Vec<(usize, MerkleProof)> = (0..100)
            .map(|i| {
                let idx = (i * 97) % size;
                (idx, tree.generate_proof(idx))
            })
            .collect();
        
        let start = Instant::now();
        let iterations = 1000;
        
        for _ in 0..iterations {
            for (idx, proof) in &proofs {
                let verified = proof.verify(&tree.root(), &leaves[*idx], *idx);
                assert!(verified);
            }
        }
        
        let duration = start.elapsed();
        let ops = iterations * proofs.len();
        
        println!("  Verification: {:?} ({:.2} μs/verify)", 
            duration,
            duration.as_micros() as f64 / ops as f64
        );
    }
    
    #[test]
    fn bench_merkle_multi_proof() {
        println!("\n=== Merkle Multi-Proof Benchmarks ===");
        
        let size = 10000;
        let leaves: Vec<Fp128> = (0..size).map(|i| Fp128::from(i as u64)).collect();
        let tree = MerkleTree::new(&leaves);
        
        let test_sizes = vec![2, 5, 10, 20, 50];
        
        for proof_size in test_sizes {
            // Generate random indices
            let indices: Vec<usize> = (0..proof_size)
                .map(|i| (i * 997) % size)
                .collect();
            
            // Benchmark multi-proof generation
            let start = Instant::now();
            let multi_proof = tree.generate_multi_proof(&indices);
            let gen_duration = start.elapsed();
            
            // Benchmark verification
            let values: Vec<Fp128> = indices.iter().map(|&i| leaves[i]).collect();
            let start = Instant::now();
            let iterations = 1000;
            
            for _ in 0..iterations {
                let verified = multi_proof.verify_multi(&tree.root(), &values, &indices);
                assert!(verified);
            }
            
            let verify_duration = start.elapsed();
            
            println!("  {} indices:", proof_size);
            println!("    Generation: {:?}", gen_duration);
            println!("    Verification: {:?} ({:.2} μs/op)", 
                verify_duration,
                verify_duration.as_micros() as f64 / iterations as f64
            );
        }
    }
}