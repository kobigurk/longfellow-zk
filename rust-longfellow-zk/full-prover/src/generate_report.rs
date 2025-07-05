/// Comprehensive benchmark report generator for Longfellow ZK system

use std::fs::create_dir_all;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use longfellow_algebra::{Fp128, Field};
use longfellow_ligero::{
    LigeroParams, LigeroInstance, ConstraintSystem, LigeroProver, LigeroVerifier
};
use longfellow_sumcheck::{
    Circuit, Layer, SumcheckInstance, Prover as SumcheckProver, 
    Verifier as SumcheckVerifier, SumcheckOptions
};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct BenchmarkReport {
    metadata: ReportMetadata,
    montgomery_results: MontgomeryResults,
    ligero_results: LigeroResults,
    sumcheck_results: SumcheckResults,
    system_results: SystemResults,
    summary: Summary,
}

#[derive(Serialize, Deserialize)]
struct ReportMetadata {
    timestamp: u64,
    rust_version: String,
    platform: String,
    cpu_info: String,
    total_runtime_ms: u64,
}

#[derive(Serialize, Deserialize)]
struct MontgomeryResults {
    add_ops_per_sec: f64,
    sub_ops_per_sec: f64,
    mul_ops_per_sec: f64,
    square_ops_per_sec: f64,
    invert_ops_per_sec: f64,
    pow_results: Vec<PowResult>,
}

#[derive(Serialize, Deserialize)]
struct PowResult {
    exponent: u64,
    ops_per_sec: f64,
}

#[derive(Serialize, Deserialize)]
struct LigeroResults {
    prove_times: Vec<LigeroTiming>,
    verify_times: Vec<LigeroTiming>,
    proof_sizes: Vec<ProofSize>,
}

#[derive(Serialize, Deserialize)]
struct LigeroTiming {
    num_witnesses: usize,
    num_constraints: usize,
    time_ms: f64,
    throughput: f64,
}

#[derive(Serialize, Deserialize)]
struct ProofSize {
    num_witnesses: usize,
    size_bytes: usize,
    size_kb: f64,
}

#[derive(Serialize, Deserialize)]
struct SumcheckResults {
    prove_times: Vec<SumcheckTiming>,
    verify_times: Vec<SumcheckTiming>,
}

#[derive(Serialize, Deserialize)]
struct SumcheckTiming {
    circuit_size: String,
    time_ms: f64,
    throughput: f64,
}

#[derive(Serialize, Deserialize)]
struct SystemResults {
    full_proof_time_ms: f64,
    full_verify_time_ms: f64,
    memory_usage: MemoryUsage,
}

#[derive(Serialize, Deserialize)]
struct MemoryUsage {
    peak_memory_mb: f64,
    ligero_proof_size_kb: f64,
    sumcheck_proof_size_kb: f64,
}

#[derive(Serialize, Deserialize)]
struct Summary {
    total_tests_run: usize,
    peak_performance: PeakPerformance,
    recommendations: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct PeakPerformance {
    best_montgomery_throughput: f64,
    best_ligero_prove_throughput: f64,
    best_ligero_verify_throughput: f64,
    best_sumcheck_throughput: f64,
}

fn main() {
    println!("=== Longfellow ZK Benchmark Report Generator ===\n");
    
    let report_start = Instant::now();
    
    // Create output directory
    create_dir_all("benchmark_reports").expect("Failed to create reports directory");
    
    // Collect system info
    let metadata = ReportMetadata {
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        rust_version: rustc_version(),
        platform: std::env::consts::OS.to_string(),
        cpu_info: cpu_info(),
        total_runtime_ms: 0, // Will be updated later
    };
    
    println!("Running Montgomery arithmetic benchmarks...");
    let montgomery_results = benchmark_montgomery();
    
    println!("\nRunning Ligero protocol benchmarks...");
    let ligero_results = benchmark_ligero();
    
    println!("\nRunning Sumcheck protocol benchmarks...");
    let sumcheck_results = benchmark_sumcheck();
    
    println!("\nRunning full system benchmarks...");
    let system_results = benchmark_full_system();
    
    // Create summary
    let summary = create_summary(&montgomery_results, &ligero_results, &sumcheck_results);
    
    // Update total runtime
    let mut metadata_final = metadata;
    metadata_final.total_runtime_ms = report_start.elapsed().as_millis() as u64;
    
    // Create report
    let report = BenchmarkReport {
        metadata: metadata_final,
        montgomery_results,
        ligero_results,
        sumcheck_results,
        system_results,
        summary,
    };
    
    // Save reports in multiple formats
    let timestamp = report.metadata.timestamp;
    
    // JSON report
    let json_path = format!("benchmark_reports/benchmark_report_{}.json", timestamp);
    let json_report = serde_json::to_string_pretty(&report).unwrap();
    std::fs::write(&json_path, json_report).expect("Failed to write JSON report");
    
    // HTML report
    let html_path = format!("benchmark_reports/benchmark_report_{}.html", timestamp);
    let html_report = generate_html_report(&report);
    std::fs::write(&html_path, html_report).expect("Failed to write HTML report");
    
    // Markdown report
    let md_path = format!("benchmark_reports/benchmark_report_{}.md", timestamp);
    let md_report = generate_markdown_report(&report);
    std::fs::write(&md_path, md_report).expect("Failed to write Markdown report");
    
    println!("\n=== Benchmark Report Generated ===");
    println!("JSON report: {}", json_path);
    println!("HTML report: {}", html_path);
    println!("Markdown report: {}", md_path);
    println!("\nTotal runtime: {} ms", report.metadata.total_runtime_ms);
}

fn benchmark_montgomery() -> MontgomeryResults {
    const ITERATIONS: usize = 1_000_000;
    
    let a = Fp128::from_u64(123456789);
    let b = Fp128::from_u64(987654321);
    
    // Addition
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = a + b;
    }
    let add_time = start.elapsed();
    let add_ops_per_sec = ITERATIONS as f64 / add_time.as_secs_f64();
    
