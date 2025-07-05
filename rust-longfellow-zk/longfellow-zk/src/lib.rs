/// Main zero-knowledge proof system orchestrating Ligero and Sumcheck protocols
/// 
/// This module provides the high-level API for proving statements about
/// cryptographic documents (JWT, mDOC, W3C VCs) using zero-knowledge proofs.

use longfellow_core::{LongfellowError, Result};
use longfellow_algebra::traits::Field;
use longfellow_cbor::{jwt::Jwt, mdoc::Document, vc::VerifiableCredential, ClaimExtractor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod statement;
pub mod prover;
pub mod prover_impl;
pub mod prover_full;
pub mod verifier;
pub mod document;
pub mod serialization;

pub use statement::{Statement, Predicate, DocumentType};
pub use prover::ZkProver;
pub use verifier::ZkVerifier;

/// Zero-knowledge proof combining Ligero and Sumcheck
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZkProof<F: Field> {
    /// Statement being proven
    pub statement: Statement,
    
    /// Ligero proof for circuit satisfiability
    pub ligero_proof: longfellow_ligero::LigeroProof<F>,
    
    /// Sumcheck proof for circuit evaluation
    pub sumcheck_proof: Option<longfellow_sumcheck::SumcheckProof<F>>,
    
    /// Commitment to hidden values
    pub commitments: Vec<[u8; 32]>,
    
    /// Proof metadata
    pub metadata: ProofMetadata,
}

/// Proof metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofMetadata {
    /// Proof system version
    pub version: String,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Security level in bits
    pub security_bits: usize,
    
    /// Document type
    pub document_type: DocumentType,
    
    /// Circuit statistics
    pub circuit_stats: CircuitStats,
    
    /// Proof generation time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_generation_time_ms: Option<u64>,
    
    /// Reed-Solomon encoding rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reed_solomon_rate: Option<f64>,
    
    /// Encoding type used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_type: Option<String>,
}

/// Circuit statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitStats {
    /// Number of gates
    pub num_gates: usize,
    
    /// Number of wires
    pub num_wires: usize,
    
    /// Number of constraints
    pub num_constraints: usize,
    
    /// Circuit depth
    pub depth: usize,
}

/// Zero-knowledge proof instance
pub struct ZkInstance<F: Field> {
    /// The statement to prove
    pub statement: Statement,
    
    /// The witness (private)
    pub witness: ZkWitness,
    
    /// Circuit representation
    pub circuit: ZkCircuit<F>,
}

/// Witness for zero-knowledge proof
#[derive(Clone, Debug)]
pub struct ZkWitness {
    /// The full document
    pub document: DocumentData,
    
    /// Private field values
    pub private_values: HashMap<String, Vec<u8>>,
    
    /// Randomness for commitments
    pub randomness: Vec<[u8; 32]>,
}

/// Document data variants
#[derive(Clone, Debug)]
pub enum DocumentData {
    Jwt(Jwt),
    Mdoc(Document),
    VerifiableCredential(VerifiableCredential),
    Raw(Vec<u8>),
}

/// Circuit representation for ZK proof
pub struct ZkCircuit<F: Field> {
    /// Ligero constraint system
    pub ligero_cs: longfellow_ligero::ConstraintSystem<F>,
    
    /// Sumcheck circuit (optional)
    pub sumcheck_circuit: Option<longfellow_sumcheck::Circuit<F>>,
    
    /// Wire assignments
    pub wire_values: Vec<F>,
    
    /// Public inputs
    pub public_inputs: Vec<F>,
}

impl<F: Field> ZkCircuit<F> {
    /// Create a new circuit
    pub fn new(num_witnesses: usize) -> Self {
        Self {
            ligero_cs: longfellow_ligero::ConstraintSystem::new(num_witnesses),
            sumcheck_circuit: None,
            wire_values: Vec::new(),
            public_inputs: Vec::new(),
        }
    }
    
    /// Add a linear constraint
    pub fn add_linear_constraint(&mut self, coeffs: Vec<(usize, F)>, constant: F) -> Result<()> {
        self.ligero_cs.add_linear_constraint(coeffs, constant);
        Ok(())
    }
    
    /// Add a quadratic constraint
    pub fn add_quadratic_constraint(&mut self, x: usize, y: usize, z: usize) -> Result<()> {
        self.ligero_cs.add_quadratic_constraint(x, y, z);
        Ok(())
    }
    
    /// Set wire values
    pub fn set_wire_values(&mut self, values: Vec<F>) {
        self.wire_values = values;
    }
    
    /// Set public inputs
    pub fn set_public_inputs(&mut self, inputs: Vec<F>) {
        self.public_inputs = inputs;
    }
    
