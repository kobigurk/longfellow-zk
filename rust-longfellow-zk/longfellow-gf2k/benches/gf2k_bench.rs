use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use longfellow_gf2k::Gf2_128;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn bench_gf2k_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Gf2_128");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    let a = Gf2_128::new(rng.gen(), rng.gen());
    let b = Gf2_128::new(rng.gen(), rng.gen());
    
    group.bench_function("addition", |bench| {
        bench.iter(|| {
            black_box(a) + black_box(b)
        });
    });
    
    group.bench_function("multiplication", |bench| {
        bench.iter(|| {
            black_box(a) * black_box(b)
        });
    });
    
    group.bench_function("square", |bench| {
        bench.iter(|| {
            black_box(a).square()
        });
    });
    
    group.bench_function("inversion", |bench| {
        bench.iter(|| {
            black_box(a).invert()
        });
    });
    
    group.finish();
}

fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Gf2_128 Batch");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    for size in [100, 1000, 10000].iter() {
        let a_vec: Vec<Gf2_128> = (0..*size)
            .map(|_| Gf2_128::new(rng.gen(), rng.gen()))
            .collect();
        let b_vec: Vec<Gf2_128> = (0..*size)
            .map(|_| Gf2_128::new(rng.gen(), rng.gen()))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("batch_multiply", size),
            size,
            |bench, _| {
                bench.iter(|| {
                    longfellow_gf2k::batch_multiply(&a_vec, &b_vec)
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_gf2k_operations, bench_batch_operations);
criterion_main!(benches);