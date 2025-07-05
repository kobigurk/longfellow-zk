/// Comprehensive benchmarks for the full Longfellow ZK system

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use longfellow_algebra::{Fp128, Field};
use longfellow_ligero::{
    LigeroParams, LigeroInstance, ConstraintSystem, LigeroProver, LigeroVerifier
};
use longfellow_sumcheck::{
    Circuit, Layer, SumcheckInstance, Prover as SumcheckProver, 
    Verifier as SumcheckVerifier, SumcheckOptions
};
use longfellow_zk::{
    Statement, Predicate, DocumentType, ZkProver, ZkVerifier, ProofOptions
};
use rand::rngs::OsRng;
use std::time::Duration;

/// Benchmark Ligero protocol
fn bench_ligero(c: &mut Criterion) {
    let mut group = c.benchmark_group("ligero");
    group.measurement_time(Duration::from_secs(10));
    
    // Different constraint system sizes
    let sizes = vec![10, 50, 100, 500, 1000];
    
    for size in sizes {
        // Create constraint system
        let mut cs = ConstraintSystem::<Fp128>::new(size);
        
        // Add linear constraints
        for i in 0..size/2 {
            let mut coeffs = vec![];
            for j in 0..3 {
                if i + j < size {
                    coeffs.push((i + j, Fp128::from((j + 1) as u64)));
                }
            }
            cs.add_linear_constraint(coeffs, Fp128::from(i as u64));
        }
        
        // Add quadratic constraints
        for i in 0..size/10 {
            if i + 2 < size {
                cs.add_quadratic_constraint(i, i + 1, i + 2);
            }
        }
        
        let params = LigeroParams::security_128();
        let instance = LigeroInstance::new(params, cs.clone()).unwrap();
        
        // Create witness
        let witness: Vec<Fp128> = (0..size)
            .map(|i| Fp128::from(i as u64))
            .collect();
        
        // Benchmark proof generation
        group.bench_with_input(
            BenchmarkId::new("prove", size),
            &size,
            |b, _| {
                let prover = LigeroProver::new(instance.clone()).unwrap();
                b.iter(|| {
                    let proof = prover.prove(&witness, &mut OsRng).unwrap();
                    black_box(proof)
                });
            }
        );
        
        // Generate proof for verification benchmark
        let prover = LigeroProver::new(instance.clone()).unwrap();
        let proof = prover.prove(&witness, &mut OsRng).unwrap();
        
        // Benchmark verification
        group.bench_with_input(
            BenchmarkId::new("verify", size),
            &size,
            |b, _| {
                let verifier = LigeroVerifier::new(instance.clone()).unwrap();
                b.iter(|| {
                    let valid = verifier.verify(&proof).unwrap();
                    black_box(valid)
                });
            }
        );
    }
    
    group.finish();
}

