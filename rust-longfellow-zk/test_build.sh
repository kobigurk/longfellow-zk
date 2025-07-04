#!/bin/bash

# Test build script for longfellow-zk Rust implementation

echo "Testing Rust longfellow-zk build..."

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust."
    exit 1
fi

# Build all packages
echo "Building all packages..."
cargo build --all 2>&1 | tee build.log

if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo "Build failed! Check build.log for details."
    exit 1
fi

echo "Build successful!"

# Run tests
echo "Running tests..."
cargo test --all 2>&1 | tee test.log

if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo "Tests failed! Check test.log for details."
    exit 1
fi

echo "All tests passed!"

# Check benchmarks compile
echo "Checking benchmarks compile..."
cargo bench --all --no-run 2>&1 | tee bench.log

if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo "Benchmark compilation failed! Check bench.log for details."
    exit 1
fi

echo "All checks passed successfully!"