    /// Check if the circuit is satisfied
    pub fn is_satisfied(&self) -> Result<bool> {
        self.ligero_cs.is_satisfied(&self.wire_values)
    }
}

/// Options for proof generation
#[derive(Clone, Debug)]
pub struct ProofOptions {
    /// Security level in bits
    pub security_bits: usize,
    
    /// Use Sumcheck for circuit evaluation
    pub use_sumcheck: bool,
    
    /// Enable parallel computation
    pub parallel: bool,
    
    /// Optimize for proof size
    pub optimize_size: bool,
    
    /// Reed-Solomon encoding rate (optional)
    pub reed_solomon_rate: Option<f64>,
}

impl Default for ProofOptions {
    fn default() -> Self {
        Self {
            security_bits: 128,
            use_sumcheck: true,
            parallel: true,
            optimize_size: false,
            reed_solomon_rate: None,
        }
    }
}

/// Create a proof instance from a statement and document
pub fn create_instance<F: Field>(
    statement: Statement,
    document: DocumentData,
    private_fields: Vec<String>,
) -> Result<ZkInstance<F>> {
    // Extract all claims from document
    let all_claims = match &document {
        DocumentData::Jwt(jwt) => jwt.extract_claims()?,
        DocumentData::Mdoc(mdoc) => mdoc.extract_claims()?,
        DocumentData::VerifiableCredential(vc) => vc.extract_claims()?,
        DocumentData::Raw(_) => HashMap::new(),
    };
    
    // Separate private values
    let mut private_values = HashMap::new();
    for field in &private_fields {
        if let Some(value) = all_claims.get(field) {
            private_values.insert(
                field.clone(),
                serde_json::to_vec(value)
                    .map_err(|e| LongfellowError::SerializationError(e.to_string()))?
            );
        }
    }
    
    // Create witness
    let witness = ZkWitness {
        document,
        private_values,
        randomness: vec![[0u8; 32]; private_fields.len()], // Would be random in practice
    };
    
    // Build circuit based on statement
    let circuit = build_circuit_for_statement(&statement, &witness)?;
    
    Ok(ZkInstance {
        statement,
        witness,
        circuit,
    })
}

/// Build a circuit for a given statement
fn build_circuit_for_statement<F: Field>(
    statement: &Statement,
    _witness: &ZkWitness,
) -> Result<ZkCircuit<F>> {
    let circuit = ZkCircuit::new(1000); // Placeholder size
    
    // Add constraints based on predicates
    for predicate in &statement.predicates {
        match predicate {
            Predicate::FieldEquals { field: _, value: _ } => {
                // Add constraints for field equality
                // This would involve parsing, hashing, and comparison circuits
            }
            Predicate::FieldExists { field: _ } => {
                // Add constraints for field existence
            }
            Predicate::FieldGreaterThan { field: _, value: _ } => {
                // Add comparison constraints
            }
            Predicate::AgeOver { years: _ } => {
                // Add date arithmetic constraints
            }
            Predicate::ValidSignature => {
                // Add signature verification constraints
            }
            Predicate::ValidIssuer { issuer: _ } => {
                // Add issuer validation constraints
            }
            Predicate::NotExpired => {
                // Add expiration check constraints
            }
            Predicate::Custom { id: _, params: _ } => {
                // Add custom constraints
            }
        }
    }
    
    Ok(circuit)
}

/// Error types specific to ZK operations
#[derive(Debug, thiserror::Error)]
pub enum ZkError {
    #[error("Invalid statement: {0}")]
    InvalidStatement(String),
    
    #[error("Circuit construction failed: {0}")]
    CircuitError(String),
    
    #[error("Proof generation failed: {0}")]
    ProofError(String),
    
    #[error("Verification failed: {0}")]
    VerificationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_circuit_creation() {
        let mut circuit = ZkCircuit::<Fp128>::new(10);
        
        // Add some constraints
        circuit.add_linear_constraint(
            vec![(0, Fp128::one()), (1, Fp128::from(2))],
            Fp128::from(5),
        ).unwrap();
        
        circuit.add_quadratic_constraint(2, 3, 4).unwrap();
        
        // Set witness
        circuit.set_wire_values(vec![
            Fp128::from(1),
            Fp128::from(2),
            Fp128::from(3),
            Fp128::from(4),
            Fp128::from(12), // 3 * 4
            Fp128::zero(),
            Fp128::zero(),
            Fp128::zero(),
            Fp128::zero(),
            Fp128::zero(),
        ]);
        
        assert!(circuit.is_satisfied().unwrap());
    }
}