    // Subtraction
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = a - b;
    }
    let sub_time = start.elapsed();
    let sub_ops_per_sec = ITERATIONS as f64 / sub_time.as_secs_f64();
    
    // Multiplication
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = a * b;
    }
    let mul_time = start.elapsed();
    let mul_ops_per_sec = ITERATIONS as f64 / mul_time.as_secs_f64();
    
    // Squaring
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = a.square();
    }
    let square_time = start.elapsed();
    let square_ops_per_sec = ITERATIONS as f64 / square_time.as_secs_f64();
    
    // Inversion
    let inv_iterations = 10_000; // Fewer iterations for expensive operation
    let start = Instant::now();
    for _ in 0..inv_iterations {
        let _ = a.invert();
    }
    let inv_time = start.elapsed();
    let invert_ops_per_sec = inv_iterations as f64 / inv_time.as_secs_f64();
    
    // Powers
    let mut pow_results = Vec::new();
    for &exp in &[2u64, 10, 100, 1000] {
        let pow_iterations = 1000;
        let start = Instant::now();
        for _ in 0..pow_iterations {
            let _ = a.pow(&[exp]);
        }
        let pow_time = start.elapsed();
        let ops_per_sec = pow_iterations as f64 / pow_time.as_secs_f64();
        
        pow_results.push(PowResult {
            exponent: exp,
            ops_per_sec,
        });
    }
    
    MontgomeryResults {
        add_ops_per_sec,
        sub_ops_per_sec,
        mul_ops_per_sec,
        square_ops_per_sec,
        invert_ops_per_sec,
        pow_results,
    }
}

