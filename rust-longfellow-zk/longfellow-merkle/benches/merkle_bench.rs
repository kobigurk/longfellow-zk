use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use longfellow_merkle::*;
use longfellow_merkle::hash::{Sha256Hasher, Sha3_256Hasher, Blake3Hasher};
use longfellow_merkle::batch::{BatchMerkleTree, IncrementalMerkleTree};

fn generate_data(size: usize) -> Vec<Vec<u8>> {
    (0..size)
        .map(|i| {
            let mut data = vec![0u8; 32];
            data[0..8].copy_from_slice(&(i as u64).to_le_bytes());
            data
        })
        .collect()
}

fn bench_tree_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_construction");
    
    for size in [100, 1000, 10000, 100000] {
        let data = generate_data(size);
        
        group.throughput(Throughput::Elements(size as u64));
        
        // Benchmark different hash functions
        group.bench_with_input(
            BenchmarkId::new("sha256", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let tree = MerkleTree::<Sha256Hasher>::new(&data).unwrap();
                    black_box(tree.root());
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("sha3_256", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let tree = MerkleTree::<Sha3_256Hasher>::new(&data).unwrap();
                    black_box(tree.root());
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("blake3", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let tree = MerkleTree::<Blake3Hasher>::new(&data).unwrap();
                    black_box(tree.root());
                });
            },
        );
    }
    
    group.finish();
}

fn bench_proof_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_generation");
    
    for size in [1000, 10000, 100000] {
        let data = generate_data(size);
        let tree = MerkleTree::<Sha3_256Hasher>::new(&data).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("single_proof", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let proof = tree.prove(size / 2).unwrap();
                    black_box(proof);
                });
            },
        );
        
        // Batch proofs
        let indices: Vec<usize> = (0..10).map(|i| i * (size / 10)).collect();
        
        group.bench_with_input(
            BenchmarkId::new("batch_proof_10", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let proof = tree.prove_batch(&indices).unwrap();
                    black_box(proof);
                });
            },
        );
    }
    
    group.finish();
}

fn bench_proof_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_verification");
    
    for size in [1000, 10000, 100000] {
        let data = generate_data(size);
        let tree = MerkleTree::<Sha3_256Hasher>::new(&data).unwrap();
        let proof = tree.prove(size / 2).unwrap();
        let root = tree.root();
        
        group.bench_with_input(
            BenchmarkId::new("verify", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let result = MerkleTree::<Sha3_256Hasher>::verify(
                        root,
                        size / 2,
                        &data[size / 2],
                        &proof,
                    );
                    black_box(result);
                });
            },
        );
        
        // Multi-proof verification
        let indices: Vec<usize> = (0..10).map(|i| i * (size / 10)).collect();
        let multiproof = tree.prove_batch(&indices).unwrap();
        let leaf_data: Vec<(usize, &[u8])> = indices.iter()
            .map(|&i| (i, data[i].as_slice()))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("verify_batch_10", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let result = multiproof.verify(root, &leaf_data);
                    black_box(result);
                });
            },
        );
    }
    
    group.finish();
}

fn bench_incremental_tree(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_tree");
    
    for capacity in [1024, 16384, 65536] {
        group.bench_with_input(
            BenchmarkId::new("append", capacity),
            &capacity,
            |b, &cap| {
                let mut tree = IncrementalMerkleTree::<Blake3Hasher>::new(cap).unwrap();
                let data = vec![0u8; 32];
                let mut i = 0;
                
                b.iter(|| {
                    if i >= cap {
                        tree = IncrementalMerkleTree::<Blake3Hasher>::new(cap).unwrap();
                        i = 0;
                    }
                    tree.append(&data).unwrap();
                    i += 1;
                    black_box(tree.root());
                });
            },
        );
    }
    
    group.finish();
}

fn bench_merkle_forest(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_forest");
    
    let data = generate_data(100000);
    
    for tree_size in [100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("construct", tree_size),
            &tree_size,
            |b, &size| {
                b.iter(|| {
                    let forest = MerkleForest::<Sha3_256Hasher>::new(&data, size).unwrap();
                    black_box(forest.roots());
                });
            },
        );
    }
    
    // Proof generation in forest
    let forest = MerkleForest::<Sha3_256Hasher>::new(&data, 1000).unwrap();
    
    group.bench_function("forest_prove", |b| {
        b.iter(|| {
            let (tree_idx, proof) = forest.prove(50000).unwrap();
            black_box((tree_idx, proof));
        });
    });
    
    group.finish();
}

fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    
    let data = generate_data(10000);
    let mut batch_tree = BatchMerkleTree::<Blake3Hasher>::new(&data).unwrap();
    
    group.bench_function("build_cache", |b| {
        b.iter(|| {
            batch_tree.build_cache();
        });
    });
    
    batch_tree.build_cache();
    
    // Range proof generation
    group.bench_function("prove_range_100", |b| {
        b.iter(|| {
            let proofs = batch_tree.prove_range(1000, 1100).unwrap();
            black_box(proofs);
        });
    });
    
    // Batch verification
    let proofs = batch_tree.prove_range(1000, 1100).unwrap();
    let verify_data: Vec<_> = (1000..1100)
        .zip(&proofs)
        .map(|(i, proof)| (i, data[i].as_slice(), proof.clone()))
        .collect();
    
    group.bench_function("batch_verify_100", |b| {
        b.iter(|| {
            let results = batch_tree.batch_verify(&verify_data);
            black_box(results);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_tree_construction,
    bench_proof_generation,
    bench_proof_verification,
    bench_incremental_tree,
    bench_merkle_forest,
    bench_batch_operations
);
criterion_main!(benches);