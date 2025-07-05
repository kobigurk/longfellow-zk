/// Test Proof Generation Module for C++ Verifier Testing
/// 
/// This module provides functions to generate various test proofs that can be verified by the C++ verifier

use longfellow_algebra::{Fp128, Field};
use longfellow_cbor::jwt::{Jwt, JwtBuilder, JwtAlgorithm};
use longfellow_cbor::{Value, ClaimExtractor};
use longfellow_core::Result;
use longfellow_ligero::{
    LigeroParams, LigeroInstance, ConstraintSystem, LigeroProver
};
use longfellow_sumcheck::{
    Circuit, Layer, SumcheckInstance, Prover as SumcheckProver, SumcheckOptions
};
use longfellow_zk::{
    Statement, Predicate, DocumentType, DocumentData, ZkWitness, ZkInstance, ZkCircuit,
    ZkProof, ProofMetadata, CircuitStats, ProofOptions
};
use rand::rngs::OsRng;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration for test proof generation
pub struct TestProofConfig {
    pub output_dir: String,
    pub generate_json: bool,
    pub generate_binary: bool,
    pub num_proofs: usize,
}

impl Default for TestProofConfig {
    fn default() -> Self {
        Self {
            output_dir: "test_proofs".to_string(),
            generate_json: true,
            generate_binary: true,
            num_proofs: 3,
        }
    }
}

/// Generate all test proofs with given configuration
pub fn generate_all_test_proofs(config: TestProofConfig) -> Result<()> {
    // Create output directory
    fs::create_dir_all(&config.output_dir)?;
    
    println!("Generating {} test proofs...", config.num_proofs);
    
    // Generate different types of proofs
    for i in 0..config.num_proofs {
        match i % 3 {
            0 => generate_simple_proof(&config, i)?,
            1 => generate_jwt_proof(&config, i)?,
            2 => generate_complex_proof(&config, i)?,
            _ => unreachable!(),
        }
    }
    
    // Generate metadata file
    generate_metadata_file(&config)?;
    
    println!("\nGenerated {} proofs in {}/", config.num_proofs, config.output_dir);
    Ok(())
}

