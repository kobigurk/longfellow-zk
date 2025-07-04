use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use longfellow_ec::{Point, ScalarElement, FieldElement};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn bench_point_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("EC Point");
    
    let gen = Point::generator();
    let point2 = gen.double();
    
    group.bench_function("point_addition", |bench| {
        bench.iter(|| {
            black_box(&gen).add(black_box(&point2))
        });
    });
    
    group.bench_function("point_doubling", |bench| {
        bench.iter(|| {
            black_box(&gen).double()
        });
    });
    
    // Scalar multiplication with small scalar
    let small_scalar = ScalarElement::from_bytes(&[
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 123,
    ]).unwrap();
    
    group.bench_function("scalar_mul_small", |bench| {
        bench.iter(|| {
            black_box(&gen).scalar_mul(black_box(&small_scalar))
        });
    });
    
    // Scalar multiplication with random scalar
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    let mut scalar_bytes = [0u8; 32];
    rng.fill(&mut scalar_bytes);
    let random_scalar = ScalarElement::from_bytes_reduced(&scalar_bytes);
    
    group.bench_function("scalar_mul_random", |bench| {
        bench.iter(|| {
            black_box(&gen).scalar_mul(black_box(&random_scalar))
        });
    });
    
    group.finish();
}

fn bench_field_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("EC Field");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    let mut bytes_a = [0u8; 32];
    let mut bytes_b = [0u8; 32];
    rng.fill(&mut bytes_a);
    rng.fill(&mut bytes_b);
    
    let a = FieldElement::from_bytes(&bytes_a).unwrap_or(FieldElement::one());
    let b = FieldElement::from_bytes(&bytes_b).unwrap_or(FieldElement::one());
    
    group.bench_function("field_addition", |bench| {
        bench.iter(|| {
            black_box(&a).add(black_box(&b))
        });
    });
    
    group.bench_function("field_multiplication", |bench| {
        bench.iter(|| {
            black_box(&a).mul(black_box(&b))
        });
    });
    
    group.bench_function("field_square", |bench| {
        bench.iter(|| {
            black_box(&a).square()
        });
    });
    
    group.bench_function("field_inversion", |bench| {
        bench.iter(|| {
            black_box(&a).invert()
        });
    });
    
    group.finish();
}

fn bench_point_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("EC Validation");
    
    let gen = Point::generator();
    
    group.bench_function("is_on_curve", |bench| {
        bench.iter(|| {
            black_box(&gen).is_on_curve()
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_point_operations,
    bench_field_operations,
    bench_point_validation
);
criterion_main!(benches);