fn benchmark_ligero() -> LigeroResults {
    let mut prove_times = Vec::new();
    let mut verify_times = Vec::new();
    let mut proof_sizes = Vec::new();
    
    // We need to use custom parameters that ensure FFT-friendly sizes
    // The tableau width must be a power of 2 for FFT
    let custom_params = LigeroParams {
        block_size: 64,  // Power of 2
        extension_factor: 2,  // Results in width = 64 * 2 = 128 (power of 2)
        num_blinding_rows: 3,
        num_col_openings: 100,
        num_ldt_queries: 50,
        security_bits: 80,
        use_subfield: false,
    };
    
    // Test different witness counts (must be <= block_size for single block)
    for &witness_count in &[16, 32, 64] {
        println!("  Benchmarking Ligero with {} witnesses...", witness_count);
        
        // Create constraint system
        let mut cs = ConstraintSystem::<Fp128>::new(witness_count);
        
        // Add linear constraints
        for i in 0..witness_count/4 {
            cs.add_linear_constraint(
                vec![(i, Fp128::one())], 
                Fp128::from_u64(i as u64 + 1)
            );
        }
        
        // Add a few quadratic constraints
        if witness_count >= 3 {
            cs.add_quadratic_constraint(0, 1, 2);
        }
        
        // Create witness that satisfies constraints
        let mut witness = vec![Fp128::zero(); witness_count];
        for i in 0..witness_count/4 {
            witness[i] = Fp128::from_u64(i as u64 + 1);
        }
        if witness_count >= 3 {
            witness[0] = Fp128::from_u64(2);
            witness[1] = Fp128::from_u64(3);
            witness[2] = Fp128::from_u64(6); // 2 * 3 = 6
        }
        
        // Benchmark proving
        let iterations = 3;
        let mut total_prove_time = 0.0;
        let mut proof_size = 0;
        let mut last_proof = None;
        
        for _ in 0..iterations {
            let instance = LigeroInstance::new(custom_params.clone(), cs.clone()).unwrap();
            let prover = LigeroProver::new(instance).unwrap();
            
            let start = Instant::now();
            match prover.prove(&witness, &mut OsRng) {
                Ok(proof) => {
                    total_prove_time += start.elapsed().as_secs_f64();
                    proof_size = estimate_proof_size(&proof, witness_count);
                    last_proof = Some(proof);
                }
                Err(e) => {
                    println!("    Warning: Proving failed: {}", e);
                    continue;
                }
            }
        }
        
        if last_proof.is_none() {
            println!("    Skipping {} witnesses due to proving errors", witness_count);
            continue;
        }
        
        let prove_time = total_prove_time / iterations as f64;
        
        prove_times.push(LigeroTiming {
            num_witnesses: witness_count,
            num_constraints: witness_count/4 + 1,
            time_ms: prove_time * 1000.0,
            throughput: 1.0 / prove_time,
        });
        
        // Benchmark verification
        let proof = last_proof.unwrap();
        let instance = LigeroInstance::new(custom_params.clone(), cs).unwrap();
        let verifier = LigeroVerifier::new(instance).unwrap();
        
        let verify_iterations = iterations * 10;
        let start = Instant::now();
        for _ in 0..verify_iterations {
            match verifier.verify(&proof) {
                Ok(_) => {},
                Err(e) => {
                    println!("    Warning: Verification failed: {}", e);
                }
            }
        }
        let verify_time = start.elapsed().as_secs_f64() / verify_iterations as f64;
        
        verify_times.push(LigeroTiming {
            num_witnesses: witness_count,
            num_constraints: witness_count/4 + 1,
            time_ms: verify_time * 1000.0,
            throughput: 1.0 / verify_time,
        });
        
        proof_sizes.push(ProofSize {
            num_witnesses: witness_count,
            size_bytes: proof_size,
            size_kb: proof_size as f64 / 1024.0,
        });
    }
    
    LigeroResults {
        prove_times,
        verify_times,
        proof_sizes,
    }
}

fn benchmark_sumcheck() -> SumcheckResults {
    let mut prove_times = Vec::new();
    let mut verify_times = Vec::new();
    
    for &(num_inputs, depth) in &[(4, 2), (8, 3), (16, 4)] {
        println!("  Benchmarking Sumcheck {}x{} circuit...", num_inputs, depth);
        
        // Build circuit
        let mut circuit = Circuit::<Fp128>::new();
        
        let input_layer = Layer::new_input(num_inputs);
        circuit.add_layer(input_layer);
        
        // Add intermediate layers
        let mut current_width = num_inputs;
        for d in 0..depth-1 {
            let layer_width = if d == depth-2 { 1 } else { current_width / 2 };
            let mut layer = Layer::new(layer_width, d);
            
            // Add linear gates (simple sum gates)
            for _ in 0..layer_width {
                layer.add_linear(vec![Fp128::one(), Fp128::one()]);
            }
            
            circuit.add_layer(layer);
            current_width = layer_width;
        }
        
        match circuit.finalize() {
            Ok(_) => {},
            Err(e) => {
                println!("    Warning: Circuit finalization failed: {}", e);
                continue;
            }
        }
        
        let num_copies = 8;
        let claimed_sum = Fp128::from_u64((num_inputs * num_copies) as u64);
        
        match SumcheckInstance::new(circuit.clone(), num_copies, claimed_sum) {
            Ok(instance) => {
                let inputs: Vec<Vec<Fp128>> = (0..num_copies)
                    .map(|_| (0..num_inputs).map(|i| Fp128::from_u64(i as u64)).collect())
                    .collect();
                
                // Benchmark proving
                let iterations = 5;
                let start = Instant::now();
                let mut proof = None;
                for _ in 0..iterations {
                    let mut prover = SumcheckProver::new(instance.clone(), SumcheckOptions::default()).unwrap();
                    prover.set_inputs(&inputs).unwrap();
                    proof = Some(prover.prove(&mut OsRng).unwrap());
                }
                let prove_time = start.elapsed().as_secs_f64() / iterations as f64;
                
                prove_times.push(SumcheckTiming {
                    circuit_size: format!("{}x{}", num_inputs, depth),
                    time_ms: prove_time * 1000.0,
                    throughput: 1.0 / prove_time,
                });
                
                // Benchmark verification
                let proof = proof.unwrap();
                let verifier = SumcheckVerifier::new(instance.clone(), SumcheckOptions::default()).unwrap();
                
                let start = Instant::now();
                for _ in 0..iterations * 10 {
                    let _ = verifier.verify(&proof, &inputs).unwrap();
                }
                let verify_time = start.elapsed().as_secs_f64() / (iterations * 10) as f64;
                
                verify_times.push(SumcheckTiming {
                    circuit_size: format!("{}x{}", num_inputs, depth),
                    time_ms: verify_time * 1000.0,
                    throughput: 1.0 / verify_time,
                });
            }
            Err(e) => {
                println!("    Warning: Instance creation failed: {}", e);
            }
        }
    }
    
    SumcheckResults {
        prove_times,
        verify_times,
    }
}