/// Benchmark Sumcheck protocol
fn bench_sumcheck(c: &mut Criterion) {
    let mut group = c.benchmark_group("sumcheck");
    group.measurement_time(Duration::from_secs(10));
    
    // Different circuit sizes and depths
    let configs = vec![
        (4, 2),   // 4 inputs, depth 2
        (8, 3),   // 8 inputs, depth 3
        (16, 4),  // 16 inputs, depth 4
        (32, 5),  // 32 inputs, depth 5
    ];
    
    for (num_inputs, depth) in configs {
        // Build layered circuit
        let mut circuit = Circuit::<Fp128>::new();
        
        // Input layer
        let input_layer = Layer::new_input(num_inputs);
        circuit.add_layer(input_layer);
        
        // Intermediate layers
        let mut current_width = num_inputs;
        for d in 0..depth-1 {
            current_width = current_width / 2;
            if current_width < 1 {
                current_width = 1;
            }
            
            let mut layer = Layer::new(current_width, d);
            for i in 0..current_width {
                // Add with two inputs from previous layer
                let idx1 = (i * 2) % num_inputs;
                let idx2 = (i * 2 + 1) % num_inputs;
                layer.add_gate(i, vec![(idx1, Fp128::one()), (idx2, Fp128::one())]);
            }
            circuit.add_layer(layer);
        }
        
        circuit.finalize().unwrap();
        
        // Create instance
        let num_copies = 8;
        let claimed_sum = Fp128::from((num_inputs * num_copies) as u64);
        let instance = SumcheckInstance::new(circuit.clone(), num_copies, claimed_sum).unwrap();
        
        // Create inputs
        let inputs: Vec<Vec<Fp128>> = (0..num_copies)
            .map(|_| (0..num_inputs).map(|i| Fp128::from(i as u64)).collect())
            .collect();
        
        // Benchmark proof generation
        group.bench_with_input(
            BenchmarkId::new("prove", format!("{}x{}", num_inputs, depth)),
            &num_inputs,
            |b, _| {
                b.iter(|| {
                    let mut prover = SumcheckProver::new(
                        instance.clone(),
                        SumcheckOptions::default()
                    ).unwrap();
                    prover.set_inputs(&inputs).unwrap();
                    let proof = prover.prove(&mut OsRng).unwrap();
                    black_box(proof)
                });
            }
        );
        
        // Generate proof for verification
        let mut prover = SumcheckProver::new(instance.clone(), SumcheckOptions::default()).unwrap();
        prover.set_inputs(&inputs).unwrap();
        let proof = prover.prove(&mut OsRng).unwrap();
        
        // Benchmark verification
        group.bench_with_input(
            BenchmarkId::new("verify", format!("{}x{}", num_inputs, depth)),
            &num_inputs,
            |b, _| {
                let verifier = SumcheckVerifier::new(
                    instance.clone(),
                    SumcheckOptions::default()
                ).unwrap();
                b.iter(|| {
                    let valid = verifier.verify(&proof, &inputs).unwrap();
                    black_box(valid)
                });
            }
        );
    }
    
    group.finish();
}

/// Benchmark Montgomery arithmetic operations
fn bench_montgomery(c: &mut Criterion) {
    let mut group = c.benchmark_group("montgomery");
    
    let a = Fp128::from_u64(123456789);
    let b = Fp128::from_u64(987654321);
    
    group.bench_function("add", |b| {
        b.iter(|| black_box(a) + black_box(b))
    });
    
    group.bench_function("sub", |b| {
        b.iter(|| black_box(a) - black_box(b))
    });
    
    group.bench_function("mul", |b| {
        b.iter(|| black_box(a) * black_box(b))
    });
    
    group.bench_function("square", |b| {
        b.iter(|| black_box(a).square())
    });
    
    group.bench_function("invert", |b| {
        b.iter(|| black_box(a).invert())
    });
    
    // Benchmark different power sizes
    let powers = vec![2, 10, 100, 1000];
    for p in powers {
        group.bench_function(format!("pow_{}", p), |b| {
            b.iter(|| black_box(a).pow(&[p]))
        });
    }
    
    group.finish();
}

/// Benchmark full ZK proof system
fn bench_full_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_zk_system");
    group.measurement_time(Duration::from_secs(20));
    
    // Different statement complexities
    let configs = vec![
        ("simple", vec![Predicate::AgeOver { years: 18 }]),
        ("medium", vec![
            Predicate::AgeOver { years: 18 },
            Predicate::FieldEquals { 
                field: "country".to_string(),
                value: serde_json::json!("US")
            },
        ]),
        ("complex", vec![
            Predicate::AgeOver { years: 18 },
            Predicate::FieldEquals { 
                field: "country".to_string(),
                value: serde_json::json!("US")
            },
            Predicate::ValidSignature,
            Predicate::NotExpired,
        ]),
    ];
    
    for (name, predicates) in configs {
        // Create statement
        let statement = Statement {
            document_type: DocumentType::Jwt,
            predicates,
            revealed_fields: vec!["verified".to_string()],
            hidden_fields: vec!["age".to_string(), "name".to_string()],
        };
        
        // Create mock instance
        let circuit = create_mock_circuit();
        let instance = create_mock_instance(statement.clone(), circuit);
        
        // Benchmark proof generation
        group.bench_function(format!("prove_{}", name), |b| {
            b.iter(|| {
                let prover = ZkProver::new(instance.clone()).unwrap();
                let proof = prover.prove(&mut OsRng, ProofOptions::default()).unwrap();
                black_box(proof)
            });
        });
        
        // Generate proof for verification
        let prover = ZkProver::new(instance.clone()).unwrap();
        let proof = prover.prove(&mut OsRng, ProofOptions::default()).unwrap();
        
        // Benchmark verification
        group.bench_function(format!("verify_{}", name), |b| {
            b.iter(|| {
                let verifier = ZkVerifier::new(statement.clone()).unwrap();
                let valid = verifier.verify(&proof).unwrap();
                black_box(valid)
            });
        });
    }
    
    group.finish();
}

