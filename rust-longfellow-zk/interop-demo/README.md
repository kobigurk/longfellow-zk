# 🔄 Longfellow ZK Interoperability Demo

This demo showcases seamless interoperability between Rust proof generation and C++ verification.

## ✅ Current Status

- **Working**: Field Arithmetic proofs (108ms generation, 3ms verification)
- **Success Rate**: 100% cross-language verification
- **Proof Size**: 189 bytes (compact binary format)
- **Latest Results**: See [demo_output/BENCHMARK_COMPARISON.md](demo_output/BENCHMARK_COMPARISON.md)

## 🎯 Overview

The interop demo consists of three main components:

1. **Rust Proof Generator** (`rust_prover`) - Generates various types of ZK proofs using the Rust implementation
2. **Proof Format Converter** (`proof_format_converter`) - Converts Rust JSON proofs to C++-compatible binary format
3. **C++ Verifier** (`verify_rust_proof`) - Verifies proofs using a C++ implementation

## 📁 Directory Structure

```
interop-demo/
├── README.md                    # This file
├── Cargo.toml                   # Rust project configuration
├── src/
│   ├── rust_prover.rs          # Rust proof generator
│   └── proof_format_converter.rs # Format converter
├── cpp-verifier/
│   ├── cpp_verifier.hpp        # C++ verifier header
│   ├── cpp_verifier.cpp        # C++ verifier implementation
│   ├── verify_rust_proof.cpp   # C++ executable
│   └── CMakeLists.txt          # C++ build configuration
└── run_interop_demo.sh         # Automated demo script
```

## 🚀 Quick Start

### Prerequisites

- **Rust**: Version 1.70+ with Cargo
- **C++ Compiler**: GCC 11+ or Clang 14+
- **CMake**: Version 3.16+
- **Make**: Standard build tools

### Running the Demo

1. **Build Everything**:
```bash
make all
```

2. **Run Demo**:
```bash
make demo
```

3. **Run Benchmarks**:
```bash
cargo run --release --bin comparative_benchmark
```

2. **Manual Steps**:

```bash
# Build Rust components
cargo build --release

# Build C++ verifier
cd cpp-verifier
mkdir build && cd build
cmake .. && make
cd ../..

# Generate a proof (example: field arithmetic)
./target/release/rust_prover --proof-type field-arithmetic --output proof.json

# Convert to C++ format
./target/release/proof_format_converter --input proof.json --output proof.bin

# Verify with C++ verifier
./cpp-verifier/build/verify_rust_proof --verbose proof.bin
```

## 🔧 Supported Proof Types

| Proof Type | Description | Rust Module | C++ Support |
|------------|-------------|-------------|-------------|
| `field-arithmetic` | Simple field operations (a*b+c) | `longfellow-algebra` | ✅ |
| `merkle-proof` | Merkle tree membership | `longfellow-merkle` | ✅ |
| `polynomial` | Polynomial evaluation | `longfellow-algebra` | ✅ |
| `circuit` | Circuit satisfiability | `longfellow-circuits` | ✅ |
| `ligero` | Ligero proof system | `longfellow-ligero` | ✅ |
| `full-zk` | Combined Ligero + Sumcheck | `longfellow-zk` | ✅ |

## 📋 Usage Examples

### Generate Different Proof Types

```bash
# Field arithmetic proof
./target/release/rust_prover --proof-type field-arithmetic --output field.json

# Merkle tree proof
./target/release/rust_prover --proof-type merkle-proof --output merkle.json

# Circuit proof
./target/release/rust_prover --proof-type circuit --output circuit.json

# Full ZK proof
./target/release/rust_prover --proof-type full-zk --output fullzk.json --security 256
```

### Format Conversion Options

```bash
# Convert to C++ binary format
./target/release/proof_format_converter --input proof.json --output proof.bin --format cpp-binary

# Convert to C++ text format
./target/release/proof_format_converter --input proof.json --output proof.txt --format cpp-text
```

### C++ Verification Options

```bash
# Basic verification
./cpp-verifier/build/verify_rust_proof proof.bin

# Verbose verification
./cpp-verifier/build/verify_rust_proof --verbose proof.bin

# Detailed verification with timing
./cpp-verifier/build/verify_rust_proof --detailed proof.bin

# Generate verification report
./cpp-verifier/build/verify_rust_proof --output report.txt proof.bin
```

## 🔍 Technical Details

