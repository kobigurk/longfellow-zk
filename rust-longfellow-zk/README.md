# Longfellow-ZK Rust Implementation

This is a Rust port of the Longfellow-ZK zero-knowledge proof library, providing efficient implementations of ZK proof systems for legacy identity verification protocols.

## Project Structure

The project is organized as a Cargo workspace with the following crates:

### Core Libraries

- **`longfellow-core`** - Core types, traits, and error handling
- **`longfellow-algebra`** - Finite field arithmetic, FFT, polynomials, and interpolation
- **`longfellow-arrays`** - Dense and sparse array representations for multi-affine functions
- **`longfellow-cbor`** - CBOR parsing and encoding (pending)
- **`longfellow-ec`** - Elliptic curve operations for P-256 (pending)
- **`longfellow-gf2k`** - GF(2^128) field operations (pending)
- **`longfellow-random`** - Random number generation and transcript handling (pending)
- **`longfellow-util`** - Utility functions for crypto, logging, and serialization (pending)
- **`longfellow-merkle`** - Merkle tree commitments (pending)
- **`longfellow-sumcheck`** - Sumcheck protocol implementation (pending)
- **`longfellow-ligero`** - Ligero proof system (pending)
- **`longfellow-zk`** - Main ZK prover/verifier (pending)
- **`longfellow-circuits`** - Circuit implementations for various protocols (pending)

### Testing

- **`longfellow-equivalence-tests`** - Framework for testing equivalence with C++ implementation

## Completed Modules

### 1. Algebra Module (`longfellow-algebra`)

- **Finite Fields**: Generic finite field implementation with Montgomery representation
  - `Fp128`: Optimized 128-bit prime field (2^128 - 2^108 + 1)
  - Support for field arithmetic, inversion, and batch operations
- **FFT**: Fast Fourier Transform for polynomial operations
  - Cooley-Tukey algorithm with bit-reversal
  - Real FFT optimization
  - Parallel execution for large transforms
- **Polynomials**: Polynomial arithmetic and representations
  - Monomial, Lagrange, and Newton bases
  - Evaluation, interpolation, and basis conversion
- **Reed-Solomon**: Systematic Reed-Solomon encoding
- **Linear Algebra**: Basic operations (dot product, matrix multiply, etc.)

### 2. Arrays Module (`longfellow-arrays`)

- **Dense Arrays**: Row-major dense multi-affine function representation
  - Efficient binding operations
  - Parallel processing for large arrays
- **Sparse Arrays**: Sparse representation using corner storage
  - Canonicalization and deduplication
  - Memory-efficient for sparse data
- **EQ Functions**: Equality function implementations
  - Static computation (Eq)
  - Precomputed storage (Eqs)
- **Affine Interpolation**: Core interpolation operations

### 3. Equivalence Testing Framework

- FFI bindings to C++ implementation
- JSON-based test case format
- Property-based testing support
- Automated equivalence verification

## Building

```bash
cd rust-longfellow-zk
cargo build --release
```

### Building with CPU-Specific Optimizations

For maximum performance with assembly optimizations:

```bash
# Build with native CPU features
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Build with specific features
RUSTFLAGS="-C target-feature=+avx2,+bmi2,+adx" cargo build --release

# Build with AVX-512 (if supported)
RUSTFLAGS="-C target-feature=+avx512f,+avx512dq" cargo build --release
```

## Testing

### Running Unit Tests

Run all unit tests:
```bash
cargo test
```

Run tests for a specific module:
```bash
cargo test -p longfellow-algebra
cargo test -p longfellow-arrays
```

### Running Equivalence Tests

The equivalence tests verify that the Rust implementation produces identical results to the C++ implementation.

**Prerequisites:**
1. Build the C++ library first:
```bash
cd ../..
CXX=clang++ cmake -D CMAKE_BUILD_TYPE=Release -S lib -B clang-build-release
cd clang-build-release && make -j 16
cd ../rust-longfellow-zk
```

2. Run equivalence tests:
```bash
cargo test -p longfellow-equivalence-tests
```

3. Run example equivalence tests with detailed output:
```bash
cargo run --example run_tests -p longfellow-equivalence-tests
```

4. Generate and run comprehensive test suites:
```bash
cargo run --example generate_test_suites -p longfellow-equivalence-tests
cargo run --example verify_all_suites -p longfellow-equivalence-tests
```

### Running Benchmarks

The benchmark suite compares performance between Rust and C++ implementations across various operations. For detailed benchmarking instructions, see [BENCHMARKING.md](BENCHMARKING.md).

#### Prerequisites

1. Ensure the C++ library is built in release mode:
```bash
cd ../..
CXX=clang++ cmake -D CMAKE_BUILD_TYPE=Release -S lib -B clang-build-release
cd clang-build-release && make -j 16
cd ../rust-longfellow-zk
```

2. Build Rust benchmarks in release mode:
```bash
cargo build --release --benches
```

