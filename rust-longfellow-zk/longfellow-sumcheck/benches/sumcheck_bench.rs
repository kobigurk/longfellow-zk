use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use longfellow_sumcheck::{
    Circuit, Layer, SumcheckInstance, SumcheckOptions,
    Prover, Verifier, multilinear_extension,
};
use longfellow_algebra::Fp128;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn create_test_circuit(num_layers: usize, layer_size: usize) -> Circuit<Fp128> {
    let mut circuit = Circuit::new();
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    // Create layers with random gates
    for i in 0..num_layers {
        let mut layer = Layer::new(layer_size);
        
        // Add some random addition gates
        for _ in 0..layer_size / 2 {
            let left = rng.gen::<usize>() % layer_size;
            let right = rng.gen::<usize>() % layer_size;
            layer.add_addition_gate(left, right);
        }
        
        // Add some random multiplication gates
        for _ in 0..layer_size / 2 {
            let left = rng.gen::<usize>() % layer_size;
            let right = rng.gen::<usize>() % layer_size;
            layer.add_multiplication_gate(left, right);
        }
        
        circuit.add_layer(layer);
    }
    
    circuit
}

fn bench_prover(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sumcheck Prover");
    
    for (num_layers, layer_size) in [(4, 16), (6, 32), (8, 64)].iter() {
        let circuit = create_test_circuit(*num_layers, *layer_size);
        let num_copies = 10;
        let claimed_sum = Fp128::from(12345);
        
        let instance = SumcheckInstance::new(
            circuit.clone(),
            num_copies,
            claimed_sum,
        ).unwrap();
        
        // Create input values
        let input_size = layer_size * num_copies;
        let input_values: Vec<Fp128> = (0..input_size)
            .map(|i| Fp128::from(i as u64))
            .collect();
        
        let options = SumcheckOptions::default();
        
        group.bench_with_input(
            BenchmarkId::new("prove", format!("{}x{}", num_layers, layer_size)),
            &(*num_layers, *layer_size),
            |bench, _| {
                bench.iter(|| {
                    let mut prover = Prover::new(instance.clone(), input_values.clone(), options.clone());
                    let mut transcript = Vec::new();
                    
                    // Run the protocol
                    let proof = prover.prove(&mut transcript).unwrap();
                    black_box(proof)
                });
            },
        );
    }
    
    group.finish();
}

fn bench_verifier(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sumcheck Verifier");
    
    let num_layers = 6;
    let layer_size = 32;
    let circuit = create_test_circuit(num_layers, layer_size);
    let num_copies = 10;
    let claimed_sum = Fp128::from(12345);
    
    let instance = SumcheckInstance::new(
        circuit.clone(),
        num_copies,
        claimed_sum,
    ).unwrap();
    
    // Generate a proof
    let input_size = layer_size * num_copies;
    let input_values: Vec<Fp128> = (0..input_size)
        .map(|i| Fp128::from(i as u64))
        .collect();
    
    let options = SumcheckOptions::default();
    let mut prover = Prover::new(instance.clone(), input_values.clone(), options.clone());
    let mut transcript = Vec::new();
    let proof = prover.prove(&mut transcript).unwrap();
    
    group.bench_function("verify", |bench| {
        bench.iter(|| {
            let mut verifier = Verifier::new(instance.clone(), options.clone());
            let mut transcript_copy = transcript.clone();
            
            let result = verifier.verify(&proof, &mut transcript_copy).unwrap();
            black_box(result)
        });
    });
    
    group.finish();
}

fn bench_multilinear_extension(c: &mut Criterion) {
    let mut group = c.benchmark_group("Multilinear Extension");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    for num_vars in [4, 6, 8, 10].iter() {
        let size = 1 << num_vars;
        let values: Vec<Fp128> = (0..size)
            .map(|_| Fp128::from(rng.gen::<u64>()))
            .collect();
        
        let point: Vec<Fp128> = (0..*num_vars)
            .map(|_| Fp128::from(rng.gen::<u64>()))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("evaluate", num_vars),
            num_vars,
            |bench, _| {
                bench.iter(|| {
                    multilinear_extension(&values, &point).unwrap()
                });
            },
        );
    }
    
    group.finish();
}

fn bench_layer_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Layer Evaluation");
    
    for layer_size in [16, 32, 64, 128].iter() {
        let mut layer = Layer::new(*layer_size);
        let mut rng = ChaCha20Rng::seed_from_u64(42);
        
        // Add gates
        for i in 0..layer_size / 2 {
            layer.add_addition_gate(i * 2, i * 2 + 1);
        }
        for i in 0..layer_size / 4 {
            layer.add_multiplication_gate(i * 4, i * 4 + 2);
        }
        
        let input_values: Vec<Fp128> = (0..*layer_size)
            .map(|_| Fp128::from(rng.gen::<u64>()))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("evaluate", layer_size),
            layer_size,
            |bench, _| {
                bench.iter(|| {
                    let output = layer.evaluate(&input_values);
                    black_box(output)
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_prover,
    bench_verifier,
    bench_multilinear_extension,
    bench_layer_evaluation
);
criterion_main!(benches);