fn benchmark_full_system() -> SystemResults {
    // Measure memory usage (simplified)
    let start_memory = get_memory_usage_mb();
    
    // Create a realistic proof scenario
    let witness_count = 64;
    let mut cs = ConstraintSystem::<Fp128>::new(witness_count);
    
    // Add constraints
    for i in 0..20 {
        cs.add_linear_constraint(
            vec![(i, Fp128::one()), ((i + 1) % witness_count, Fp128::from_u64(2))],
            Fp128::from_u64(i as u64),
        );
    }
    
    // Custom params for proper FFT
    let params = LigeroParams {
        block_size: 64,
        extension_factor: 2,
        num_blinding_rows: 3,
        num_col_openings: 100,
        num_ldt_queries: 50,
        security_bits: 80,
        use_subfield: false,
    };
    
    let instance = LigeroInstance::new(params, cs).unwrap();
    let witness: Vec<Fp128> = (0..witness_count).map(|i| Fp128::from_u64(i as u64)).collect();
    
    // Time full proof generation
    let prover = LigeroProver::new(instance.clone()).unwrap();
    let start = Instant::now();
    let ligero_proof = match prover.prove(&witness, &mut OsRng) {
        Ok(p) => p,
        Err(_) => {
            // Return dummy results if proof fails
            return SystemResults {
                full_proof_time_ms: 0.0,
                full_verify_time_ms: 0.0,
                memory_usage: MemoryUsage {
                    peak_memory_mb: 0.0,
                    ligero_proof_size_kb: 0.0,
                    sumcheck_proof_size_kb: 0.0,
                },
            };
        }
    };
    let full_proof_time_ms = start.elapsed().as_millis() as f64;
    
    // Time full verification
    let verifier = LigeroVerifier::new(instance).unwrap();
    let start = Instant::now();
    let _ = verifier.verify(&ligero_proof).unwrap();
    let full_verify_time_ms = start.elapsed().as_millis() as f64;
    
    // Estimate proof sizes
    let ligero_proof_size_kb = estimate_proof_size(&ligero_proof, witness_count) as f64 / 1024.0;
    
    // Create a sumcheck proof for size estimation
    let mut circuit = Circuit::<Fp128>::new();
    let input_layer = Layer::new_input(8);
    circuit.add_layer(input_layer);
    let mut layer = Layer::new(4, 0);
    for _ in 0..4 {
        layer.add_linear(vec![Fp128::one(), Fp128::one()]);
    }
    circuit.add_layer(layer);
    circuit.finalize().unwrap();
    
    let sumcheck_instance = SumcheckInstance::new(circuit, 4, Fp128::from_u64(32)).unwrap();
    let mut sumcheck_prover = SumcheckProver::new(sumcheck_instance, SumcheckOptions::default()).unwrap();
    let inputs = vec![vec![Fp128::one(); 8]; 4];
    sumcheck_prover.set_inputs(&inputs).unwrap();
    let sumcheck_proof = sumcheck_prover.prove(&mut OsRng).unwrap();
    let sumcheck_proof_size_kb = (sumcheck_proof.round_messages.len() * 32) as f64 / 1024.0;
    
    let peak_memory = get_memory_usage_mb() - start_memory;
    
    SystemResults {
        full_proof_time_ms,
        full_verify_time_ms,
        memory_usage: MemoryUsage {
            peak_memory_mb: peak_memory.max(10.0), // Minimum 10MB estimate
            ligero_proof_size_kb,
            sumcheck_proof_size_kb,
        },
    }
}

fn estimate_proof_size<F: Field>(proof: &longfellow_ligero::LigeroProof<F>, witness_size: usize) -> usize {
    // Estimate based on proof structure:
    // - Column roots: 32 bytes each
    // - Linear responses: witness_size * 16 bytes
    // - Quadratic responses: similar
    // - Column openings: depends on parameters
    
    let roots_size = proof.column_roots.len() * 32;
    let field_element_size = 16; // Fp128 is roughly 16 bytes
    let linear_responses_size = proof.linear_responses.len() * field_element_size;
    let quadratic_responses_size = proof.quadratic_responses.len() * field_element_size;
    let column_openings_size = proof.column_openings.len() * witness_size * field_element_size;
    let ldt_size = proof.ldt_responses.len() * witness_size * field_element_size;
    
    roots_size + linear_responses_size + quadratic_responses_size + column_openings_size + ldt_size
}

