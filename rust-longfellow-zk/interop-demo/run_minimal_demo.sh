#!/bin/bash

# Minimal Longfellow ZK Interoperability Demonstration
# This script demonstrates basic Rust proof generation → C++ verification

set -e  # Exit on any error

echo "🚀 Minimal Longfellow ZK Interoperability Demonstration"
echo "======================================================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DEMO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="$DEMO_DIR/demo_output"
MINIMAL_PROVER="$DEMO_DIR/target/release/minimal_prover"
PROOF_CONVERTER="$DEMO_DIR/target/release/proof_format_converter"
CPP_VERIFIER="$DEMO_DIR/cpp-verifier/build/verify_rust_proof"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Function to print colored output
print_step() {
    echo -e "${BLUE}==> $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Build Rust components
print_step "Building Rust minimal prover and converter..."

cd "$DEMO_DIR"
if cargo build --release --bin minimal_prover --bin proof_format_converter; then
    print_success "Rust components built successfully"
else
    print_error "Failed to build Rust components"
    exit 1
fi

# Build C++ verifier
print_step "Building C++ verifier..."

mkdir -p "$DEMO_DIR/cpp-verifier/build"
cd "$DEMO_DIR/cpp-verifier/build"

if cmake .. && make; then
    print_success "C++ verifier built successfully"
else
    print_error "Failed to build C++ verifier"
    exit 1
fi

cd "$DEMO_DIR"

# Define proof types to test
PROOF_TYPES=(
    "field-arithmetic"
    "polynomial"
)

# Generate and verify each proof type
echo
print_step "Running minimal interoperability tests..."
echo

TOTAL_TESTS=0
PASSED_TESTS=0

for proof_type in "${PROOF_TYPES[@]}"; do
    echo -e "${YELLOW}📋 Testing proof type: $proof_type${NC}"
    
    # Generate Rust proof
    rust_proof_file="$OUTPUT_DIR/${proof_type}_rust.json"
    cpp_proof_file="$OUTPUT_DIR/${proof_type}_cpp.bin"
    
    echo "  🔧 Generating Rust proof..."
    if "$MINIMAL_PROVER" --proof-type "$proof_type" --output "$rust_proof_file" --security 128; then
        print_success "  Rust proof generated: $rust_proof_file"
    else
        print_error "  Failed to generate Rust proof for $proof_type"
        continue
    fi
    
    # Convert to C++ format
    echo "  🔄 Converting to C++ format..."
    if "$PROOF_CONVERTER" --input "$rust_proof_file" --output "$cpp_proof_file" --format cpp-binary; then
        print_success "  Proof converted: $cpp_proof_file"
    else
        print_error "  Failed to convert proof for $proof_type"
        continue
    fi
    
    # Verify with C++ verifier
    echo "  🔍 Verifying with C++ verifier..."
    if "$CPP_VERIFIER" --verbose "$cpp_proof_file"; then
        print_success "  C++ verification: PASSED"
        ((PASSED_TESTS++))
    else
        print_error "  C++ verification: FAILED"
    fi
    
    ((TOTAL_TESTS++))
    echo
done

# Final summary
echo
echo "🎯 Minimal Interoperability Demonstration Complete"
echo "=================================================="
echo
echo "📊 Results Summary:"
echo "  Total Tests: $TOTAL_TESTS"
echo "  Passed: $PASSED_TESTS"
echo "  Failed: $((TOTAL_TESTS - PASSED_TESTS))"
echo "  Success Rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%"
echo
echo "📁 Output Files:"
echo "  Demo Directory: $OUTPUT_DIR"

if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
    echo
    print_success "🎉 Basic interoperability achieved! Rust proofs ↔ C++ verification working."
    exit 0
else
    echo
    print_error "⚠️  Some tests failed. Check output for details."
    exit 1
fi