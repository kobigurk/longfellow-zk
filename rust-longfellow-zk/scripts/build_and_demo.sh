#!/bin/bash

# Build and run the complete Longfellow ZK demo with C++ interop

set -e

echo "=== Building Longfellow ZK System ==="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Build Rust components
echo -e "${BLUE}1. Building Rust components...${NC}"
cargo build --release --workspace

# Build interop library
echo -e "${BLUE}2. Building FFI interop library...${NC}"
cd interop-demo
cargo build --release --lib
cd ..

# Generate test proofs
echo -e "${BLUE}3. Generating test proofs...${NC}"
cargo run --release --bin generate_test_proofs -- \
    --output-dir test_proofs \
    --json \
    --binary \
    --count 3

# Build C++ verifier
echo -e "${BLUE}4. Building C++ verifier...${NC}"
mkdir -p interop-demo/cpp/build
cd interop-demo/cpp/build

cmake .. \
    -DCMAKE_BUILD_TYPE=Release \
    -DRUST_LIB_PATH=../../../target/release

make -j$(nproc)
cd ../../..

# Run Rust demo
echo -e "${BLUE}5. Running Rust proof generation demo...${NC}"
cargo run --release --example full_system_demo

# Run C++ verification
echo -e "${BLUE}6. Running C++ verification...${NC}"
for proof in test_proofs/*.json; do
    echo -e "${GREEN}Verifying: $proof${NC}"
    ./interop-demo/cpp/build/example_verifier "$proof"
done

# Run benchmarks
echo -e "${BLUE}7. Running performance benchmarks...${NC}"
echo "This may take a while..."

# Montgomery benchmarks
echo -e "${GREEN}Montgomery arithmetic benchmarks:${NC}"
cargo run --release --example montgomery_benchmarks

# Run comparative benchmarks
echo -e "${GREEN}Comparative benchmarks (Rust vs C++):${NC}"
cargo run --release --bin comparative_benchmark

# Run criterion benchmarks (optional, takes longer)
if [ "$1" == "--full-bench" ]; then
    echo -e "${BLUE}8. Running full criterion benchmarks...${NC}"
    cargo bench
fi

echo -e "${GREEN}=== Demo completed successfully! ===${NC}"
echo -e "${GREEN}Generated proofs are in: test_proofs/${NC}"
echo -e "${GREEN}C++ verifier binary: interop-demo/cpp/build/example_verifier${NC}"