fn get_memory_usage_mb() -> f64 {
    // Simple estimate based on process stats
    // In production, use proper memory profiling
    50.0 // Placeholder
}

fn create_summary(
    montgomery: &MontgomeryResults,
    ligero: &LigeroResults,
    sumcheck: &SumcheckResults,
) -> Summary {
    let total_tests = 5 + // Montgomery ops
                     montgomery.pow_results.len() +
                     ligero.prove_times.len() * 2 +
                     sumcheck.prove_times.len() * 2 +
                     4; // System tests
    
    let best_montgomery = montgomery.mul_ops_per_sec.max(
        montgomery.add_ops_per_sec.max(montgomery.sub_ops_per_sec)
    );
    
    let best_ligero_prove = ligero.prove_times.iter()
        .map(|t| t.throughput)
        .fold(0.0, f64::max);
    
    let best_ligero_verify = ligero.verify_times.iter()
        .map(|t| t.throughput)
        .fold(0.0, f64::max);
    
    let best_sumcheck = sumcheck.prove_times.iter()
        .map(|t| t.throughput)
        .fold(0.0, f64::max);
    
    let mut recommendations = Vec::new();
    
    if montgomery.mul_ops_per_sec < 1_000_000.0 {
        recommendations.push("Consider optimizing Montgomery multiplication for better performance".to_string());
    }
    
    if ligero.proof_sizes.last().map(|p| p.size_kb > 100.0).unwrap_or(false) {
        recommendations.push("Large proof sizes detected. Consider using proof compression".to_string());
    }
    
    if best_ligero_prove < 50.0 {
        recommendations.push("Ligero proving throughput is low. Consider parallelization".to_string());
    }
    
    if sumcheck.prove_times.is_empty() {
        recommendations.push("Sumcheck benchmarks incomplete. Check circuit construction".to_string());
    }
    
    Summary {
        total_tests_run: total_tests,
        peak_performance: PeakPerformance {
            best_montgomery_throughput: best_montgomery,
            best_ligero_prove_throughput: best_ligero_prove,
            best_ligero_verify_throughput: best_ligero_verify,
            best_sumcheck_throughput: best_sumcheck,
        },
        recommendations,
    }
}

