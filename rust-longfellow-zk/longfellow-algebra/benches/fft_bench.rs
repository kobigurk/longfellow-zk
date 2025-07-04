use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use longfellow_algebra::{fft::FFT, traits::Field, Fp128};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn get_root_of_unity(n: usize) -> Fp128 {
    // For benchmarking, use a simple value that works
    // In production, this would be a proper nth root of unity
    match n {
        4 => Fp128::from_u64(17166008163159356379u64),
        8 => Fp128::from_u64(123456789u64),
        16 => Fp128::from_u64(987654321u64),
        _ => Fp128::from_u64(35u64),
    }
}

fn bench_fft_forward(c: &mut Criterion) {
    let mut group = c.benchmark_group("FFT Forward");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    for log_size in [8, 10, 12, 14].iter() {
        let size = 1 << log_size;
        let omega = get_root_of_unity(size);
        let fft = FFT::new(size, omega).unwrap();
        
        let mut data: Vec<Fp128> = (0..size)
            .map(|_| Fp128::from_u64(rng.gen::<u64>()))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("Rust", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let mut work = data.clone();
                    fft.forward(&mut work).unwrap();
                    black_box(work)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_fft_inverse(c: &mut Criterion) {
    let mut group = c.benchmark_group("FFT Inverse");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    for log_size in [8, 10, 12, 14].iter() {
        let size = 1 << log_size;
        let omega = get_root_of_unity(size);
        let fft = FFT::new(size, omega).unwrap();
        
        let mut data: Vec<Fp128> = (0..size)
            .map(|_| Fp128::from_u64(rng.gen::<u64>()))
            .collect();
        
        // Transform to frequency domain first
        fft.forward(&mut data).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("Rust", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let mut work = data.clone();
                    fft.inverse(&mut work).unwrap();
                    black_box(work)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_polynomial_multiplication(c: &mut Criterion) {
    let mut group = c.benchmark_group("Polynomial Multiplication (FFT)");
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    
    for degree in [64, 128, 256, 512].iter() {
        let size = 2 * degree; // Need 2n points for n-degree polynomial multiplication
        let omega = get_root_of_unity(size.next_power_of_two());
        
        let poly_a: Vec<Fp128> = (0..*degree)
            .map(|_| Fp128::from_u64(rng.gen::<u64>()))
            .collect();
        let poly_b: Vec<Fp128> = (0..*degree)
            .map(|_| Fp128::from_u64(rng.gen::<u64>()))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("Rust", degree),
            degree,
            |b, _| {
                b.iter(|| {
                    let result = longfellow_algebra::fft::polynomial_multiplication(
                        &poly_a,
                        &poly_b,
                        omega,
                    ).unwrap();
                    black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_fft_forward,
    bench_fft_inverse,
    bench_polynomial_multiplication
);
criterion_main!(benches);