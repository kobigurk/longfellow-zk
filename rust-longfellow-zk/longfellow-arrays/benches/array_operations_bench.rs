use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use longfellow_algebra::{traits::Field, Fp128};
use longfellow_arrays::{Dense, Sparse};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn bench_dense_bind(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dense Array Bind");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    for log_size in [8, 10, 12].iter() {
        let n0 = 1 << log_size;
        let n1 = 256;
        
        let values: Vec<Fp128> = (0..(n0 * n1))
            .map(|_| Fp128::from_u64(rng.gen::<u64>()))
            .collect();
        
        let r = Fp128::from_u64(rng.gen::<u64>());
        
        group.bench_with_input(
            BenchmarkId::new("Rust", n0),
            &n0,
            |b, _| {
                b.iter(|| {
                    let mut dense = Dense::from_vec(n0, n1, values.clone()).unwrap();
                    dense.bind(black_box(r));
                    black_box(dense)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_dense_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dense Array Scale");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    for size in [1000, 10000, 100000].iter() {
        let n0 = (*size as f64).sqrt() as usize;
        let n1 = size / n0;
        
        let values: Vec<Fp128> = (0..(n0 * n1))
            .map(|_| Fp128::from_u64(rng.gen::<u64>()))
            .collect();
        
        let scale_factor = Fp128::from_u64(rng.gen::<u64>());
        
        group.bench_with_input(
            BenchmarkId::new("Rust", size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut dense = Dense::from_vec(n0, n1, values.clone()).unwrap();
                    dense.scale(black_box(scale_factor), black_box(scale_factor));
                    black_box(dense)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_sparse_bind(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sparse Array Bind");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    for (n, num_corners) in [(256, 100), (512, 500), (1024, 1000)].iter() {
        let corners: Vec<(usize, usize, usize, Fp128)> = (0..*num_corners)
            .map(|_| {
                (
                    rng.gen_range(0..*n),
                    rng.gen_range(0..*n),
                    rng.gen_range(0..*n),
                    Fp128::from_u64(rng.gen::<u64>()),
                )
            })
            .collect();
        
        let r = Fp128::from_u64(rng.gen::<u64>());
        
        group.bench_with_input(
            BenchmarkId::new("Rust", format!("n={},corners={}", n, num_corners)),
            n,
            |b, _| {
                b.iter(|| {
                    let mut sparse = Sparse::from_corners(*n, corners.clone()).unwrap();
                    sparse.bind(black_box(r));
                    black_box(sparse)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_sparse_canonicalize(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sparse Array Canonicalize");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    for num_corners in [100, 1000, 10000].iter() {
        let n = 1024;
        
        // Create corners with some duplicates
        let mut sparse = Sparse::<Fp128>::new(n);
        for _ in 0..*num_corners {
            let p0 = rng.gen_range(0..n);
            let p1 = rng.gen_range(0..n);
            let p2 = rng.gen_range(0..n);
            let v = Fp128::from_u64(rng.gen::<u64>());
            sparse.insert(p0, p1, p2, v).unwrap();
        }
        
        // Add some duplicates
        for _ in 0..(num_corners / 10) {
            let idx = rng.gen_range(0..*num_corners);
            sparse.insert(0, 0, 0, Fp128::from_u64(rng.gen::<u64>())).unwrap();
        }
        
        group.bench_with_input(
            BenchmarkId::new("Rust", num_corners),
            num_corners,
            |b, _| {
                b.iter(|| {
                    let mut work = sparse.clone();
                    work.canonicalize();
                    black_box(work)
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_dense_bind,
    bench_dense_scale,
    bench_sparse_bind,
    bench_sparse_canonicalize
);
criterion_main!(benches);