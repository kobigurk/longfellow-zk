use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use longfellow_ligero::*;
use longfellow_algebra::Fp128;
use rand::rngs::OsRng;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

fn create_test_instance(num_witnesses: usize, num_constraints: usize) -> LigeroInstance<Fp128> {
    let mut cs = ConstraintSystem::new(num_witnesses);
    
    // Add linear constraints
    for i in 0..num_constraints / 2 {
        let row = vec![
            (i % num_witnesses, Fp128::from(2)),
            ((i + 1) % num_witnesses, Fp128::from(3)),
            ((i + 2) % num_witnesses, -Fp128::from(5)),
        ];
        cs.add_linear_constraint(row, Fp128::zero());
    }
    
    // Add quadratic constraints
    for i in 0..num_constraints / 2 {
        cs.add_quadratic_constraint(
            i % num_witnesses,
            (i + 1) % num_witnesses,
            (i + 2) % num_witnesses,
        );
    }
    
    let params = LigeroParams::security_128();
    LigeroInstance::new(params, cs).unwrap()
}

fn create_satisfying_witness(num_witnesses: usize) -> Vec<Fp128> {
    // Create a witness that satisfies basic constraints
    let mut witness = vec![Fp128::one(); num_witnesses];
    
    // Set some specific values to satisfy constraints
    for i in 0..num_witnesses.min(10) {
        witness[i] = Fp128::from(i as u64 + 1);
    }
    
    witness
}

fn bench_ligero_prove(c: &mut Criterion) {
    let mut group = c.benchmark_group("ligero_prove");
    
    for &num_witnesses in &[1000, 10000, 100000] {
        let num_constraints = num_witnesses / 10;
        let instance = create_test_instance(num_witnesses, num_constraints);
        let witness = create_satisfying_witness(num_witnesses);
        
        // Adjust constraints to make witness satisfy them
        let prover = LigeroProver::new(instance).unwrap();
        
        group.bench_with_input(
            BenchmarkId::from_parameter(num_witnesses),
            &num_witnesses,
            |b, _| {
                let mut rng = ChaCha20Rng::seed_from_u64(42);
                b.iter(|| {
                    let proof = prover.prove(&witness, &mut rng).unwrap();
                    black_box(proof);
                });
            },
        );
    }
    
    group.finish();
}

fn bench_ligero_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("ligero_verify");
    
    for &num_witnesses in &[1000, 10000, 100000] {
        let num_constraints = num_witnesses / 10;
        let instance = create_test_instance(num_witnesses, num_constraints);
        let witness = create_satisfying_witness(num_witnesses);
        
        let prover = LigeroProver::new(instance.clone()).unwrap();
        let proof = prover.prove(&witness, &mut ChaCha20Rng::seed_from_u64(42)).unwrap();
        
        let verifier = LigeroVerifier::new(instance).unwrap();
        
        group.bench_with_input(
            BenchmarkId::from_parameter(num_witnesses),
            &num_witnesses,
            |b, _| {
                b.iter(|| {
                    let result = verifier.verify(&proof).unwrap();
                    black_box(result);
                });
            },
        );
    }
    
    group.finish();
}

fn bench_tableau_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("tableau_operations");
    
    let params = LigeroParams::security_128();
    
    // Benchmark tableau creation and encoding
    group.bench_function("create_and_encode_10k", |b| {
        let height = 100;
        b.iter(|| {
            let mut tableau = tableau::Tableau::<Fp128>::new(params.clone(), height);
            tableau.encode_rows().unwrap();
            black_box(tableau);
        });
    });
    
    // Benchmark column extraction
    let mut tableau = tableau::Tableau::<Fp128>::new(params.clone(), 100);
    tableau.encode_rows().unwrap();
    
    group.bench_function("extract_columns", |b| {
        b.iter(|| {
            let columns: Vec<Vec<Fp128>> = (0..params.block_enc_size())
                .map(|j| tableau.column(j))
                .collect();
            black_box(columns);
        });
    });
    
    group.finish();
}

fn bench_merkle_tree(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_tree");
    
    for &num_columns in &[100, 1000, 10000] {
        // Create test columns
        let columns: Vec<Vec<Fp128>> = (0..num_columns)
            .map(|i| {
                (0..100)
                    .map(|j| Fp128::from((i * 100 + j) as u64))
                    .collect()
            })
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("build", num_columns),
            &num_columns,
            |b, _| {
                b.iter(|| {
                    let tree = merkle::MerkleTree::new(&columns).unwrap();
                    black_box(tree.root());
                });
            },
        );
        
        // Benchmark proof generation
        let tree = merkle::MerkleTree::new(&columns).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("prove", num_columns),
            &num_columns,
            |b, _| {
                b.iter(|| {
                    let proof = tree.prove(num_columns / 2).unwrap();
                    black_box(proof);
                });
            },
        );
        
        // Benchmark verification
        let proof = tree.prove(num_columns / 2).unwrap();
        let root = tree.root();
        
        group.bench_with_input(
            BenchmarkId::new("verify", num_columns),
            &num_columns,
            |b, _| {
                b.iter(|| {
                    let result = merkle::MerkleTree::verify(
                        &root,
                        num_columns / 2,
                        &columns[num_columns / 2],
                        &proof,
                    );
                    black_box(result);
                });
            },
        );
    }
    
    group.finish();
}

fn bench_constraint_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("constraint_system");
    
    for &size in &[1000, 10000, 100000] {
        let mut cs = ConstraintSystem::<Fp128>::new(size);
        
        // Add constraints
        for i in 0..size / 10 {
            cs.add_linear_constraint(
                vec![(i, Fp128::one()), ((i + 1) % size, Fp128::from(2))],
                Fp128::from(3),
            );
            
            if i < size / 20 {
                cs.add_quadratic_constraint(i, (i + 1) % size, (i + 2) % size);
            }
        }
        
        let witness = create_satisfying_witness(size);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &size,
            |b, _| {
                b.iter(|| {
                    let satisfied = cs.is_satisfied(&witness).unwrap();
                    black_box(satisfied);
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_ligero_prove,
    bench_ligero_verify,
    bench_tableau_operations,
    bench_merkle_tree,
    bench_constraint_system
);
criterion_main!(benches);