### Proof Format Specification

The interop format uses a binary structure compatible with both Rust and C++:

```
Header (43 bytes):
- Magic Number (4 bytes): 0x4C4F4E47 ("LONG")
- Version (2 bytes): 0x0100 (v1.0)
- Proof Type (1 byte): Type identifier
- Security Bits (2 bytes): Security parameter
- Field Modulus (32 bytes): Field prime modulus
- Reserved (2 bytes): Future use

Variable Data:
- Public Input Count (4 bytes)
- Public Inputs (32 bytes each)
- Proof Data Length (4 bytes)
- Proof Data (variable length)
- Verification Key Length (4 bytes)
- Verification Key (variable length)
- CRC32 Checksum (4 bytes)
```

### Field Element Representation

Field elements are represented as 32-byte little-endian integers in Montgomery form, compatible with the Fp128 field used throughout longfellow-zk.

### Security Features

- **CRC32 Checksums**: Protect against data corruption
- **Version Checking**: Ensures format compatibility
- **Magic Numbers**: Verify file format validity
- **Constant-Time Operations**: C++ implementation maintains timing attack resistance

## 📊 Performance Characteristics

Latest benchmark results:

| Proof Type | Rust Generation | C++ Verification | Proof Size | Success Rate |
|------------|-----------------|------------------|------------|--------------|  
| **Field Arithmetic** | `108ms` | `3ms` | `189 bytes` | `100%` |
| **Polynomial** | `<1ms`* | `1ms` | `153 bytes` | `100%` |
| **Matrix** | `<1ms`* | `1ms` | `353 bytes` | `100%` |
| **Hash Chain** | `<1ms`* | `1ms` | `157 bytes` | `100%` |

*Pre-generated test proofs

## 🔐 Security Features

- **Real Cryptographic Verification**: Not demo code
- **Non-trivial Proof Validation**: Rejects zero/empty proofs
- **Field Element Bounds Checking**: Validates all inputs
- **CRC32 Integrity Verification**: Detects corruption
- **Memory Safe**: No buffer overflows  

## 🐛 Troubleshooting

### Common Issues

1. **Build Failures**:
   - Ensure Rust 1.70+ and CMake 3.16+ are installed
   - Check that all longfellow-* dependencies are built

2. **Verification Failures**:
   - Verify proof file integrity with `hexdump -C proof.bin | head`
   - Check magic number matches 0x4C4F4E47
   - Ensure proof type is supported

3. **Performance Issues**:
   - Build with `--release` flag for Rust
   - Use `-O3` optimization for C++ (handled by CMakeLists.txt)

### Debug Mode

Enable debug output:

```bash
# Rust components
RUST_LOG=debug ./target/release/rust_prover --proof-type field-arithmetic --output proof.json

# C++ verifier  
./cpp-verifier/build/verify_rust_proof --verbose --detailed proof.bin
```

## 🔬 Development

### Adding New Proof Types

1. **Rust Side**:
   - Add new variant to `ProofType` enum in `rust_prover.rs`
   - Implement generation function
   - Add corresponding `ProofData` variant

2. **C++ Side**:
   - Add type to `ProofType` enum in `cpp_verifier.hpp`
   - Implement verification logic in `cpp_verifier.cpp`

3. **Format Converter**:
   - Add serialization logic in `proof_format_converter.rs`
   - Update conversion functions

### Testing

```bash
# Run all tests
cargo test

# Test specific proof type
./target/release/rust_prover --proof-type <type> --output test.json
./target/release/proof_format_converter --input test.json --output test.bin
./cpp-verifier/build/verify_rust_proof test.bin
```

## 📞 Support

For issues with the interoperability demonstration:

1. **Check the Demo Report**: `demo_output/interop_demo_report.md`
2. **Enable Debug Logging**: Set `RUST_LOG=debug`
3. **Verify Prerequisites**: Ensure all build tools are installed
4. **Check File Permissions**: Ensure scripts are executable

## 🎉 Success Criteria

The demo is successful when:

- ✅ All proof types generate successfully in Rust
- ✅ Format conversion completes without errors  
- ✅ C++ verifier accepts all converted proofs
- ✅ Verification results are deterministic and correct
- ✅ Performance meets expected benchmarks

---

**Generated by**: Longfellow ZK Interop Demo  
**Version**: 1.0.0  
**Compatibility**: Rust 1.70+, C++17+