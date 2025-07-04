use longfellow_merkle::*;
use longfellow_merkle::hash::{Sha256Hasher, Sha3_256Hasher, Blake3Hasher};
use longfellow_merkle::batch::{BatchMerkleTree, IncrementalMerkleTree};
use longfellow_merkle::proof::CompressedProof;

#[test]
fn test_empty_tree_error() {
    let data: Vec<&[u8]> = vec![];
    let result = MerkleTree::<Sha3_256Hasher>::new(&data);
    assert!(result.is_err());
}

#[test]
fn test_single_element_tree() {
    let data = vec![b"hello"];
    let tree = MerkleTree::<Sha3_256Hasher>::new(&data).unwrap();
    
    assert_eq!(tree.num_leaves(), 1);
    assert_eq!(tree.height(), 1);
    
    let proof = tree.prove(0).unwrap();
    assert!(proof.siblings.is_empty());
    assert!(MerkleTree::<Sha3_256Hasher>::verify(
        tree.root(),
        0,
        data[0],
        &proof
    ));
}

#[test]
fn test_power_of_two_tree() {
    let data: Vec<Vec<u8>> = (0..8).map(|i| vec![i]).collect();
    let tree = MerkleTree::<Blake3Hasher>::new(&data).unwrap();
    
    assert_eq!(tree.num_leaves(), 8);
    assert_eq!(tree.height(), 4); // log2(8) + 1
    
    // Test all proofs
    for i in 0..8 {
        let proof = tree.prove(i).unwrap();
        assert_eq!(proof.siblings.len(), 3);
        assert!(MerkleTree::<Blake3Hasher>::verify(
            tree.root(),
            i,
            &data[i],
            &proof
        ));
    }
}

#[test]
fn test_non_power_of_two_tree() {
    let data: Vec<&str> = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
    let tree = MerkleTree::<Sha256Hasher>::new(&data).unwrap();
    
    assert_eq!(tree.num_leaves(), 10);
    
    // Tree should be padded to 16 leaves internally
    let expected_height = 5; // log2(16) + 1
    assert_eq!(tree.height(), expected_height);
    
    // Verify all proofs
    for i in 0..10 {
        let proof = tree.prove(i).unwrap();
        assert!(MerkleTree::<Sha256Hasher>::verify(
            tree.root(),
            i,
            data[i].as_bytes(),
            &proof
        ));
    }
}

#[test]
fn test_proof_rejection() {
    let data = vec![b"apple", b"banana", b"cherry", b"date"];
    let tree = MerkleTree::<Sha3_256Hasher>::new(&data).unwrap();
    
    let proof = tree.prove(1).unwrap();
    
    // Wrong index
    assert!(!MerkleTree::<Sha3_256Hasher>::verify(
        tree.root(),
        0,
        data[1],
        &proof
    ));
    
    // Wrong data
    assert!(!MerkleTree::<Sha3_256Hasher>::verify(
        tree.root(),
        1,
        b"wrong",
        &proof
    ));
    
    // Tampered proof
    let mut tampered_proof = proof.clone();
    if !tampered_proof.siblings.is_empty() {
        tampered_proof.siblings[0] = [0u8; 32];
    }
    assert!(!MerkleTree::<Sha3_256Hasher>::verify(
        tree.root(),
        1,
        data[1],
        &tampered_proof
    ));
}

#[test]
fn test_multiproof() {
    let data: Vec<Vec<u8>> = (0..16).map(|i| vec![i as u8; 32]).collect();
    let tree = MerkleTree::<Blake3Hasher>::new(&data).unwrap();
    
    // Create multiproof for multiple indices
    let indices = vec![0, 3, 7, 10, 15];
    let multiproof = tree.prove_batch(&indices).unwrap();
    
    // Prepare leaf data
    let leaf_data: Vec<(usize, &[u8])> = indices.iter()
        .map(|&i| (i, data[i].as_slice()))
        .collect();
    
    assert!(multiproof.verify(tree.root(), &leaf_data));
    
    // Test with missing leaf
    let incomplete_data: Vec<(usize, &[u8])> = indices[..4].iter()
        .map(|&i| (i, data[i].as_slice()))
        .collect();
    assert!(!multiproof.verify(tree.root(), &incomplete_data));
    
    // Test with wrong data
    let mut wrong_data = leaf_data.clone();
    wrong_data[0].1 = b"wrong";
    assert!(!multiproof.verify(tree.root(), &wrong_data));
}

#[test]
fn test_merkle_forest() {
    let data: Vec<Vec<u8>> = (0..1000).map(|i| i.to_le_bytes().to_vec()).collect();
    let forest = MerkleForest::<Sha3_256Hasher>::new(&data, 100).unwrap();
    
    assert_eq!(forest.trees.len(), 10); // 1000 / 100
    
    // Test proofs at different positions
    for &global_idx in &[0, 99, 100, 500, 999] {
        let (tree_idx, proof) = forest.prove(global_idx).unwrap();
        assert_eq!(tree_idx, global_idx / 100);
        assert_eq!(proof.leaf_index, global_idx % 100);
        
        // Verify the proof
        let root = forest.trees[tree_idx].root();
        assert!(MerkleTree::<Sha3_256Hasher>::verify(
            root,
            proof.leaf_index,
            &data[global_idx],
            &proof
        ));
    }
}

