use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use longfellow_algebra::{Fp128, Polynomial};
use longfellow_arrays::{DenseArray, SparseArray, MultiAffineFunction};
use longfellow_merkle::MerkleTree;
use longfellow_random::{Transcript, ChaChaRng};
use longfellow_ec::Point;
use longfellow_gf2k::GF2_128;
use longfellow_circuits::{StandardCircuit, CircuitBuilder, gadgets, utils};
use longfellow_util::{sha256, sha3_256, base64_encode, hex_encode};

/// Benchmark field operations
fn bench_field_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("field_operations");
    
    // Fp128 operations
    let a = Fp128::from(12345u64);
    let b = Fp128::from(67890u64);
    
    group.bench_function("fp128_add", |bench| {
        bench.iter(|| black_box(&a) + black_box(&b))
    });
    
    group.bench_function("fp128_mul", |bench| {
        bench.iter(|| black_box(&a) * black_box(&b))
    });
    
    group.bench_function("fp128_inverse", |bench| {
        bench.iter(|| black_box(&a).inverse())
    });
    
    // GF2_128 operations
    let x = GF2_128::from(12345u64);
    let y = GF2_128::from(67890u64);
    
    group.bench_function("gf2_128_add", |bench| {
        bench.iter(|| black_box(&x) + black_box(&y))
    });
    
    group.bench_function("gf2_128_mul", |bench| {
        bench.iter(|| black_box(&x) * black_box(&y))
    });
    
    group.finish();
}

/// Benchmark polynomial operations
fn bench_polynomial_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("polynomial_operations");
    
    for degree in [16, 32, 64, 128].iter() {
        let coeffs: Vec<Fp128> = (0..*degree)
            .map(|i| Fp128::from(i as u64))
            .collect();
        let poly = Polynomial::from_coefficients(coeffs);
        let point = Fp128::from(42u64);
        
        group.bench_with_input(
            BenchmarkId::new("evaluate", degree),
            degree,
            |bench, _| {
                bench.iter(|| poly.evaluate(black_box(&point)))
            },
        );
    }
    
    // FFT benchmarks
    for size_log in [8, 10, 12, 14].iter() {
        let size = 1 << size_log;
        let coeffs: Vec<Fp128> = (0..size)
            .map(|i| Fp128::from(i as u64))
            .collect();
        let mut poly = Polynomial::from_coefficients(coeffs);
        
        group.bench_with_input(
            BenchmarkId::new("fft", size),
            &size,
            |bench, _| {
                bench.iter(|| poly.fft())
            },
        );
    }
    
    group.finish();
}

/// Benchmark array operations
fn bench_array_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_operations");
    
    // Dense array benchmarks
    for dim in [8, 16, 32].iter() {
        let dims = vec![*dim, *dim];
        let mut array = DenseArray::<Fp128>::new(&dims);
        
        // Initialize
        for i in 0..*dim {
            for j in 0..*dim {
                array.set(&[i, j], Fp128::from((i * dim + j) as u64));
            }
        }
        
        group.bench_with_input(
            BenchmarkId::new("dense_get", format!("{}x{}", dim, dim)),
            dim,
            |bench, &d| {
                bench.iter(|| {
                    for i in 0..d {
                        for j in 0..d {
                            black_box(array.get(&[i, j]));
                        }
                    }
                })
            },
        );
        
        // Multi-affine evaluation
        let point = vec![Fp128::from(2u64), Fp128::from(3u64)];
        group.bench_with_input(
            BenchmarkId::new("dense_evaluate", format!("{}x{}", dim, dim)),
            dim,
            |bench, _| {
                bench.iter(|| array.evaluate(black_box(&point)))
            },
        );
    }
    
    // Sparse array benchmarks
    let size = 1000;
    let mut sparse = SparseArray::<Fp128>::new(&[size, size]);
    let density = 0.01;
    let num_elements = ((size * size) as f64 * density) as usize;
    
    for i in 0..num_elements {
        let x = (i * 97) % size;
        let y = (i * 61) % size;
        sparse.set(&[x, y], Fp128::from(i as u64));
    }
    
    group.bench_function("sparse_get_hit", |bench| {
        bench.iter(|| {
            for i in 0..100 {
                let x = (i * 97) % size;
                let y = (i * 61) % size;
                black_box(sparse.get(&[x, y]));
            }
        })
    });
    
    group.finish();
}

