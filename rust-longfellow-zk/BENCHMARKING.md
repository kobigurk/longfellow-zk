# Benchmarking Guide for Longfellow-ZK Rust

This guide provides detailed instructions for benchmarking the Rust implementation and comparing it with the C++ version.

## Quick Start

```bash
# Run all benchmarks
cargo bench

# Compare with C++
cargo run --release --example benchmark_comparison
```

## Benchmark Structure

```
longfellow-algebra/benches/
├── field_arithmetic_bench.rs    # Field operations
├── fft_bench.rs                # FFT and polynomial operations
└── polynomial_bench.rs         # Polynomial arithmetic

longfellow-arrays/benches/
├── array_operations_bench.rs   # Dense and sparse array operations
└── eq_bench.rs                # EQ function benchmarks
```

## Available Benchmarks

### Field Arithmetic
- **Addition**: Benchmarks field element addition
- **Multiplication**: Montgomery multiplication performance
- **Inversion**: Binary GCD-based inversion
- **Batch Inversion**: Montgomery's trick for multiple inversions

### FFT Operations
- **Forward FFT**: Cooley-Tukey algorithm performance
- **Inverse FFT**: Inverse transform with scaling
- **Polynomial Multiplication**: FFT-based multiplication

### Array Operations
- **Dense Bind**: Affine interpolation on dense arrays
- **Dense Scale**: Element-wise scaling
- **Sparse Bind**: Binding operation on sparse arrays
- **Sparse Canonicalize**: Sorting and deduplication

## Running Specific Benchmarks

### By Module
```bash
# Algebra module only
cargo bench -p longfellow-algebra

# Arrays module only  
cargo bench -p longfellow-arrays
```

### By Operation
```bash
# Only FFT benchmarks
cargo bench -- fft

# Only inversion benchmarks
cargo bench -- inversion

# Case-insensitive partial matching
cargo bench -- dense
```

### With Specific Parameters
```bash
# Set benchmark time limit (in seconds)
cargo bench -- --measurement-time 30

# Set sample size
cargo bench -- --sample-size 500

# Quick benchmark (less accurate)
cargo bench -- --quick
```

## Profiling and Analysis

### CPU Profiling
```bash
# Generate perf data
cargo bench --bench field_arithmetic_bench -- --profile-time=30

# Create flamegraph
cargo install flamegraph
cargo flamegraph --bench fft_bench
```

### Memory Profiling
```bash
# Using valgrind (Linux)
valgrind --tool=massif --massif-out-file=massif.out \
    target/release/deps/field_arithmetic_bench-* --bench

# Visualize results
ms_print massif.out
```

### Cache Analysis
```bash
# Using perf (Linux)
perf stat -e cache-misses,cache-references \
    cargo bench --bench array_operations_bench
```

## Comparing Implementations

### Automated Comparison
```bash
# Generate full comparison report
cargo run --release --example benchmark_comparison

# With detailed analysis
cargo run --release --example benchmark_comparison -- --verbose

# Compare specific operations
cargo run --release --example benchmark_comparison -- --filter "fft"
```

### Manual Comparison

1. Run C++ benchmarks:
```bash
cd ../../clang-build-release
./bench_field_arithmetic
./bench_fft
./bench_arrays
```

2. Run Rust benchmarks:
```bash
cd ../rust-longfellow-zk
cargo bench --bench field_arithmetic_bench
cargo bench --bench fft_bench  
cargo bench --bench array_operations_bench
```

3. Compare results using the provided analysis tools.

## Benchmark Development

### Adding New Benchmarks

1. Create benchmark file in `benches/`:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_operation(c: &mut Criterion) {
    c.bench_function("my_operation", |b| {
        b.iter(|| {
            // Code to benchmark
            black_box(my_operation())
        })
    });
}

criterion_group!(benches, bench_my_operation);
criterion_main!(benches);
```

2. Add to `Cargo.toml`:
```toml
[[bench]]
name = "my_bench"
harness = false
```

### Best Practices

1. **Use `black_box`** to prevent compiler optimizations from eliminating code
2. **Benchmark realistic workloads** - use appropriate input sizes
3. **Run multiple times** to ensure consistent results
4. **Control environment** - disable CPU frequency scaling, close other applications
5. **Compare like-for-like** - ensure C++ and Rust benchmarks do the same work

## Performance Tuning

### Compiler Flags
```bash
# Maximum optimization
RUSTFLAGS="-C target-cpu=native -C opt-level=3" cargo bench

# Link-time optimization
RUSTFLAGS="-C lto=fat" cargo bench

# Profile-guided optimization
RUSTFLAGS="-C profile-generate=/tmp/pgo" cargo bench
RUSTFLAGS="-C profile-use=/tmp/pgo" cargo bench
```

### Runtime Configuration
```bash
# Set thread count for parallel operations
RAYON_NUM_THREADS=16 cargo bench

# Disable parallel execution for consistent results
RAYON_NUM_THREADS=1 cargo bench
```

## Troubleshooting

### Inconsistent Results
- Ensure CPU frequency scaling is disabled
- Run with higher sample sizes: `--sample-size 1000`
- Check for background processes

### C++ Benchmark Failures
- Verify C++ library is built in release mode
- Check library paths in `build.rs`
- Ensure FFI bindings are up to date

### Out of Memory
- Reduce benchmark sizes
- Run benchmarks individually
- Increase system swap space

## Benchmark Results Archive

Results are saved in `target/criterion/`. To archive results:

```bash
# Save current results
cp -r target/criterion benchmark_results_$(date +%Y%m%d)

# Compare with archived results
cargo bench -- --baseline benchmark_results_20240115
```