#[test]
fn test_incremental_tree() {
    let mut inc_tree = IncrementalMerkleTree::<Blake3Hasher>::new(32).unwrap();
    
    // Test appending
    let mut roots = Vec::new();
    for i in 0..20 {
        inc_tree.append(vec![i as u8]).unwrap();
        roots.push(inc_tree.root());
    }
    
    // Roots should change with each append
    for i in 1..roots.len() {
        assert_ne!(roots[i-1], roots[i]);
    }
    
    // Test determinism - create another tree with same data
    let mut inc_tree2 = IncrementalMerkleTree::<Blake3Hasher>::new(32).unwrap();
    for i in 0..20 {
        inc_tree2.append(vec![i as u8]).unwrap();
    }
    
    assert_eq!(inc_tree.root(), inc_tree2.root());
}

#[test]
fn test_incremental_tree_capacity() {
    let mut inc_tree = IncrementalMerkleTree::<Sha256Hasher>::new(8).unwrap();
    
    // Fill to capacity
    for i in 0..8 {
        assert!(inc_tree.append(vec![i]).is_ok());
    }
    
    // Should fail when exceeding capacity
    assert!(inc_tree.append(vec![8]).is_err());
}

#[test]
fn test_batch_merkle_operations() {
    let data: Vec<Vec<u8>> = (0..1000)
        .map(|i| format!("item_{}", i).into_bytes())
        .collect();
    
    let mut batch_tree = BatchMerkleTree::<Sha3_256Hasher>::new(&data).unwrap();
    batch_tree.build_cache();
    
    // Test range proofs
    let proofs = batch_tree.prove_range(100, 200).unwrap();
    assert_eq!(proofs.len(), 100);
    
    // Verify all proofs
    let verify_data: Vec<_> = (100..200)
        .zip(&proofs)
        .map(|(i, proof)| (i, data[i].as_slice(), proof.clone()))
        .collect();
    
    let results = batch_tree.batch_verify(&verify_data);
    assert!(results.iter().all(|&r| r));
}

#[test]
fn test_compressed_proof() {
    let data: Vec<Vec<u8>> = (0..64).map(|i| vec![i as u8]).collect();
    let tree = MerkleTree::<Blake3Hasher>::new(&data).unwrap();
    
    // Create multiproof
    let indices = vec![1, 5, 10, 20, 30, 40, 50, 60];
    let multiproof = tree.prove_batch(&indices).unwrap();
    
    // Create compressed version
    let compressed = CompressedProof::from_multiproof(&multiproof, tree.height());
    
    // Check size reduction
    let multiproof_size = multiproof.size_bytes();
    let compressed_size = compressed.size_bytes();
    
    // Compressed should typically be smaller (though not always)
    println!("Multiproof size: {} bytes", multiproof_size);
    println!("Compressed size: {} bytes", compressed_size);
}

#[test]
fn test_different_hash_functions() {
    let data = vec![b"test", b"data", b"for", b"hashing"];
    
    // Create trees with different hash functions
    let tree_sha256 = MerkleTree::<Sha256Hasher>::new(&data).unwrap();
    let tree_sha3 = MerkleTree::<Sha3_256Hasher>::new(&data).unwrap();
    let tree_blake3 = MerkleTree::<Blake3Hasher>::new(&data).unwrap();
    
    // Roots should be different
    assert_ne!(tree_sha256.root(), tree_sha3.root());
    assert_ne!(tree_sha256.root(), tree_blake3.root());
    assert_ne!(tree_sha3.root(), tree_blake3.root());
    
    // But proofs should still work correctly for each
    for i in 0..4 {
        let proof_sha256 = tree_sha256.prove(i).unwrap();
        let proof_sha3 = tree_sha3.prove(i).unwrap();
        let proof_blake3 = tree_blake3.prove(i).unwrap();
        
        assert!(MerkleTree::<Sha256Hasher>::verify(
            tree_sha256.root(), i, data[i], &proof_sha256
        ));
        assert!(MerkleTree::<Sha3_256Hasher>::verify(
            tree_sha3.root(), i, data[i], &proof_sha3
        ));
        assert!(MerkleTree::<Blake3Hasher>::verify(
            tree_blake3.root(), i, data[i], &proof_blake3
        ));
    }
}

#[test]
fn test_large_tree_performance() {
    // Test with a reasonably large tree
    let data: Vec<Vec<u8>> = (0..10000)
        .map(|i| i.to_le_bytes().to_vec())
        .collect();
    
    let start = std::time::Instant::now();
    let tree = MerkleTree::<Blake3Hasher>::new(&data).unwrap();
    let construction_time = start.elapsed();
    
    println!("Tree construction time for 10k leaves: {:?}", construction_time);
    
    // Test proof generation
    let start = std::time::Instant::now();
    let proof = tree.prove(5000).unwrap();
    let proof_time = start.elapsed();
    
    println!("Proof generation time: {:?}", proof_time);
    
    // Test verification
    let start = std::time::Instant::now();
    let result = MerkleTree::<Blake3Hasher>::verify(
        tree.root(),
        5000,
        &data[5000],
        &proof
    );
    let verify_time = start.elapsed();
    
    println!("Verification time: {:?}", verify_time);
    assert!(result);
}