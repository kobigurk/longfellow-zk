use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use longfellow_algebra::Fp128;

fn bench_field_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Fp128");
    
    let a = Fp128::from_bytes_le(&[0x89, 0x78, 0x56, 0x34, 0x12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
    let b = Fp128::from_bytes_le(&[0x21, 0x43, 0x65, 0x87, 0x09, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
    
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
    
    group.bench_function("inversion", |bench| {
        bench.iter(|| {
            black_box(a).invert()
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_field_operations);
criterion_main!(benches);