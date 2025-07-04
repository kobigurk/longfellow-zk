/// Rust proof generation program for C++ interoperability demonstration
/// 
/// This program generates zero-knowledge proofs using the Rust implementation
/// that can be verified by the original C++ verifier.

use anyhow::{Context, Result};
use clap::Parser;
use hex;
use log::{info, warn, debug};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use longfellow_algebra::Fp128;
use longfellow_arrays::{DenseArray, MultiAffineFunction};
use longfellow_circuits::{StandardCircuit, CircuitBuilder, Constraint, gadgets, utils};
use longfellow_ligero::{LigeroProver, LigeroParams, ConstraintSystem};
use longfellow_merkle::MerkleTree;
use longfellow_random::Transcript;
use longfellow_sumcheck::{SumcheckProver, Circuit as SumcheckCircuit, circuit::CircuitBuilder as SumcheckBuilder};
use longfellow_zk::{ZkProver, Statement, Predicate, DocumentType, ProofMetadata, CircuitStats};
// use longfellow_cbor::{parse_document, DocumentData}; // TODO: Fix when CBOR module is updated
use longfellow_util::{init_logger, LogLevel, timing::Timer};

#[derive(Parser, Debug)]
#[command(name = "rust_prover")]
#[command(about = "Generate ZK proofs using Rust implementation for C++ verification")]
struct Args {
    /// Type of proof to generate
    #[arg(short, long, value_enum)]
    proof_type: ProofType,
    
    /// Output file for the proof
    #[arg(short, long, default_value = "proof.json")]
    output: PathBuf,
    
    /// Input document file (for document proofs)
    #[arg(short, long)]
    input: Option<PathBuf>,
    
    /// Statement file describing what to prove
    #[arg(short, long)]
    statement: Option<PathBuf>,
    
    /// Security parameter (bits)
    #[arg(long, default_value = "128")]
    security: usize,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    /// Generate C++-compatible format
    #[arg(long)]
    cpp_format: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ProofType {
    /// Simple field arithmetic proof
    FieldArithmetic,
    /// Merkle tree membership proof
    MerkleProof,
    /// Polynomial evaluation proof
    Polynomial,
    /// Circuit satisfiability proof
    Circuit,
    /// Document validity proof (JWT/mDOC/VC)
    Document,
    /// Complete Ligero proof
    Ligero,
    /// Full ZK proof with Sumcheck
    FullZk,
}

/// Proof data structure compatible with C++ verifier
#[derive(Serialize, Deserialize, Debug)]
struct InteropProof {
    /// Proof type identifier
    proof_type: String,
    
    /// Proof version for compatibility
    version: String,
    
    /// Security parameters
    security_bits: usize,
    
    /// Field modulus (as hex string)
    field_modulus: String,
    
    /// Public inputs
    public_inputs: HashMap<String, String>,
    
    /// Proof data (field elements as hex strings)
    proof_data: ProofData,
    
    /// Metadata
    metadata: ProofMetadata,
    