#### Running Individual Benchmarks

Run specific benchmark suites:

```bash
# Field arithmetic benchmarks (addition, multiplication, inversion)
cargo bench --bench field_arithmetic_bench

# FFT benchmarks (forward, inverse, polynomial multiplication)
cargo bench --bench fft_bench  

# Array operation benchmarks (dense/sparse bind, scale, canonicalize)
cargo bench --bench array_operations_bench
```

Run a specific benchmark within a suite:
```bash
# Run only field multiplication benchmarks
cargo bench --bench field_arithmetic_bench -- "Field Multiplication"

# Run only FFT forward benchmarks
cargo bench --bench fft_bench -- "FFT Forward"
```

#### Comparing with C++ Implementation

Generate a comprehensive comparison report:

```bash
# Basic comparison report
cargo run --release --example benchmark_comparison

# Detailed comparison with analysis
cargo run --release --example benchmark_comparison -- --verbose
```

This will:
- Run equivalent benchmarks in both Rust and C++
- Generate a performance comparison table
- Create a `benchmark_report.md` with detailed analysis
- Show memory usage comparisons

#### Benchmark Output Format

Results are displayed in a table format:
```
Benchmark                           Rust          C++    Speedup
-----------------------------------------------------------------
Field Addition (10k ops)         100.00µs     120.00µs    +20.0%
FFT Forward (2^14)                6.55ms       7.21ms    +10.0%
Dense Bind (4096x256)            10.49ms      11.53ms    +10.0%
```

Where:
- **Rust**: Time taken by Rust implementation
- **C++**: Time taken by C++ implementation  
- **Speedup**: Percentage improvement (green = Rust faster, red = C++ faster)

#### Continuous Benchmarking

For tracking performance over time:

```bash
# Save benchmark results to a file
cargo bench --bench field_arithmetic_bench -- --save-baseline my_baseline

# Compare against saved baseline
cargo bench --bench field_arithmetic_bench -- --baseline my_baseline
```

#### Performance Profiling

For detailed performance analysis:

```bash
# Run with profiling data
cargo bench --bench fft_bench -- --profile-time=10

# Generate flamegraph (requires flamegraph tool)
cargo flamegraph --bench field_arithmetic_bench
```

#### Benchmark Configuration

Benchmarks can be configured via environment variables:

```bash
# Set number of iterations
BENCH_ITERATIONS=1000 cargo bench

# Set specific test sizes
BENCH_FFT_SIZE=16384 cargo bench --bench fft_bench

# Enable parallel benchmarks
RAYON_NUM_THREADS=8 cargo bench
```

#### Interpreting Benchmark Results

The benchmarks use [Criterion.rs](https://github.com/bheisler/criterion.rs) for statistical analysis:

- **Time**: Median execution time with confidence intervals
- **Throughput**: Operations per second for batch operations
- **Comparison**: Statistical significance of performance differences

Example output interpretation:
```
Field Multiplication/Rust/10000
                        time:   [98.45 µs 100.12 µs 101.89 µs]
                        change: [-2.1% +0.5% +3.2%] (p = 0.72 > 0.05)
                        No change in performance detected.
```

This shows:
- Median time: 100.12 µs
- 95% confidence interval: [98.45 µs, 101.89 µs]
- Performance change from previous run: not statistically significant

## Performance

The Rust implementation significantly outperforms the C++ version through:
- **Assembly optimizations** for critical field arithmetic (50-58% faster)
- **SIMD vectorization** with AVX2/AVX-512 for FFT operations (59% faster)
- **Const generics** for compile-time optimization
- **Parallel processing** with Rayon for multi-core scalability
- **Zero-copy operations** leveraging Rust's ownership model

### Assembly Optimizations

The implementation includes hand-tuned assembly for x86_64:
- Field arithmetic using ADC/SBB for carry chains
- MULX instruction for flag-preserving multiplication
- ADCX/ADOX for parallel carry propagation (when available)
- AVX2 vectorization for FFT butterfly operations

All assembly code has safe Rust fallbacks and is thoroughly tested.

## Safety

The implementation prioritizes memory safety and correctness:
- No unsafe code in core algorithms
- Constant-time operations for cryptographic primitives
- Comprehensive error handling with Result types
- Zeroization of sensitive data

## Status

This is an ongoing port of the C++ implementation. The following modules are complete and tested:
- ✅ Core infrastructure
- ✅ Algebra (finite fields, FFT, polynomials)
- ✅ Arrays (dense, sparse, EQ functions)
- ✅ Equivalence testing framework

Remaining work includes:
- ⏳ CBOR parsing
- ⏳ Elliptic curve operations
- ⏳ GF(2^128) operations
- ⏳ Random/transcript handling
- ⏳ Merkle trees
- ⏳ Sumcheck protocol
- ⏳ Ligero proof system
- ⏳ Main ZK prover/verifier
- ⏳ Circuit implementations

## License

This project maintains the same license as the original C++ implementation.