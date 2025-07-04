#!/bin/bash

# Super Minimal Interoperability Demo

set -e

echo "🚀 Super Minimal Interoperability Demo"
echo "====================================="
echo

# Build super minimal prover
echo "Building super minimal prover..."
cd /home/kobigurk/dev/longfellow-zk/rust-longfellow-zk/interop-demo
cargo build --release --bin super_minimal --bin proof_format_converter

# Generate proof
echo "Generating proof..."
../target/release/super_minimal

# Convert to C++ format
echo "Converting to C++ format..."
../target/release/proof_format_converter --input proof.json --output proof.bin --format cpp-binary

# Build C++ verifier
echo "Building C++ verifier..."
mkdir -p cpp-verifier/build
cd cpp-verifier/build
cmake .. && make
cd ../..

# Verify proof
echo "Verifying proof..."
if ./cpp-verifier/build/verify_rust_proof proof.bin; then
    echo "✅ Verification successful!"
else
    echo "❌ Verification failed!"
fi