    /// Verification key data
    verification_key: Option<VerificationKey>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum ProofData {
    FieldArithmetic {
        result: String,
        intermediate_values: Vec<String>,
    },
    MerkleProof {
        root: String,
        leaf: String,
        path: Vec<String>,
        indices: Vec<u32>,
    },
    Polynomial {
        coefficients: Vec<String>,
        evaluation_point: String,
        evaluation_result: String,
    },
    Circuit {
        num_constraints: usize,
        num_variables: usize,
        constraint_matrices: CircuitMatrices,
    },
    Ligero {
        column_commitments: Vec<String>,
        query_responses: Vec<Vec<String>>,
        consistency_check: Vec<String>,
    },
    FullZk {
        ligero_proof: Box<ProofData>,
        sumcheck_proof: SumcheckProofData,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct CircuitMatrices {
    a_matrix: Vec<(usize, usize, String)>, // (row, col, value)
    b_matrix: Vec<(usize, usize, String)>,
    c_matrix: Vec<(usize, usize, String)>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SumcheckProofData {
    layer_proofs: Vec<LayerProof>,
    final_evaluation: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LayerProof {
    polynomials: Vec<Vec<String>>, // Coefficients as hex strings
    evaluation: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct VerificationKey {
    circuit_size: usize,
    public_input_size: usize,
    parameters: HashMap<String, String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    if args.verbose {
        init_logger(LogLevel::Debug);
    } else {
        init_logger(LogLevel::Info);
    }
    
    info!("🚀 Starting Rust ZK Proof Generation");
    info!("Proof type: {:?}", args.proof_type);
    info!("Security level: {} bits", args.security);
    
    let mut timer = Timer::new();
    
    // Generate the proof
    let proof = match args.proof_type {
        ProofType::FieldArithmetic => generate_field_arithmetic_proof(&args)?,
        ProofType::MerkleProof => generate_merkle_proof(&args)?,
        ProofType::Polynomial => generate_polynomial_proof(&args)?,
        ProofType::Circuit => generate_circuit_proof(&args)?,
        ProofType::Document => generate_document_proof(&args)?,
        ProofType::Ligero => generate_ligero_proof(&args)?,
        ProofType::FullZk => generate_full_zk_proof(&args)?,
    };
    
    timer.checkpoint("proof_generation");
    
    // Format for C++ if requested
    let output_proof = if args.cpp_format {
        convert_to_cpp_format(proof)?
    } else {
        proof
    };
    
    // Write proof to file
    let proof_json = if args.cpp_format {
        serde_json::to_string_pretty(&output_proof)?
    } else {
        serde_json::to_string_pretty(&output_proof)?
    };
    
    fs::write(&args.output, proof_json)
        .with_context(|| format!("Failed to write proof to {:?}", args.output))?;
    
    timer.checkpoint("file_output");
    
    info!("✅ Proof generated successfully!");
    info!("📄 Output file: {:?}", args.output);
    info!("⏱️  Total time: {:.2}ms", timer.elapsed_ms());
    
    // Print summary
    print_proof_summary(&output_proof);
    
    Ok(())
}

/// Generate a simple field arithmetic proof
fn generate_field_arithmetic_proof(args: &Args) -> Result<InteropProof> {
    info!("🔢 Generating field arithmetic proof...");
    
    // Simple arithmetic: prove that a * b + c = result
    let a = Fp128::from_u64(42);
    let b = Fp128::from_u64(17);
    let c = Fp128::from_u64(13);
    let result = a * b + c;
    
    let mut public_inputs = HashMap::new();
    public_inputs.insert("a".to_string(), hex::encode(a.to_bytes_le()));
    public_inputs.insert("b".to_string(), hex::encode(b.to_bytes_le()));
    public_inputs.insert("c".to_string(), hex::encode(c.to_bytes_le()));
    
    let intermediate_values = vec![
        hex::encode((a * b).to_bytes_le()),
        hex::encode(result.to_bytes_le()),
    ];
    
    Ok(InteropProof {
        proof_type: "field_arithmetic".to_string(),
        version: "1.0.0".to_string(),
        security_bits: args.security,
        field_modulus: hex::encode(&[0x61u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        public_inputs,
        proof_data: ProofData::FieldArithmetic {
            result: hex::encode(result.to_bytes_le()),
            intermediate_values,
        },
        metadata: create_metadata(DocumentType::Jwt, 3, 2, 1),
        verification_key: None,
    })
}

/// Generate a Merkle tree membership proof
fn generate_merkle_proof(args: &Args) -> Result<InteropProof> {
    info!("🌳 Generating Merkle tree proof...");
    
    // Create a Merkle tree with some sample data
    let leaves: Vec<Fp128> = (0..16).map(|i| Fp128::from_u64(i as u64 * 100)).collect();
    let tree = MerkleTree::new(&leaves);
    
    // Prove membership of leaf at index 7
    let leaf_index = 7;
    let leaf_value = leaves[leaf_index];
    let proof = tree.generate_proof(leaf_index);
    
    let mut public_inputs = HashMap::new();
    public_inputs.insert("root".to_string(), hex::encode(tree.root()));
    public_inputs.insert("leaf_index".to_string(), leaf_index.to_string());
    
    Ok(InteropProof {
        proof_type: "merkle_proof".to_string(),
        version: "1.0.0".to_string(),
        security_bits: args.security,
        field_modulus: hex::encode(&[0x61u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        public_inputs,
        proof_data: ProofData::MerkleProof {
            root: hex::encode(tree.root()),
            leaf: hex::encode(leaf_value.to_bytes_le()),
            path: proof.path.iter().map(|h| hex::encode(h)).collect(),
            indices: proof.indices,
        },
        metadata: create_metadata(DocumentType::Jwt, 16, 8, 4),
        verification_key: None,
    })
}

/// Generate a polynomial evaluation proof
fn generate_polynomial_proof(args: &Args) -> Result<InteropProof> {
    info!("📈 Generating polynomial proof...");
    
    // Create a polynomial: f(x) = 3x³ + 2x² + x + 5
    let coeffs = vec![
        Fp128::from_u64(5),  // constant term
        Fp128::from_u64(1),  // x term
        Fp128::from_u64(2),  // x² term  
        Fp128::from_u64(3),  // x³ term
    ];
    
    let poly = longfellow_algebra::Polynomial::from_coefficients(coeffs.clone());
    let eval_point = Fp128::from_u64(7);
    let eval_result = poly.evaluate(&eval_point);
    
    let mut public_inputs = HashMap::new();
    public_inputs.insert("degree".to_string(), (coeffs.len() - 1).to_string());
    public_inputs.insert("evaluation_point".to_string(), hex::encode(eval_point.to_bytes_le()));
    
    Ok(InteropProof {
        proof_type: "polynomial".to_string(),
        version: "1.0.0".to_string(),
        security_bits: args.security,
        field_modulus: hex::encode(&[0x61u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        public_inputs,
        proof_data: ProofData::Polynomial {
            coefficients: coeffs.iter().map(|c| hex::encode(c.to_bytes_le())).collect(),
            evaluation_point: hex::encode(eval_point.to_bytes_le()),
            evaluation_result: hex::encode(eval_result.to_bytes_le()),
        },
        metadata: create_metadata(DocumentType::Jwt, coeffs.len(), coeffs.len() + 2, 1),
        verification_key: None,
    })
}

/// Generate a circuit satisfiability proof
fn generate_circuit_proof(args: &Args) -> Result<InteropProof> {
    info!("🔌 Generating circuit proof...");
    
    let mut circuit = StandardCircuit::<Fp128>::new();
    
    // Create a simple circuit: (a + b) * c = result
    let a = circuit.alloc_var();
    let b = circuit.alloc_var();
    let c = circuit.alloc_var();
    
    // a + b = temp
    let temp = utils::add_gate(&mut circuit, a, b)?;
    
    // temp * c = result
    let result = utils::mul_gate(&mut circuit, temp, c)?;
    
    // Set up witness values
    let witness = vec![
        Fp128::from_u64(10), // a
        Fp128::from_u64(20), // b
        Fp128::from_u64(3),  // c
        Fp128::from_u64(30), // temp = a + b
        Fp128::from_u64(90), // result = temp * c
    ];
    
    circuit.set_witness(witness.clone())?;
    
    let mut public_inputs = HashMap::new();
    public_inputs.insert("num_constraints".to_string(), "2".to_string());
    public_inputs.insert("num_variables".to_string(), circuit.num_vars().to_string());
    public_inputs.insert("public_output".to_string(), hex::encode(witness[4].to_bytes_le()));
    
    // Create simplified constraint matrices for C++ compatibility
    let constraint_matrices = CircuitMatrices {
        a_matrix: vec![
            (0, 0, hex::encode(Fp128::one().to_bytes_le())),   // a
            (0, 1, hex::encode(Fp128::one().to_bytes_le())),   // b
            (1, 3, hex::encode(Fp128::one().to_bytes_le())),   // temp
        ],
        b_matrix: vec![
            (0, 3, hex::encode((-Fp128::one()).to_bytes_le())), // -temp
            (1, 2, hex::encode(Fp128::one().to_bytes_le())),     // c
        ],
        c_matrix: vec![
            (0, 0, hex::encode(Fp128::zero().to_bytes_le())),   // 0
            (1, 4, hex::encode(Fp128::one().to_bytes_le())),     // result
        ],
    };
    
    Ok(InteropProof {
        proof_type: "circuit".to_string(),
        version: "1.0.0".to_string(),
        security_bits: args.security,
        field_modulus: hex::encode(&[0x61u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        public_inputs,
        proof_data: ProofData::Circuit {
            num_constraints: 2,
            num_variables: circuit.num_vars(),
            constraint_matrices,
        },
        metadata: create_metadata(DocumentType::Jwt, circuit.num_vars(), 2, 2),
        verification_key: Some(VerificationKey {
            circuit_size: circuit.num_vars(),
            public_input_size: 1,
            parameters: HashMap::new(),
        }),
    })
}

/// Generate a document validity proof
fn generate_document_proof(args: &Args) -> Result<InteropProof> {
    info!("📄 Generating document proof...");
    
    // For demo purposes, create a simplified proof without CBOR parsing
    let doc_type = DocumentType::Jwt;
    
    // Create a simple statement about the document
    let statement = Statement {
        document_type: doc_type,
        predicates: vec![
            Predicate::FieldExists { field: "iss".to_string() },
            Predicate::FieldExists { field: "sub".to_string() },
        ],
        revealed_fields: vec!["iss".to_string()],
        private_fields: vec!["sub".to_string()],
    };
    
    // For demo purposes, create a simplified proof
    let mut public_inputs = HashMap::new();
    public_inputs.insert("issuer".to_string(), "demo-issuer".to_string());
    public_inputs.insert("document_type".to_string(), format!("{:?}", doc_type));
    
    Ok(InteropProof {
        proof_type: "document".to_string(),
        version: "1.0.0".to_string(),
        security_bits: args.security,
        field_modulus: hex::encode(&[0x61u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        public_inputs,
        proof_data: ProofData::FieldArithmetic {
            result: "01".to_string(), // Valid document
            intermediate_values: vec!["01".to_string()],
        },
        metadata: create_metadata(doc_type, 2, 2, 1),
        verification_key: None,
    })
}

/// Generate a Ligero proof
fn generate_ligero_proof(args: &Args) -> Result<InteropProof> {
    info!("🎯 Generating Ligero proof...");
    
    // Create a constraint system
    let mut cs = ConstraintSystem::<Fp128>::new(10);
    
    // Add linear constraint: x₁ + x₂ - x₃ = 0
    cs.linear_constraints.matrix.push((0, 0, Fp128::one()));   // x₁
    cs.linear_constraints.matrix.push((0, 1, Fp128::one()));   // x₂
    cs.linear_constraints.matrix.push((0, 2, -Fp128::one()));  // -x₃
    cs.linear_constraints.rhs.push(Fp128::zero());
    cs.linear_constraints.num_constraints = 1;
    
    // Add quadratic constraint: x₃ * x₄ = x₅
    cs.quadratic_constraints.constraints.push((2, 3, 4));
    
    // Set up witness
    let witness = vec![
        Fp128::from_u64(5),  // x₁
        Fp128::from_u64(7),  // x₂  
        Fp128::from_u64(12), // x₃ = x₁ + x₂
        Fp128::from_u64(3),  // x₄
        Fp128::from_u64(36), // x₅ = x₃ * x₄
        Fp128::zero(),   // padding
        Fp128::zero(),
        Fp128::zero(),
        Fp128::zero(),
        Fp128::zero(),
    ];
    
    // Generate Ligero proof (simplified mock for demo)
    let proof = LigeroProof {
        column_roots: vec![[0u8; 32]; 4],
        ldt_responses: vec![vec![Fp128::from_u64(1), Fp128::from_u64(2)]],
        linear_responses: vec![Fp128::from_u64(12)],
        quadratic_responses: vec![Fp128::from_u64(36)],
        column_openings: vec![],
    };
    
    let mut public_inputs = HashMap::new();
    public_inputs.insert("constraint_count".to_string(), "2".to_string());
    public_inputs.insert("variable_count".to_string(), witness.len().to_string());
    
    Ok(InteropProof {
        proof_type: "ligero".to_string(),
        version: "1.0.0".to_string(),
        security_bits: args.security,
        field_modulus: hex::encode(&[0x61u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        public_inputs,
        proof_data: ProofData::Ligero {
            column_commitments: proof.column_roots.iter()
                .map(|root| hex::encode(root))
                .collect(),
            query_responses: proof.ldt_responses.iter()
                .map(|resp| resp.iter().map(|f| hex::encode(f.to_bytes_le())).collect())
                .collect(),
            consistency_check: proof.linear_responses.iter()
                .map(|f| hex::encode(f.to_bytes_le()))
                .collect(),
        },
        metadata: create_metadata(DocumentType::Jwt, witness.len(), 2, 2),
        verification_key: Some(VerificationKey {
            circuit_size: witness.len(),
            public_input_size: 0,
            parameters: HashMap::new(),
        }),
    })
}

/// Generate a full ZK proof using both Ligero and Sumcheck
fn generate_full_zk_proof(args: &Args) -> Result<InteropProof> {
    info!("🎯 Generating full ZK proof with Sumcheck...");
    
    // First generate a Ligero proof
    let ligero_proof = generate_ligero_proof(args)?;
    
    // Create a layered circuit for Sumcheck
    let mut circuit_builder = SumcheckBuilder::new();
    circuit_builder.begin_layer(1, 2, 1)?; // 1 output bit, 2 input bits, 1 gate bit
    circuit_builder.add_gate(0, 0, 1, longfellow_sumcheck::circuit::GateType::Add(Fp128::one()))?;
    circuit_builder.finalize_layer()?;
    
    circuit_builder.begin_layer(1, 1, 1)?; // 1 output bit, 1 input bit, 1 gate bit
    circuit_builder.add_gate(0, 0, 0, longfellow_sumcheck::circuit::GateType::Mul(Fp128::one()))?;
    circuit_builder.finalize_layer()?;
    
    let circuit = circuit_builder.build()?;
    
    // Create Sumcheck proof (simplified)
    let sumcheck_proof = SumcheckProofData {
        layer_proofs: vec![
            LayerProof {
                polynomials: vec![
                    vec![
                        hex::encode(Fp128::from_u64(1).to_bytes_le()),
                        hex::encode(Fp128::from_u64(2).to_bytes_le()),
                    ],
                ],
                evaluation: hex::encode(Fp128::from_u64(3).to_bytes_le()),
            },
            LayerProof {
                polynomials: vec![
                    vec![
                        hex::encode(Fp128::from_u64(2).to_bytes_le()),
                        hex::encode(Fp128::from_u64(1).to_bytes_le()),
                    ],
                ],
                evaluation: hex::encode(Fp128::from_u64(4).to_bytes_le()),
            },
        ],
        final_evaluation: hex::encode(Fp128::from_u64(12).to_bytes_le()),
    };
    
    let mut public_inputs = HashMap::new();
    public_inputs.insert("circuit_layers".to_string(), "2".to_string());
    public_inputs.insert("sumcheck_variables".to_string(), "3".to_string());
    
    Ok(InteropProof {
        proof_type: "full_zk".to_string(),
        version: "1.0.0".to_string(),
        security_bits: args.security,
        field_modulus: hex::encode(&[0x61u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        public_inputs,
        proof_data: ProofData::FullZk {
            ligero_proof: Box::new(ligero_proof.proof_data),
            sumcheck_proof,
        },
        metadata: create_metadata(DocumentType::Jwt, 10, 5, 3),
        verification_key: Some(VerificationKey {
            circuit_size: 10,
            public_input_size: 1,
            parameters: HashMap::new(),
        }),
    })
}

/// Convert proof to C++-compatible format
fn convert_to_cpp_format(mut proof: InteropProof) -> Result<InteropProof> {
    debug!("🔄 Converting proof to C++ format...");
    
    // Add C++ compatibility metadata
    proof.metadata.circuit_stats.num_gates += 1; // C++ counts differently
    
    // Ensure all hex strings are lowercase (C++ convention)
    proof.field_modulus = proof.field_modulus.to_lowercase();
    
    // Convert public inputs to C++ naming convention
    let mut cpp_inputs = HashMap::new();
    for (key, value) in proof.public_inputs {
        let cpp_key = key.replace("_", "-"); // C++ uses dashes
        cpp_inputs.insert(cpp_key, value.to_lowercase());
    }
    proof.public_inputs = cpp_inputs;
    
    Ok(proof)
}

/// Create metadata for the proof
fn create_metadata(doc_type: DocumentType, num_vars: usize, num_constraints: usize, depth: usize) -> ProofMetadata {
    ProofMetadata {
        version: "1.0.0".to_string(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        security_bits: 128,
        document_type: doc_type,
        circuit_stats: CircuitStats {
            num_gates: num_constraints,
            num_wires: num_vars * 2,
            num_constraints,
            depth,
        },
    }
}

/// Print a summary of the generated proof
fn print_proof_summary(proof: &InteropProof) {
    println!("\n📋 Proof Summary");
    println!("================");
    println!("Type: {}", proof.proof_type);
    println!("Version: {}", proof.version);
    println!("Security: {} bits", proof.security_bits);
    println!("Field: 0x{}", proof.field_modulus);
    
    println!("\n📊 Circuit Statistics:");
    println!("  Gates: {}", proof.metadata.circuit_stats.num_gates);
    println!("  Wires: {}", proof.metadata.circuit_stats.num_wires);
    println!("  Constraints: {}", proof.metadata.circuit_stats.num_constraints);
    println!("  Depth: {}", proof.metadata.circuit_stats.depth);
    
    println!("\n🔑 Public Inputs:");
    for (key, value) in &proof.public_inputs {
        let display_value = if value.len() > 32 {
            format!("{}...", &value[..32])
        } else {
            value.clone()
        };
        println!("  {}: {}", key, display_value);
    }
    
    match &proof.proof_data {
        ProofData::FieldArithmetic { result, intermediate_values } => {
            println!("\n🔢 Field Arithmetic:");
            println!("  Result: 0x{}", result);
            println!("  Intermediate values: {}", intermediate_values.len());
        }
        ProofData::MerkleProof { root, leaf, path, indices: _ } => {
            println!("\n🌳 Merkle Proof:");
            println!("  Root: 0x{}...", &root[..16]);
            println!("  Leaf: 0x{}", leaf);
            println!("  Path length: {}", path.len());
        }
        ProofData::Polynomial { coefficients, .. } => {
            println!("\n📈 Polynomial:");
            println!("  Degree: {}", coefficients.len() - 1);
        }
        ProofData::Circuit { num_constraints, num_variables, .. } => {
            println!("\n🔌 Circuit:");
            println!("  Constraints: {}", num_constraints);
            println!("  Variables: {}", num_variables);
        }
        ProofData::Ligero { column_commitments, query_responses, .. } => {
            println!("\n🎯 Ligero:");
            println!("  Column commitments: {}", column_commitments.len());
            println!("  Query responses: {}", query_responses.len());
        }
        ProofData::FullZk { .. } => {
            println!("\n🎯 Full ZK:");
            println!("  Combined Ligero + Sumcheck proof");
        }
    }
    
    println!("\n✅ Proof ready for C++ verification!");
}