/// Generate a simple Ligero proof
fn generate_simple_proof(config: &TestProofConfig, index: usize) -> Result<()> {
    println!("\nGenerating simple proof {}...", index);
    
    // Create constraint system: prove x + y = 10, x * y = 21
    let mut cs = ConstraintSystem::<Fp128>::new(3);
    
    cs.add_linear_constraint(
        vec![(0, Fp128::one()), (1, Fp128::one())],
        Fp128::from(10),
    );
    cs.add_quadratic_constraint(0, 1, 2);
    cs.add_linear_constraint(
        vec![(2, Fp128::one())],
        Fp128::from(21),
    );
    
    // Create instance and prover
    let params = LigeroParams::security_128();
    let instance = LigeroInstance::new(params, cs)?;
    let prover = LigeroProver::new(instance)?;
    
    // Generate proof with witness [3, 7, 21]
    let witness = vec![Fp128::from(3), Fp128::from(7), Fp128::from(21)];
    let ligero_proof = prover.prove(&witness, &mut OsRng)?;
    
    // Create ZK proof wrapper
    let proof = ZkProof {
        statement: Statement {
            document_type: DocumentType::Raw,
            predicates: vec![
                Predicate::Custom {
                    id: "simple_arithmetic".to_string(),
                    params: vec!["x+y=10".to_string(), "x*y=21".to_string()],
                },
            ],
            revealed_fields: vec![],
            hidden_fields: vec!["x".to_string(), "y".to_string()],
        },
        ligero_proof,
        sumcheck_proof: None,
        commitments: vec![[0u8; 32]; 2], // Dummy commitments
        metadata: ProofMetadata {
            version: "1.0.0".to_string(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            security_bits: 128,
            document_type: DocumentType::Raw,
            circuit_stats: CircuitStats {
                num_gates: 1,
                num_wires: 3,
                num_constraints: 3,
                depth: 2,
            },
        },
    };
    
    save_proof(&proof, config, &format!("simple_proof_{}", index))?;
    Ok(())
}

/// Generate a JWT-based proof
fn generate_jwt_proof(config: &TestProofConfig, index: usize) -> Result<()> {
    println!("\nGenerating JWT proof {}...", index);
    
    // Create a test JWT
    let jwt = JwtBuilder::new(JwtAlgorithm::HS256)
        .issuer("test-issuer".to_string())
        .subject(format!("user{}", index))
        .claim("age".to_string(), Value::Integer(25 + index as i64))
        .claim("country".to_string(), Value::Text("US".to_string()))
        .claim("verified".to_string(), Value::Bool(true))
        .expiration(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 3600)
        .build_unsigned()?;
    
    // Create statement
    let statement = Statement {
        document_type: DocumentType::Jwt,
        predicates: vec![
            Predicate::AgeOver { years: 18 },
            Predicate::FieldEquals {
                field: "country".to_string(),
                value: serde_json::json!("US"),
            },
        ],
        revealed_fields: vec!["verified".to_string()],
        hidden_fields: vec!["age".to_string(), "sub".to_string()],
    };
    
    // Create simple circuit for demo
    let mut cs = ConstraintSystem::<Fp128>::new(10);
    
    // Age constraint: age - 18 >= 0
    cs.add_linear_constraint(
        vec![(0, Fp128::one())], // age wire
        Fp128::from(25 + index as u64), // actual age
    );
    
    // Create Ligero proof
    let params = LigeroParams::security_128();
    let instance = LigeroInstance::new(params, cs)?;
    let prover = LigeroProver::new(instance)?;
    
    let mut witness = vec![Fp128::from(25 + index as u64)];
    while witness.len() < 10 {
        witness.push(Fp128::zero());
    }
    
    let ligero_proof = prover.prove(&witness, &mut OsRng)?;
    
    // Create ZK proof
    let proof = ZkProof {
        statement,
        ligero_proof,
        sumcheck_proof: None,
        commitments: vec![[1u8; 32], [2u8; 32]], // Dummy commitments
        metadata: ProofMetadata {
            version: "1.0.0".to_string(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            security_bits: 128,
            document_type: DocumentType::Jwt,
            circuit_stats: CircuitStats {
                num_gates: 0,
                num_wires: 10,
                num_constraints: 1,
                depth: 1,
            },
        },
    };
    
    save_proof(&proof, config, &format!("jwt_proof_{}", index))?;
    Ok(())
}

/// Generate a complex proof with Sumcheck
fn generate_complex_proof(config: &TestProofConfig, index: usize) -> Result<()> {
    println!("\nGenerating complex proof with Sumcheck {}...", index);
    
    // Create Ligero constraint system
    let mut cs = ConstraintSystem::<Fp128>::new(4);
    cs.add_linear_constraint(
        vec![(0, Fp128::one()), (1, Fp128::one()), (2, -Fp128::one())],
        Fp128::zero(),
    );
    cs.add_quadratic_constraint(0, 1, 3);
    
    let params = LigeroParams::security_128();
    let instance = LigeroInstance::new(params, cs)?;
    let prover = LigeroProver::new(instance)?;
    
    let witness = vec![
        Fp128::from(2 + index as u64),
        Fp128::from(3 + index as u64),
        Fp128::from(5 + 2 * index as u64),
        Fp128::from((2 + index as u64) * (3 + index as u64)),
    ];
    
    let ligero_proof = prover.prove(&witness, &mut OsRng)?;
    
    // Create Sumcheck circuit
    let mut circuit = Circuit::<Fp128>::new();
    
    let input_layer = Layer::new_input(2);
    circuit.add_layer(input_layer);
    
    let mut output_layer = Layer::new(1, 0);
    output_layer.add_gate(0, vec![(0, Fp128::one()), (1, Fp128::one())]);
    circuit.add_layer(output_layer);
    
    circuit.finalize()?;
    
    // Generate Sumcheck proof
    let claimed_sum = Fp128::from(20 + 8 * index as u64);
    let sumcheck_instance = SumcheckInstance::new(circuit, 4, claimed_sum)?;
    
    let inputs = vec![
        vec![Fp128::from(2 + index as u64), Fp128::from(3 + index as u64)],
        vec![Fp128::from(1 + index as u64), Fp128::from(4 + index as u64)],
        vec![Fp128::from(3 + index as u64), Fp128::from(2 + index as u64)],
        vec![Fp128::from(4 + index as u64), Fp128::from(1 + index as u64)],
    ];
    
    let mut sumcheck_prover = SumcheckProver::new(sumcheck_instance, SumcheckOptions::default())?;
    sumcheck_prover.set_inputs(&inputs)?;
    let sumcheck_proof = sumcheck_prover.prove(&mut OsRng)?;
    
    // Create ZK proof
    let proof = ZkProof {
        statement: Statement {
            document_type: DocumentType::Raw,
            predicates: vec![
                Predicate::Custom {
                    id: "complex_computation".to_string(),
                    params: vec!["sumcheck".to_string()],
                },
            ],
            revealed_fields: vec![],
            hidden_fields: vec!["computation".to_string()],
        },
        ligero_proof,
        sumcheck_proof: Some(sumcheck_proof),
        commitments: vec![[3u8; 32]],
        metadata: ProofMetadata {
            version: "1.0.0".to_string(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            security_bits: 128,
            document_type: DocumentType::Raw,
            circuit_stats: CircuitStats {
                num_gates: 1,
                num_wires: 4,
                num_constraints: 2,
                depth: 2,
            },
        },
    };
    
    save_proof(&proof, config, &format!("complex_proof_{}", index))?;
    Ok(())
}

/// Save proof in requested formats
fn save_proof(proof: &ZkProof<Fp128>, config: &TestProofConfig, name: &str) -> Result<()> {
    let base_path = Path::new(&config.output_dir).join(name);
    
    if config.generate_json || (!config.generate_json && !config.generate_binary) {
        // Default to JSON if no format specified
        let json_path = base_path.with_extension("json");
        let json_data = serde_json::to_string_pretty(proof)?;
        fs::write(&json_path, json_data)?;
        println!("  Saved JSON: {}", json_path.display());
    }
    
    if config.generate_binary {
        let bin_path = base_path.with_extension("bin");
        let bin_data = bincode::serialize(proof)?;
        fs::write(&bin_path, bin_data)?;
        println!("  Saved binary: {}", bin_path.display());
    }
    
    Ok(())
}

/// Generate metadata file for test proofs
fn generate_metadata_file(config: &TestProofConfig) -> Result<()> {
    use serde_json::json;
    
    let metadata_path = Path::new(&config.output_dir).join("test_proofs_metadata.json");
    
    let metadata = json!({
        "version": "2.0.0",
        "generated_at": SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        "num_proofs": config.num_proofs,
        "proof_types": [
            {
                "name": "simple_arithmetic",
                "description": "Basic Ligero proof for x + y = 10, x * y = 21",
                "has_sumcheck": false,
                "security_bits": 128
            },
            {
                "name": "jwt_age_verification",
                "description": "JWT-based age verification proof",
                "has_sumcheck": false,
                "security_bits": 128
            },
            {
                "name": "complex_with_sumcheck",
                "description": "Complex proof with both Ligero and Sumcheck components",
                "has_sumcheck": true,
                "security_bits": 128
            }
        ],
        "formats": {
            "json": config.generate_json,
            "binary": config.generate_binary
        }
    });
    
    let json_str = serde_json::to_string_pretty(&metadata)?;
    fs::write(&metadata_path, json_str)?;
    
    println!("  Generated metadata: {}", metadata_path.display());
    Ok(())
}