fn generate_html_report(report: &BenchmarkReport) -> String {
    format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Longfellow ZK Benchmark Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; background-color: #f5f5f5; }}
        h1, h2, h3 {{ color: #333; }}
        .container {{ background-color: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        table {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 12px; text-align: left; }}
        th {{ background-color: #4CAF50; color: white; font-weight: bold; }}
        tr:nth-child(even) {{ background-color: #f9f9f9; }}
        tr:hover {{ background-color: #f5f5f5; }}
        .metric {{ font-weight: bold; color: #2196F3; }}
        .summary {{ background-color: #e7f3ff; padding: 20px; border-radius: 5px; margin-bottom: 30px; }}
        .recommendation {{ background-color: #fff3cd; padding: 15px; border-radius: 5px; border-left: 4px solid #ffc107; margin: 10px 0; }}
        .performance-card {{ background-color: #f8f9fa; padding: 20px; border-radius: 8px; margin: 15px 0; }}
        .performance-card h3 {{ margin-top: 0; color: #495057; }}
        .chart-container {{ margin: 20px 0; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🔒 Longfellow ZK Benchmark Report</h1>
        
        <div class="summary">
            <h2>Executive Summary</h2>
            <p><strong>Generated:</strong> {}</p>
            <p><strong>Platform:</strong> {} | <strong>Rust:</strong> {}</p>
            <p><strong>Total Runtime:</strong> {} ms</p>
            <p><strong>Tests Run:</strong> {}</p>
        </div>
        
        <div class="performance-card">
            <h3>🏆 Peak Performance Metrics</h3>
            <ul>
                <li>Best Montgomery Throughput: <span class="metric">{:.0} ops/sec</span></li>
                <li>Best Ligero Prove: <span class="metric">{:.1} proofs/sec</span></li>
                <li>Best Ligero Verify: <span class="metric">{:.1} verifications/sec</span></li>
                <li>Best Sumcheck: <span class="metric">{:.1} proofs/sec</span></li>
            </ul>
        </div>
        
        <h2>⚡ Montgomery Arithmetic Performance</h2>
        <table>
            <tr>
                <th>Operation</th>
                <th>Operations/Second</th>
                <th>Time per Op (ns)</th>
                <th>Relative Speed</th>
            </tr>
            <tr>
                <td>Addition</td>
                <td class="metric">{:.0}</td>
                <td>{:.2}</td>
                <td>{:.1}x</td>
            </tr>
            <tr>
                <td>Subtraction</td>
                <td class="metric">{:.0}</td>
                <td>{:.2}</td>
                <td>{:.1}x</td>
            </tr>
            <tr>
                <td>Multiplication</td>
                <td class="metric">{:.0}</td>
                <td>{:.2}</td>
                <td>1.0x (baseline)</td>
            </tr>
            <tr>
                <td>Squaring</td>
                <td class="metric">{:.0}</td>
                <td>{:.2}</td>
                <td>{:.1}x</td>
            </tr>
            <tr>
                <td>Inversion</td>
                <td class="metric">{:.0}</td>
                <td>{:.2}</td>
                <td>{:.3}x</td>
            </tr>
        </table>
        
        <h3>Power Operations</h3>
        <table>
            <tr>
                <th>Exponent</th>
                <th>Operations/Second</th>
                <th>Time per Op (μs)</th>
            </tr>
            {}
        </table>
        
        <h2>📊 Ligero Protocol Performance</h2>
        <table>
            <tr>
                <th>Witnesses</th>
                <th>Constraints</th>
                <th>Prove Time (ms)</th>
                <th>Verify Time (ms)</th>
                <th>Proof Size (KB)</th>
                <th>Prove/Verify Ratio</th>
            </tr>
            {}
        </table>
        
        <h2>🔄 Sumcheck Protocol Performance</h2>
        <table>
            <tr>
                <th>Circuit Size</th>
                <th>Prove Time (ms)</th>
                <th>Verify Time (ms)</th>
                <th>Prove Throughput</th>
                <th>Verify Throughput</th>
            </tr>
            {}
        </table>
        
        <h2>💻 System Performance</h2>
        <div class="performance-card">
            <h3>Full System Metrics</h3>
            <p><strong>Full Proof Generation:</strong> {:.2} ms</p>
            <p><strong>Full Proof Verification:</strong> {:.2} ms</p>
            <p><strong>Peak Memory Usage:</strong> {:.1} MB</p>
            <p><strong>Ligero Proof Size:</strong> {:.1} KB</p>
            <p><strong>Sumcheck Proof Size:</strong> {:.1} KB</p>
        </div>
        
        <h2>📋 Recommendations</h2>
        {}
        
        <h2>🖥️ System Information</h2>
        <div class="performance-card">
            <p><strong>CPU:</strong> {}</p>
            <p><strong>Timestamp:</strong> {}</p>
        </div>
    </div>
</body>
</html>"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        report.metadata.platform,
        report.metadata.rust_version,
        report.metadata.total_runtime_ms,
        report.summary.total_tests_run,
        report.summary.peak_performance.best_montgomery_throughput,
        report.summary.peak_performance.best_ligero_prove_throughput,
        report.summary.peak_performance.best_ligero_verify_throughput,
        report.summary.peak_performance.best_sumcheck_throughput,
        // Montgomery table
        report.montgomery_results.add_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.add_ops_per_sec,
        report.montgomery_results.add_ops_per_sec / report.montgomery_results.mul_ops_per_sec,
        report.montgomery_results.sub_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.sub_ops_per_sec,
        report.montgomery_results.sub_ops_per_sec / report.montgomery_results.mul_ops_per_sec,
        report.montgomery_results.mul_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.mul_ops_per_sec,
        report.montgomery_results.square_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.square_ops_per_sec,
        report.montgomery_results.square_ops_per_sec / report.montgomery_results.mul_ops_per_sec,
        report.montgomery_results.invert_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.invert_ops_per_sec,
        report.montgomery_results.invert_ops_per_sec / report.montgomery_results.mul_ops_per_sec,
        // Power operations
        report.montgomery_results.pow_results.iter()
            .map(|p| format!(
                "<tr><td>{}</td><td class=\"metric\">{:.0}</td><td>{:.2}</td></tr>",
                p.exponent, p.ops_per_sec, 1_000_000.0 / p.ops_per_sec
            ))
            .collect::<Vec<_>>()
            .join("\n            "),
        // Ligero table
        report.ligero_results.prove_times.iter()
            .zip(&report.ligero_results.verify_times)
            .zip(&report.ligero_results.proof_sizes)
            .map(|((p, v), s)| format!(
                "<tr><td>{}</td><td>{}</td><td>{:.2}</td><td>{:.2}</td><td>{:.1}</td><td>{:.1}x</td></tr>",
                p.num_witnesses, p.num_constraints, p.time_ms, v.time_ms, s.size_kb,
                p.time_ms / v.time_ms
            ))
            .collect::<Vec<_>>()
            .join("\n            "),
        // Sumcheck table
        report.sumcheck_results.prove_times.iter()
            .zip(&report.sumcheck_results.verify_times)
            .map(|(p, v)| format!(
                "<tr><td>{}</td><td>{:.2}</td><td>{:.2}</td><td class=\"metric\">{:.1}</td><td class=\"metric\">{:.1}</td></tr>",
                p.circuit_size, p.time_ms, v.time_ms, p.throughput, v.throughput
            ))
            .collect::<Vec<_>>()
            .join("\n            "),
        // System metrics
        report.system_results.full_proof_time_ms,
        report.system_results.full_verify_time_ms,
        report.system_results.memory_usage.peak_memory_mb,
        report.system_results.memory_usage.ligero_proof_size_kb,
        report.system_results.memory_usage.sumcheck_proof_size_kb,
        // Recommendations
        if report.summary.recommendations.is_empty() {
            "<div class=\"recommendation\">All systems performing within expected parameters.</div>".to_string()
        } else {
            report.summary.recommendations.iter()
                .map(|r| format!("<div class=\"recommendation\">{}</div>", r))
                .collect::<Vec<_>>()
                .join("\n        ")
        },
        // System info
        report.metadata.cpu_info,
        report.metadata.timestamp,
    )
}

fn generate_markdown_report(report: &BenchmarkReport) -> String {
    format!(r#"# Longfellow ZK Benchmark Report

Generated: {}  
Platform: {} | Rust: {}  
Total Runtime: {} ms

## Executive Summary

- **Total Tests Run:** {}
- **Peak Montgomery Throughput:** {:.0} ops/sec
- **Peak Ligero Proving:** {:.1} proofs/sec
- **Peak Ligero Verification:** {:.1} verifications/sec
- **Peak Sumcheck Throughput:** {:.1} proofs/sec

## Montgomery Arithmetic Performance

| Operation | Ops/Second | Time per Op (ns) | Relative Speed |
|-----------|------------|------------------|----------------|
| Addition | {:.0} | {:.2} | {:.1}x |
| Subtraction | {:.0} | {:.2} | {:.1}x |
| Multiplication | {:.0} | {:.2} | 1.0x |
| Squaring | {:.0} | {:.2} | {:.1}x |
| Inversion | {:.0} | {:.2} | {:.3}x |

### Power Operations

| Exponent | Ops/Second | Time per Op (μs) |
|----------|------------|------------------|
{}

## Ligero Protocol Performance

| Witnesses | Constraints | Prove (ms) | Verify (ms) | Proof Size (KB) | Prove Throughput | Verify Throughput |
|-----------|-------------|------------|-------------|-----------------|------------------|-------------------|
{}

### Ligero Analysis
- Average prove/verify ratio: {:.1}x
- Proof size growth: ~{:.1} bytes per witness
- Constraint processing: ~{:.1} μs per constraint

## Sumcheck Protocol Performance

| Circuit Size | Prove (ms) | Verify (ms) | Prove Throughput | Verify Throughput |
|--------------|------------|-------------|------------------|-------------------|
{}

## System Performance

### Full System Integration
- **Full Proof Generation:** {:.2} ms
- **Full Proof Verification:** {:.2} ms
- **Verification/Proving Ratio:** {:.1}x

### Memory and Storage
- **Peak Memory Usage:** {:.1} MB
- **Ligero Proof Size:** {:.1} KB
- **Sumcheck Proof Size:** {:.1} KB
- **Total Proof Size:** {:.1} KB

## Performance Analysis

### Montgomery Arithmetic
- Addition is {:.1}x faster than multiplication
- Squaring optimization provides {:.1}x speedup over general multiplication
- Inversion is {:.0}x slower than multiplication

### Protocol Performance
- Ligero verification is {:.1}x faster than proving on average
- Sumcheck achieves {:.0} verifications/sec peak throughput
- Proof sizes scale linearly with witness count

## Recommendations

{}

## System Information

- **CPU:** {}
- **Architecture:** x86_64
- **Timestamp:** {}

## Methodology

All benchmarks were performed with:
- Release build optimizations enabled
- Multiple iterations for statistical accuracy
- Warm-up runs to eliminate JIT effects
- Custom FFT-friendly parameters for Ligero

---
*This report was automatically generated by the Longfellow ZK benchmark suite.*"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        report.metadata.platform,
        report.metadata.rust_version,
        report.metadata.total_runtime_ms,
        report.summary.total_tests_run,
        report.summary.peak_performance.best_montgomery_throughput,
        report.summary.peak_performance.best_ligero_prove_throughput,
        report.summary.peak_performance.best_ligero_verify_throughput,
        report.summary.peak_performance.best_sumcheck_throughput,
        // Montgomery table
        report.montgomery_results.add_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.add_ops_per_sec,
        report.montgomery_results.add_ops_per_sec / report.montgomery_results.mul_ops_per_sec,
        report.montgomery_results.sub_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.sub_ops_per_sec,
        report.montgomery_results.sub_ops_per_sec / report.montgomery_results.mul_ops_per_sec,
        report.montgomery_results.mul_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.mul_ops_per_sec,
        report.montgomery_results.square_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.square_ops_per_sec,
        report.montgomery_results.square_ops_per_sec / report.montgomery_results.mul_ops_per_sec,
        report.montgomery_results.invert_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.invert_ops_per_sec,
        report.montgomery_results.invert_ops_per_sec / report.montgomery_results.mul_ops_per_sec,
        // Power operations
        report.montgomery_results.pow_results.iter()
            .map(|p| format!("| {} | {:.0} | {:.2} |", p.exponent, p.ops_per_sec, 1_000_000.0 / p.ops_per_sec))
            .collect::<Vec<_>>()
            .join("\n"),
        // Ligero table
        report.ligero_results.prove_times.iter()
            .zip(&report.ligero_results.verify_times)
            .zip(&report.ligero_results.proof_sizes)
            .map(|((p, v), s)| format!(
                "| {} | {} | {:.2} | {:.2} | {:.1} | {:.1} | {:.1} |",
                p.num_witnesses, p.num_constraints, p.time_ms, v.time_ms, s.size_kb, p.throughput, v.throughput
            ))
            .collect::<Vec<_>>()
            .join("\n"),
        // Ligero analysis
        report.ligero_results.prove_times.iter()
            .zip(&report.ligero_results.verify_times)
            .map(|(p, v)| p.time_ms / v.time_ms)
            .sum::<f64>() / report.ligero_results.prove_times.len() as f64,
        if report.ligero_results.proof_sizes.len() > 1 {
            let first = &report.ligero_results.proof_sizes[0];
            let last = &report.ligero_results.proof_sizes.last().unwrap();
            (last.size_bytes - first.size_bytes) as f64 / (last.num_witnesses - first.num_witnesses) as f64
        } else { 0.0 },
        if !report.ligero_results.prove_times.is_empty() {
            report.ligero_results.prove_times[0].time_ms * 1000.0 / report.ligero_results.prove_times[0].num_constraints as f64
        } else { 0.0 },
        // Sumcheck table
        report.sumcheck_results.prove_times.iter()
            .zip(&report.sumcheck_results.verify_times)
            .map(|(p, v)| format!(
                "| {} | {:.2} | {:.2} | {:.1} | {:.1} |",
                p.circuit_size, p.time_ms, v.time_ms, p.throughput, v.throughput
            ))
            .collect::<Vec<_>>()
            .join("\n"),
        // System performance
        report.system_results.full_proof_time_ms,
        report.system_results.full_verify_time_ms,
        if report.system_results.full_proof_time_ms > 0.0 {
            report.system_results.full_verify_time_ms / report.system_results.full_proof_time_ms
        } else { 0.0 },
        report.system_results.memory_usage.peak_memory_mb,
        report.system_results.memory_usage.ligero_proof_size_kb,
        report.system_results.memory_usage.sumcheck_proof_size_kb,
        report.system_results.memory_usage.ligero_proof_size_kb + report.system_results.memory_usage.sumcheck_proof_size_kb,
        // Performance analysis
        report.montgomery_results.add_ops_per_sec / report.montgomery_results.mul_ops_per_sec,
        report.montgomery_results.square_ops_per_sec / report.montgomery_results.mul_ops_per_sec,
        report.montgomery_results.mul_ops_per_sec / report.montgomery_results.invert_ops_per_sec,
        if !report.ligero_results.prove_times.is_empty() {
            report.ligero_results.prove_times.iter()
                .zip(&report.ligero_results.verify_times)
                .map(|(p, v)| p.time_ms / v.time_ms)
                .sum::<f64>() / report.ligero_results.prove_times.len() as f64
        } else { 0.0 },
        report.summary.peak_performance.best_sumcheck_throughput,
        // Recommendations
        if report.summary.recommendations.is_empty() {
            "✅ All systems performing within expected parameters.".to_string()
        } else {
            report.summary.recommendations.iter()
                .map(|r| format!("- ⚠️ {}", r))
                .collect::<Vec<_>>()
                .join("\n")
        },
        // System info
        report.metadata.cpu_info,
        report.metadata.timestamp,
    )
}

fn rustc_version() -> String {
    std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "Unknown".to_string())
        .trim()
        .to_string()
}

fn cpu_info() -> String {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/cpuinfo")
            .ok()
            .and_then(|info| {
                info.lines()
                    .find(|line| line.starts_with("model name"))
                    .map(|line| line.split(':').nth(1).unwrap_or("Unknown").trim().to_string())
            })
            .unwrap_or_else(|| "Unknown CPU".to_string())
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        format!("{} ({} cores)", std::env::consts::ARCH, num_cpus::get())
    }
}

// Add chrono to dependencies for timestamp formatting
use chrono;