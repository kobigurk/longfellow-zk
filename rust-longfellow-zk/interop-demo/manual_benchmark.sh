#!/bin/bash

echo "🚀 Manual Performance Benchmark for All Proof Types"
echo "=================================================="

# Test existing proofs
declare -A PROOF_FILES=(
    ["field-arithmetic"]="field-arithmetic_cpp.bin"
    ["polynomial"]="polynomial_test.bin"
    ["matrix"]="matrix_test.bin"
    ["hash-chain"]="hash_chain_test.bin"
)

declare -A PROOF_SIZES
declare -A VERIFICATION_TIMES

echo ""
for proof_type in "${!PROOF_FILES[@]}"; do
    bin_file="${PROOF_FILES[$proof_type]}"
    
    if [ -f "demo_output/$bin_file" ]; then
        echo "📊 Testing $proof_type..."
        
        # Get file size
        size=$(stat -c%s "demo_output/$bin_file")
        PROOF_SIZES[$proof_type]=$size
        
        # Run verification 10 times and measure time
        echo "  Running verification tests..."
        total_time=0
        success_count=0
        
        for i in {1..10}; do
            start=$(date +%s%N)
            if ./cpp-verifier/build/verify_rust_proof "demo_output/$bin_file" > /dev/null 2>&1; then
                end=$(date +%s%N)
                time_ms=$(((end - start) / 1000000))
                total_time=$((total_time + time_ms))
                success_count=$((success_count + 1))
            fi
        done
        
        if [ $success_count -gt 0 ]; then
            avg_time=$((total_time / success_count))
            VERIFICATION_TIMES[$proof_type]=$avg_time
            echo "  ✅ Average time: ${avg_time}ms, Size: ${size}B, Success: ${success_count}/10"
        else
            echo "  ❌ All verifications failed"
        fi
    else
        echo "❌ Missing file: demo_output/$bin_file"
    fi
    echo ""
done

echo "📋 Performance Summary"
echo "===================="
echo "| Proof Type | Size (bytes) | Avg Time (ms) | Status |"
echo "|------------|--------------|---------------|--------|"

for proof_type in "${!PROOF_FILES[@]}"; do
    size=${PROOF_SIZES[$proof_type]:-"N/A"}
    time=${VERIFICATION_TIMES[$proof_type]:-"FAILED"}
    status="❌"
    
    if [ "$time" != "FAILED" ]; then
        status="✅"
    fi
    
    printf "| %-10s | %-12s | %-13s | %-6s |\n" "$proof_type" "$size" "$time" "$status"
done

echo ""
echo "🎯 System Performance"
echo "===================="
echo "System: $(uname -srvmo)"
echo "CPU: $(nproc) cores"
echo "Memory: $(free -h | awk '/^Mem:/ {print $2}')"
echo "Compiler: $(g++ --version | head -1)"
echo "Timestamp: $(date)"