/// Helper to create mock circuit
fn create_mock_circuit() -> longfellow_zk::ZkCircuit<Fp128> {
    let mut circuit = longfellow_zk::ZkCircuit::new(100);
    
    // Add some constraints
    for i in 0..10 {
        circuit.add_linear_constraint(
            vec![(i, Fp128::one()), (i + 1, Fp128::from(2))],
            Fp128::from(i as u64),
        ).unwrap();
    }
    
    // Set witness values
    let witness: Vec<Fp128> = (0..100).map(|i| Fp128::from(i as u64)).collect();
    circuit.set_wire_values(witness);
    
    circuit
}

/// Helper to create mock instance
fn create_mock_instance(
    statement: Statement,
    circuit: longfellow_zk::ZkCircuit<Fp128>,
) -> longfellow_zk::ZkInstance<Fp128> {
    use longfellow_cbor::jwt::Jwt;
    use longfellow_zk::{DocumentData, ZkWitness};
    
    let jwt = Jwt::new(serde_json::json!({
        "sub": "user123",
        "age": 25,
        "country": "US",
        "verified": true,
        "exp": 9999999999i64,
    })).unwrap();
    
    let witness = ZkWitness {
        document: DocumentData::Jwt(jwt),
        private_values: std::collections::HashMap::new(),
        randomness: vec![],
    };
    
    longfellow_zk::ZkInstance {
        statement,
        witness,
        circuit,
    }
}

/// Benchmark proof serialization
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    // Create a sample proof
    let circuit = create_mock_circuit();
    let statement = Statement {
        document_type: DocumentType::Jwt,
        predicates: vec![Predicate::AgeOver { years: 18 }],
        revealed_fields: vec![],
        hidden_fields: vec!["age".to_string()],
    };
    
    let instance = create_mock_instance(statement, circuit);
    let prover = ZkProver::new(instance).unwrap();
    let proof = prover.prove(&mut OsRng, ProofOptions::default()).unwrap();
    
    // Benchmark JSON serialization
    group.bench_function("to_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&proof).unwrap();
            black_box(json)
        });
    });
    
    let json = serde_json::to_string(&proof).unwrap();
    
    // Benchmark JSON deserialization
    group.bench_function("from_json", |b| {
        b.iter(|| {
            let proof: longfellow_zk::ZkProof<Fp128> = serde_json::from_str(&json).unwrap();
            black_box(proof)
        });
    });
    
    // Benchmark binary serialization
    group.bench_function("to_binary", |b| {
        b.iter(|| {
            let bytes = bincode::serialize(&proof).unwrap();
            black_box(bytes)
        });
    });
    
    let bytes = bincode::serialize(&proof).unwrap();
    
    // Benchmark binary deserialization
    group.bench_function("from_binary", |b| {
        b.iter(|| {
            let proof: longfellow_zk::ZkProof<Fp128> = bincode::deserialize(&bytes).unwrap();
            black_box(proof)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_ligero,
    bench_sumcheck,
    bench_montgomery,
    bench_full_system,
    bench_serialization
);
criterion_main!(benches);