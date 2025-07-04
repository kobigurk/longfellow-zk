use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use longfellow_algebra::{traits::Field, Fp128};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn bench_field_addition(c: &mut Criterion) {
    let mut group = c.benchmark_group("Field Addition");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    for size in [100, 1000, 10000].iter() {
        let elements: Vec<Fp128> = (0..*size)
            .map(|_| Fp128::from_u64(rng.gen::<u64>()))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("Rust", size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut sum = Fp128::zero();
                    for elem in &elements {
                        sum += black_box(*elem);
                    }
                    black_box(sum)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_field_multiplication(c: &mut Criterion) {
    let mut group = c.benchmark_group("Field Multiplication");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    for size in [100, 1000, 10000].iter() {
        let elements_a: Vec<Fp128> = (0..*size)
            .map(|_| Fp128::from_u64(rng.gen::<u64>()))
            .collect();
        let elements_b: Vec<Fp128> = (0..*size)
            .map(|_| Fp128::from_u64(rng.gen::<u64>()))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("Rust", size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut results = Vec::with_capacity(elements_a.len());
                    for (a, b) in elements_a.iter().zip(elements_b.iter()) {
                        results.push(black_box(*a * *b));
                    }
                    black_box(results)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_field_inversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("Field Inversion");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    for size in [10, 100, 1000].iter() {
        let elements: Vec<Fp128> = (0..*size)
            .map(|_| Fp128::from_u64(rng.gen_range(1..u64::MAX)))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("Rust", size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut results = Vec::with_capacity(elements.len());
                    for elem in &elements {
                        results.push(black_box(elem.invert().unwrap()));
                    }
                    black_box(results)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_batch_inversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("Batch Inversion");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    for size in [100, 1000, 10000].iter() {
        let mut elements: Vec<Fp128> = (0..*size)
            .map(|_| Fp128::from_u64(rng.gen_range(1..u64::MAX)))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("Rust", size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut batch = elements.clone();
                    Fp128::batch_invert(&mut batch);
                    black_box(batch)
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_field_addition,
    bench_field_multiplication,
    bench_field_inversion,
    bench_batch_inversion
);
criterion_main!(benches);