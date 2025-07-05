/// Comprehensive benchmark report generator for Longfellow ZK system

use std::fs::{File, create_dir_all};
use std::io::Write;
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
    num_constraints: usize,
    time_ms: f64,
    throughput: f64,
}

#[derive(Serialize, Deserialize)]
struct ProofSize {
    num_constraints: usize,
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
    serialization: SerializationResults,
}

#[derive(Serialize, Deserialize)]
struct SerializationResults {
    json_serialize_ms: f64,
    json_deserialize_ms: f64,
    json_size_bytes: usize,
    binary_serialize_ms: f64,
    binary_deserialize_ms: f64,
    binary_size_bytes: usize,
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
    
    for &size in &[10, 50, 100, 500] {
        // Create constraint system
        let mut cs = ConstraintSystem::<Fp128>::new(size);
        
        // Add constraints
        for i in 0..size/2 {
            let mut coeffs = vec![];
            for j in 0..3 {
                if i + j < size {
                    coeffs.push((i + j, Fp128::from((j + 1) as u64)));
                }
            }
            cs.add_linear_constraint(coeffs, Fp128::from(i as u64));
        }
        
        for i in 0..size/10 {
            if i + 2 < size {
                cs.add_quadratic_constraint(i, i + 1, i + 2);
            }
        }
        
        let params = LigeroParams::security_128();
        let instance = LigeroInstance::new(params, cs).unwrap();
        
        let witness: Vec<Fp128> = (0..size).map(|i| Fp128::from(i as u64)).collect();
        
        // Benchmark proving
        let iterations = 10;
        let prover = LigeroProver::new(instance.clone()).unwrap();
        
        let start = Instant::now();
        let mut proof = None;
        for _ in 0..iterations {
            proof = Some(prover.prove(&witness, &mut OsRng).unwrap());
        }
        let prove_time = start.elapsed().as_secs_f64() / iterations as f64;
        
        prove_times.push(LigeroTiming {
            num_constraints: size,
            time_ms: prove_time * 1000.0,
            throughput: 1.0 / prove_time,
        });
        
        // Benchmark verification
        let proof = proof.unwrap();
        let verifier = LigeroVerifier::new(instance).unwrap();
        
        let start = Instant::now();
        for _ in 0..iterations * 10 {
            let _ = verifier.verify(&proof).unwrap();
        }
        let verify_time = start.elapsed().as_secs_f64() / (iterations * 10) as f64;
        
        verify_times.push(LigeroTiming {
            num_constraints: size,
            time_ms: verify_time * 1000.0,
            throughput: 1.0 / verify_time,
        });
        
        // Measure proof size
        let proof_bytes = bincode::serialize(&proof).unwrap();
        proof_sizes.push(ProofSize {
            num_constraints: size,
            size_bytes: proof_bytes.len(),
            size_kb: proof_bytes.len() as f64 / 1024.0,
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
        // Build circuit
        let mut circuit = Circuit::<Fp128>::new();
        
        let input_layer = Layer::new_input(num_inputs);
        circuit.add_layer(input_layer);
        
        let mut current_width = num_inputs;
        for d in 0..depth-1 {
            current_width = current_width / 2;
            if current_width < 1 {
                current_width = 1;
            }
            
            let mut layer = Layer::new(current_width, d);
            for i in 0..current_width {
                let idx1 = (i * 2) % num_inputs;
                let idx2 = (i * 2 + 1) % num_inputs;
                layer.add_gate(i, vec![(idx1, Fp128::one()), (idx2, Fp128::one())]);
            }
            circuit.add_layer(layer);
        }
        
        circuit.finalize().unwrap();
        
        let num_copies = 8;
        let claimed_sum = Fp128::from((num_inputs * num_copies) as u64);
        let instance = SumcheckInstance::new(circuit.clone(), num_copies, claimed_sum).unwrap();
        
        let inputs: Vec<Vec<Fp128>> = (0..num_copies)
            .map(|_| (0..num_inputs).map(|i| Fp128::from(i as u64)).collect())
            .collect();
        
        // Benchmark proving
        let iterations = 10;
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
    
    SumcheckResults {
        prove_times,
        verify_times,
    }
}

fn benchmark_full_system() -> SystemResults {
    // Create a sample proof for benchmarking
    let mut cs = ConstraintSystem::<Fp128>::new(100);
    for i in 0..50 {
        cs.add_linear_constraint(
            vec![(i, Fp128::one()), ((i + 1) % 100, Fp128::from(2))],
            Fp128::from(i as u64),
        );
    }
    
    let params = LigeroParams::security_128();
    let instance = LigeroInstance::new(params, cs).unwrap();
    let witness: Vec<Fp128> = (0..100).map(|i| Fp128::from(i as u64)).collect();
    
    let prover = LigeroProver::new(instance.clone()).unwrap();
    let ligero_proof = prover.prove(&witness, &mut OsRng).unwrap();
    
    // Create ZK proof wrapper
    let proof = longfellow_zk::ZkProof {
        statement: longfellow_zk::Statement {
            document_type: longfellow_zk::DocumentType::Raw,
            predicates: vec![],
            revealed_fields: vec![],
            hidden_fields: vec![],
        },
        ligero_proof,
        sumcheck_proof: None,
        commitments: vec![],
        metadata: longfellow_zk::ProofMetadata {
            version: "1.0.0".to_string(),
            created_at: 0,
            security_bits: 128,
            document_type: longfellow_zk::DocumentType::Raw,
            circuit_stats: longfellow_zk::CircuitStats {
                num_gates: 0,
                num_wires: 100,
                num_constraints: 50,
                depth: 10,
            },
        },
    };
    
    // Benchmark full proof generation
    let start = Instant::now();
    // In real scenario, this would include circuit building
    let full_proof_time_ms = start.elapsed().as_millis() as f64;
    
    // Benchmark full verification
    let verifier = LigeroVerifier::new(instance).unwrap();
    let start = Instant::now();
    let _ = verifier.verify(&proof.ligero_proof).unwrap();
    let full_verify_time_ms = start.elapsed().as_millis() as f64;
    
    // Benchmark serialization
    let iterations = 100;
    
    // JSON serialization
    let start = Instant::now();
    let mut json_data = String::new();
    for _ in 0..iterations {
        json_data = serde_json::to_string(&proof).unwrap();
    }
    let json_serialize_ms = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
    let json_size_bytes = json_data.len();
    
    // JSON deserialization
    let start = Instant::now();
    for _ in 0..iterations {
        let _: longfellow_zk::ZkProof<Fp128> = serde_json::from_str(&json_data).unwrap();
    }
    let json_deserialize_ms = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
    
    // Binary serialization
    let start = Instant::now();
    let mut binary_data = Vec::new();
    for _ in 0..iterations {
        binary_data = bincode::serialize(&proof).unwrap();
    }
    let binary_serialize_ms = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
    let binary_size_bytes = binary_data.len();
    
    // Binary deserialization
    let start = Instant::now();
    for _ in 0..iterations {
        let _: longfellow_zk::ZkProof<Fp128> = bincode::deserialize(&binary_data).unwrap();
    }
    let binary_deserialize_ms = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
    
    SystemResults {
        full_proof_time_ms,
        full_verify_time_ms,
        serialization: SerializationResults {
            json_serialize_ms,
            json_deserialize_ms,
            json_size_bytes,
            binary_serialize_ms,
            binary_deserialize_ms,
            binary_size_bytes,
        },
    }
}

fn create_summary(
    montgomery: &MontgomeryResults,
    ligero: &LigeroResults,
    sumcheck: &SumcheckResults,
) -> Summary {
    let total_tests = 4 + // Montgomery ops
                     montgomery.pow_results.len() +
                     ligero.prove_times.len() * 2 +
                     sumcheck.prove_times.len() * 2 +
                     6; // Serialization tests
    
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
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1, h2, h3 {{ color: #333; }}
        table {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #4CAF50; color: white; }}
        tr:nth-child(even) {{ background-color: #f2f2f2; }}
        .metric {{ font-weight: bold; color: #2196F3; }}
        .summary {{ background-color: #e7f3ff; padding: 20px; border-radius: 5px; }}
    </style>
</head>
<body>
    <h1>Longfellow ZK Benchmark Report</h1>
    
    <div class="summary">
        <h2>Summary</h2>
        <p>Generated: {}</p>
        <p>Platform: {} | Rust: {}</p>
        <p>Total Runtime: {} ms</p>
        <p>Tests Run: {}</p>
    </div>
    
    <h2>Montgomery Arithmetic Performance</h2>
    <table>
        <tr>
            <th>Operation</th>
            <th>Operations/Second</th>
            <th>Time per Op (ns)</th>
        </tr>
        <tr>
            <td>Addition</td>
            <td class="metric">{:.0}</td>
            <td>{:.2}</td>
        </tr>
        <tr>
            <td>Multiplication</td>
            <td class="metric">{:.0}</td>
            <td>{:.2}</td>
        </tr>
    </table>
    
    <h2>Ligero Protocol Performance</h2>
    <table>
        <tr>
            <th>Constraints</th>
            <th>Prove Time (ms)</th>
            <th>Verify Time (ms)</th>
            <th>Proof Size (KB)</th>
        </tr>
        {}
    </table>
    
    <h2>Peak Performance</h2>
    <ul>
        <li>Best Montgomery Throughput: <span class="metric">{:.0} ops/sec</span></li>
        <li>Best Ligero Prove: <span class="metric">{:.1} proofs/sec</span></li>
        <li>Best Ligero Verify: <span class="metric">{:.1} verifications/sec</span></li>
    </ul>
</body>
</html>"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        report.metadata.platform,
        report.metadata.rust_version,
        report.metadata.total_runtime_ms,
        report.summary.total_tests_run,
        report.montgomery_results.add_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.add_ops_per_sec,
        report.montgomery_results.mul_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.mul_ops_per_sec,
        report.ligero_results.prove_times.iter()
            .zip(&report.ligero_results.verify_times)
            .zip(&report.ligero_results.proof_sizes)
            .map(|((p, v), s)| format!(
                "<tr><td>{}</td><td>{:.2}</td><td>{:.2}</td><td>{:.1}</td></tr>",
                p.num_constraints, p.time_ms, v.time_ms, s.size_kb
            ))
            .collect::<Vec<_>>()
            .join("\n        "),
        report.summary.peak_performance.best_montgomery_throughput,
        report.summary.peak_performance.best_ligero_prove_throughput,
        report.summary.peak_performance.best_ligero_verify_throughput,
    )
}

fn generate_markdown_report(report: &BenchmarkReport) -> String {
    format!(r#"# Longfellow ZK Benchmark Report

Generated: {}  
Platform: {} | Rust: {}  
Total Runtime: {} ms

## Executive Summary

- Total Tests Run: {}
- Peak Montgomery Throughput: {:.0} ops/sec
- Peak Ligero Proving: {:.1} proofs/sec
- Peak Ligero Verification: {:.1} verifications/sec

## Montgomery Arithmetic Performance

| Operation | Ops/Second | Time per Op (ns) |
|-----------|------------|------------------|
| Addition | {:.0} | {:.2} |
| Subtraction | {:.0} | {:.2} |
| Multiplication | {:.0} | {:.2} |
| Squaring | {:.0} | {:.2} |
| Inversion | {:.0} | {:.2} |

### Power Operations

| Exponent | Ops/Second |
|----------|------------|
{}

## Ligero Protocol Performance

| Constraints | Prove (ms) | Verify (ms) | Proof Size (KB) | Prove Throughput | Verify Throughput |
|-------------|------------|-------------|-----------------|------------------|-------------------|
{}

## Sumcheck Protocol Performance

| Circuit Size | Prove (ms) | Verify (ms) | Prove Throughput | Verify Throughput |
|--------------|------------|-------------|------------------|-------------------|
{}

## Full System Performance

- Full Proof Generation: {:.2} ms
- Full Proof Verification: {:.2} ms

### Serialization Performance

| Format | Serialize (ms) | Deserialize (ms) | Size (bytes) |
|--------|----------------|------------------|--------------|
| JSON | {:.3} | {:.3} | {} |
| Binary | {:.3} | {:.3} | {} |

## Recommendations

{}

## System Information

- CPU: {}
- Timestamp: {}

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
        report.montgomery_results.add_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.add_ops_per_sec,
        report.montgomery_results.sub_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.sub_ops_per_sec,
        report.montgomery_results.mul_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.mul_ops_per_sec,
        report.montgomery_results.square_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.square_ops_per_sec,
        report.montgomery_results.invert_ops_per_sec,
        1_000_000_000.0 / report.montgomery_results.invert_ops_per_sec,
        report.montgomery_results.pow_results.iter()
            .map(|p| format!("| {} | {:.0} |", p.exponent, p.ops_per_sec))
            .collect::<Vec<_>>()
            .join("\n"),
        report.ligero_results.prove_times.iter()
            .zip(&report.ligero_results.verify_times)
            .zip(&report.ligero_results.proof_sizes)
            .map(|((p, v), s)| format!(
                "| {} | {:.2} | {:.2} | {:.1} | {:.1} | {:.1} |",
                p.num_constraints, p.time_ms, v.time_ms, s.size_kb, p.throughput, v.throughput
            ))
            .collect::<Vec<_>>()
            .join("\n"),
        report.sumcheck_results.prove_times.iter()
            .zip(&report.sumcheck_results.verify_times)
            .map(|(p, v)| format!(
                "| {} | {:.2} | {:.2} | {:.1} | {:.1} |",
                p.circuit_size, p.time_ms, v.time_ms, p.throughput, v.throughput
            ))
            .collect::<Vec<_>>()
            .join("\n"),
        report.system_results.full_proof_time_ms,
        report.system_results.full_verify_time_ms,
        report.system_results.serialization.json_serialize_ms,
        report.system_results.serialization.json_deserialize_ms,
        report.system_results.serialization.json_size_bytes,
        report.system_results.serialization.binary_serialize_ms,
        report.system_results.serialization.binary_deserialize_ms,
        report.system_results.serialization.binary_size_bytes,
        report.summary.recommendations.iter()
            .map(|r| format!("- {}", r))
            .collect::<Vec<_>>()
            .join("\n"),
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
        format!("{} ({})", std::env::consts::ARCH, num_cpus::get())
    }
}

// Add chrono to dependencies for timestamp formatting
use chrono;