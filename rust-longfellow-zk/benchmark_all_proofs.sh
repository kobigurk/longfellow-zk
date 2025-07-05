#!/bin/bash

# Comprehensive benchmark script for all Longfellow ZK proof types

echo "🚀 Longfellow ZK Comprehensive Benchmark"
echo "========================================"
echo "Date: $(date)"
echo ""

# Create output directory
mkdir -p benchmark_results

# Function to run and time a proof generation
run_benchmark() {
    local proof_type=$1
    local output_file="benchmark_results/${proof_type}_proof.json"
    
    echo "📊 Benchmarking: $proof_type"
    
    # Run the proof generation 5 times and capture timing
    total_time=0
    successful_runs=0
    
    for i in {1..5}; do
        start_time=$(date +%s%N)
        
        if cargo run --release --bin full_prover -- --proof-type "$proof_type" --output "$output_file" 2>/dev/null; then
            end_time=$(date +%s%N)
            elapsed_time=$((($end_time - $start_time) / 1000000))
            total_time=$((total_time + elapsed_time))
            successful_runs=$((successful_runs + 1))
            echo "  Run $i: ${elapsed_time}ms ✅"
        else
            echo "  Run $i: FAILED ❌"
        fi
    done
    
    if [ $successful_runs -gt 0 ]; then
        avg_time=$((total_time / successful_runs))
        echo "  Average: ${avg_time}ms ($successful_runs/5 successful)"
        
        # Get proof size
        if [ -f "$output_file" ]; then
            proof_size=$(stat -c%s "$output_file" 2>/dev/null || stat -f%z "$output_file" 2>/dev/null)
            echo "  Proof size: ${proof_size} bytes"
        fi
    else
        echo "  All runs failed"
    fi
    
    echo ""
}

# Test all proof types
echo "=== Basic Proof Types ==="
run_benchmark "field-arithmetic"
run_benchmark "polynomial-commitment"
run_benchmark "merkle-proof"
run_benchmark "elliptic-curve"
run_benchmark "gf2k"

echo "=== Advanced Proof Types ==="
run_benchmark "ligero"
run_benchmark "sumcheck"
run_benchmark "zk-composition"
run_benchmark "combined"

echo "=== Summary ==="
echo "Benchmark complete. Results saved in benchmark_results/"

# Try to run the actual cargo bench for more detailed benchmarks
echo ""
echo "=== Running Cargo Benchmarks ==="
echo "Note: These may take several minutes..."

# Try Ligero benchmarks
echo ""
echo "📊 Running Ligero benchmarks..."
cd longfellow-ligero 2>/dev/null && timeout 60 cargo bench --quiet 2>&1 | grep -E "time:|bench:" | head -20
cd ..

# Try Sumcheck benchmarks  
echo ""
echo "📊 Running Sumcheck benchmarks..."
cd longfellow-sumcheck 2>/dev/null && timeout 60 cargo bench --quiet 2>&1 | grep -E "time:|bench:" | head -20
cd ..

echo ""
echo "✅ All benchmarks complete!"