/// Benchmark Merkle tree operations
fn bench_merkle_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_operations");
    
    for size in [100, 1000, 10000].iter() {
        let leaves: Vec<Fp128> = (0..*size)
            .map(|i| Fp128::from(i as u64))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("tree_construction", size),
            size,
            |bench, _| {
                bench.iter(|| MerkleTree::new(black_box(&leaves)))
            },
        );
        
        let tree = MerkleTree::new(&leaves);
        
        group.bench_with_input(
            BenchmarkId::new("proof_generation", size),
            size,
            |bench, &s| {
                bench.iter(|| {
                    for i in 0..10 {
                        let idx = (i * 97) % s;
                        black_box(tree.generate_proof(idx));
                    }
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark cryptographic operations
fn bench_crypto_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("crypto_operations");
    
    // Hash function benchmarks
    for size in [64, 256, 1024, 4096].iter() {
        let data = vec![0x42u8; *size];
        
        group.bench_with_input(
            BenchmarkId::new("sha256", size),
            size,
            |bench, _| {
                bench.iter(|| sha256(black_box(&data)))
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("sha3_256", size),
            size,
            |bench, _| {
                bench.iter(|| sha3_256(black_box(&data)))
            },
        );
    }
    
    // Transcript operations
    let mut transcript = Transcript::new(b"benchmark");
    
    group.bench_function("transcript_append", |bench| {
        bench.iter(|| {
            transcript.append_message(b"label", b"data");
        })
    });
    
    group.bench_function("transcript_challenge", |bench| {
        bench.iter(|| {
            black_box(transcript.challenge_field_element(b"challenge"));
        })
    });
    
    // EC operations
    let g = Point::generator();
    let scalar = longfellow_ec::Scalar::from(12345u64);
    
    group.bench_function("ec_scalar_mul", |bench| {
        bench.iter(|| g.scalar_mul(black_box(&scalar)))
    });
    
    group.finish();
}

/// Benchmark circuit operations
fn bench_circuit_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("circuit_operations");
    
    let mut circuit = StandardCircuit::<Fp128>::new();
    
    // Pre-allocate variables
    let vars: Vec<usize> = (0..1000).map(|_| circuit.alloc_var()).collect();
    
    group.bench_function("constraint_linear", |bench| {
        bench.iter(|| {
            circuit.add_constraint(longfellow_circuits::Constraint::Linear {
                coeffs: vec![
                    (vars[0], Fp128::one()),
                    (vars[1], Fp128::one()),
                    (vars[2], -Fp128::one()),
                ],
                constant: Fp128::zero(),
            }).unwrap();
        })
    });
    
    group.bench_function("gadget_add", |bench| {
        bench.iter(|| {
            utils::add_gate(&mut circuit, vars[0], vars[1]).unwrap()
        })
    });
    
    group.bench_function("gadget_mul", |bench| {
        bench.iter(|| {
            utils::mul_gate(&mut circuit, vars[0], vars[1]).unwrap()
        })
    });
    
    group.bench_function("bit_decompose_8", |bench| {
        bench.iter(|| {
            gadgets::bit_decompose(&mut circuit, vars[0], 8).unwrap()
        })
    });
    
    group.finish();
}

/// Benchmark encoding operations
fn bench_encoding_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding_operations");
    
    for size in [64, 256, 1024].iter() {
        let data = vec![0x42u8; *size];
        
        group.bench_with_input(
            BenchmarkId::new("base64_encode", size),
            size,
            |bench, _| {
                bench.iter(|| base64_encode(black_box(&data)))
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("hex_encode", size),
            size,
            |bench, _| {
                bench.iter(|| hex_encode(black_box(&data)))
            },
        );
    }
    
    group.finish();
}

/// Comprehensive performance comparison
fn bench_performance_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("rust_vs_cpp_comparison");
    
    // This would compare Rust vs C++ implementations
    // For now, we'll just benchmark the Rust versions
    
    // Field operations comparison
    let a = Fp128::from(12345u64);
    let b = Fp128::from(67890u64);
    
    group.bench_function("rust_fp128_mul", |bench| {
        bench.iter(|| {
            for _ in 0..1000 {
                black_box(&a) * black_box(&b);
            }
        })
    });
    
    // FFT comparison
    let coeffs: Vec<Fp128> = (0..256).map(|i| Fp128::from(i as u64)).collect();
    let mut poly = Polynomial::from_coefficients(coeffs);
    
    group.bench_function("rust_fft_256", |bench| {
        bench.iter(|| poly.fft())
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_field_operations,
    bench_polynomial_operations,
    bench_array_operations,
    bench_merkle_operations,
    bench_crypto_operations,
    bench_circuit_operations,
    bench_encoding_operations,
    bench_performance_comparison
);

criterion_main!(benches);