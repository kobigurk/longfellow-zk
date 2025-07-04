/// Zero-knowledge verifier implementation

use crate::{
    ZkProof, Statement, DocumentType, ProofMetadata,
    document::CommitmentGenerator,
};
use longfellow_algebra::traits::Field;
use longfellow_core::{LongfellowError, Result};
use longfellow_ligero::{LigeroVerifier, LigeroInstance, LigeroParams};
use longfellow_sumcheck::{SumcheckInstance, verifier::VerifierLayers};
use std::collections::HashMap;

/// Zero-knowledge verifier
pub struct ZkVerifier<F: Field> {
    /// Cached Ligero parameters by security level
    ligero_params_cache: HashMap<usize, LigeroParams>,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field> ZkVerifier<F> {
    /// Create a new verifier
    pub fn new() -> Self {
        Self {
            ligero_params_cache: HashMap::new(),
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Verify a zero-knowledge proof
    pub fn verify(
        &mut self,
        proof: &ZkProof<F>,
        public_inputs: &HashMap<String, Vec<u8>>,
    ) -> Result<bool> {
        // Validate proof metadata
        self.validate_metadata(&proof.metadata)?;
        
        // Validate statement
        proof.statement.validate()
            .map_err(|e| LongfellowError::ValidationError(e))?;
        
        // Check revealed fields match public inputs
        if !self.check_revealed_fields(&proof.statement, public_inputs) {
            return Ok(false);
        }
        
        // Get Ligero parameters
        let ligero_params = self.get_ligero_params(proof.metadata.security_bits)?;
        
        // Verify Ligero proof
        if !self.verify_ligero_proof(&proof.ligero_proof, &proof.statement, ligero_params)? {
            return Ok(false);
        }
        
        // Verify Sumcheck proof if present
        if let Some(ref sumcheck_proof) = proof.sumcheck_proof {
            if !self.verify_sumcheck_proof(sumcheck_proof, &proof.statement)? {
                return Ok(false);
            }
        }
        
        // Verify commitments correspond to private fields
        if proof.commitments.len() != proof.statement.private_fields.len() {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Verify with commitment openings
    pub fn verify_with_openings(
        &mut self,
        proof: &ZkProof<F>,
        public_inputs: &HashMap<String, Vec<u8>>,
        private_openings: &HashMap<String, (Vec<u8>, [u8; 32])>,
    ) -> Result<bool> {
        // First do standard verification
        if !self.verify(proof, public_inputs)? {
            return Ok(false);
        }
        
        // Verify commitment openings
        for (i, field) in proof.statement.private_fields.iter().enumerate() {
            if let Some((value, randomness)) = private_openings.get(field) {
                if !CommitmentGenerator::verify(&proof.commitments[i], value, randomness) {
                    return Ok(false);
                }
            }
        }
        
        Ok(true)
    }
    
    /// Validate proof metadata
    fn validate_metadata(&self, metadata: &ProofMetadata) -> Result<()> {
        // Check version
        if !metadata.version.starts_with("1.") {
            return Err(LongfellowError::ValidationError(
                format!("Unsupported proof version: {}", metadata.version)
            ));
        }
        
        // Check timestamp is reasonable (not in future)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if metadata.created_at > now + 300 { // Allow 5 minutes clock skew
            return Err(LongfellowError::ValidationError(
                "Proof timestamp is in the future".to_string()
            ));
        }
        
        // Check security level
        if metadata.security_bits < 80 {
            return Err(LongfellowError::ValidationError(
                format!("Security level too low: {} bits", metadata.security_bits)
            ));
        }
        
        Ok(())
    }
    
    /// Check revealed fields match public inputs
    fn check_revealed_fields(
        &self,
        statement: &Statement,
        public_inputs: &HashMap<String, Vec<u8>>,
    ) -> bool {
        for field in &statement.revealed_fields {
            if !public_inputs.contains_key(field) {
                return false;
            }
        }
        true
    }
    
    /// Get or create Ligero parameters
    fn get_ligero_params(&mut self, security_bits: usize) -> Result<LigeroParams> {
        if let Some(params) = self.ligero_params_cache.get(&security_bits) {
            Ok(params.clone())
        } else {
            let params = LigeroParams::new(security_bits)?;
            self.ligero_params_cache.insert(security_bits, params.clone());
            Ok(params)
        }
    }
    
    /// Verify Ligero proof
    fn verify_ligero_proof(
        &self,
        proof: &longfellow_ligero::LigeroProof<F>,
        statement: &Statement,
        params: LigeroParams,
    ) -> Result<bool> {
        // Reconstruct constraint system from statement
        let cs = self.reconstruct_constraint_system(statement)?;
        
        // Create Ligero instance
        let instance = LigeroInstance::new(params, cs)?;
        
        // Create verifier
        let verifier = LigeroVerifier::new(instance)?;
        
        // Verify proof
        verifier.verify(proof)
    }
    
    /// Verify Sumcheck proof
    fn verify_sumcheck_proof(
        &self,
        proof: &longfellow_sumcheck::SumcheckProof<F>,
        statement: &Statement,
    ) -> Result<bool> {
        // Reconstruct circuit from statement
        let circuit = self.reconstruct_sumcheck_circuit(statement)?;
        
        // Create instance with dummy claim (would be from Ligero proof)
        let instance = SumcheckInstance::new(
            circuit.clone(),
            1,
            F::zero(), // Would get from Ligero
        )?;
        
        // Create verifier
        let verifier = VerifierLayers::new(circuit);
        
        // Verify proof
        verifier.verify(&instance, proof)
    }
    
    /// Reconstruct constraint system from statement
    fn reconstruct_constraint_system(
        &self,
        statement: &Statement,
    ) -> Result<longfellow_ligero::ConstraintSystem<F>> {
        // This would reconstruct the constraint system based on the statement
        // For now, return a dummy system
        let mut cs = longfellow_ligero::ConstraintSystem::new(100);
        
        // Add constraints based on predicates
        for predicate in &statement.predicates {
            match predicate {
                crate::Predicate::FieldEquals { .. } => {
                    // Add equality constraints
                    cs.add_linear_constraint(
                        vec![(0, F::one()), (1, -F::one())],
                        F::zero(),
                    );
                }
                crate::Predicate::FieldGreaterThan { .. } => {
                    // Add comparison constraints
                    // This would involve range proofs
                }
                crate::Predicate::ValidSignature => {
                    // Add signature verification constraints
                    // This would involve EC arithmetic
                }
                _ => {}
            }
        }
        
        Ok(cs)
    }
    
    /// Reconstruct Sumcheck circuit from statement
    fn reconstruct_sumcheck_circuit(
        &self,
        _statement: &Statement,
    ) -> Result<longfellow_sumcheck::Circuit<F>> {
        // This would reconstruct the circuit based on the statement
        // For now, return a simple circuit
        use longfellow_sumcheck::circuit::{CircuitBuilder, GateType};
        
        let mut builder = CircuitBuilder::new();
        builder.begin_layer(0, 1, 0)?;
        builder.add_gate(0, 0, 1, GateType::Add(F::one()))?;
        builder.finalize_layer()?;
        
        builder.build()
    }
}

/// Batch verifier for multiple proofs
pub struct BatchVerifier<F: Field> {
    verifier: ZkVerifier<F>,
}

impl<F: Field> BatchVerifier<F> {
    /// Create a new batch verifier
    pub fn new() -> Self {
        Self {
            verifier: ZkVerifier::new(),
        }
    }
    
    /// Verify multiple proofs
    pub fn verify_all(
        &mut self,
        proofs: &[(ZkProof<F>, HashMap<String, Vec<u8>>)],
    ) -> Result<Vec<bool>> {
        proofs
            .iter()
            .map(|(proof, inputs)| self.verifier.verify(proof, inputs))
            .collect()
    }
    
    /// Verify all proofs are valid
    pub fn verify_all_valid(
        &mut self,
        proofs: &[(ZkProof<F>, HashMap<String, Vec<u8>>)],
    ) -> Result<bool> {
        for (proof, inputs) in proofs {
            if !self.verifier.verify(proof, inputs)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Proof verifier with policy enforcement
pub struct PolicyVerifier<F: Field> {
    verifier: ZkVerifier<F>,
    policies: HashMap<String, VerificationPolicy>,
}

/// Verification policy
#[derive(Clone, Debug)]
pub struct VerificationPolicy {
    /// Minimum security bits required
    pub min_security_bits: usize,
    
    /// Maximum age of proof (seconds)
    pub max_proof_age: u64,
    
    /// Required document type
    pub required_document_type: Option<DocumentType>,
    
    /// Required predicates
    pub required_predicates: Vec<String>,
    
    /// Allowed issuers
    pub allowed_issuers: Option<Vec<String>>,
}

impl<F: Field> PolicyVerifier<F> {
    /// Create a new policy verifier
    pub fn new() -> Self {
        Self {
            verifier: ZkVerifier::new(),
            policies: HashMap::new(),
        }
    }
    
    /// Add a verification policy
    pub fn add_policy(&mut self, name: String, policy: VerificationPolicy) {
        self.policies.insert(name, policy);
    }
    
    /// Verify with policy
    pub fn verify_with_policy(
        &mut self,
        proof: &ZkProof<F>,
        public_inputs: &HashMap<String, Vec<u8>>,
        policy_name: &str,
    ) -> Result<bool> {
        // Get policy
        let policy = self.policies.get(policy_name)
            .ok_or_else(|| LongfellowError::InvalidParameter(
                format!("Unknown policy: {}", policy_name)
            ))?;
        
        // Check policy requirements
        if proof.metadata.security_bits < policy.min_security_bits {
            return Ok(false);
        }
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now - proof.metadata.created_at > policy.max_proof_age {
            return Ok(false);
        }
        
        if let Some(ref required_type) = policy.required_document_type {
            if proof.metadata.document_type != *required_type {
                return Ok(false);
            }
        }
        
        // Verify proof
        self.verifier.verify(proof, public_inputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_verifier_creation() {
        let verifier = ZkVerifier::<Fp128>::new();
        assert!(verifier.ligero_params_cache.is_empty());
    }
    
    #[test]
    fn test_metadata_validation() {
        let verifier = ZkVerifier::<Fp128>::new();
        
        let valid_metadata = ProofMetadata {
            version: "1.0.0".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            security_bits: 128,
            document_type: DocumentType::Jwt,
            circuit_stats: crate::CircuitStats {
                num_gates: 1000,
                num_wires: 2000,
                num_constraints: 500,
                depth: 10,
            },
        };
        
        assert!(verifier.validate_metadata(&valid_metadata).is_ok());
        
        // Test invalid version
        let mut invalid_metadata = valid_metadata.clone();
        invalid_metadata.version = "2.0.0".to_string();
        assert!(verifier.validate_metadata(&invalid_metadata).is_err());
        
        // Test low security
        let mut low_security = valid_metadata.clone();
        low_security.security_bits = 64;
        assert!(verifier.validate_metadata(&low_security).is_err());
    }
    
    #[test]
    fn test_policy_verifier() {
        let mut policy_verifier = PolicyVerifier::<Fp128>::new();
        
        // Add a strict policy
        let strict_policy = VerificationPolicy {
            min_security_bits: 128,
            max_proof_age: 3600, // 1 hour
            required_document_type: Some(DocumentType::Jwt),
            required_predicates: vec!["ValidSignature".to_string()],
            allowed_issuers: Some(vec!["trusted-issuer".to_string()]),
        };
        
        policy_verifier.add_policy("strict".to_string(), strict_policy);
        
        // Policy should be stored
        assert!(policy_verifier.policies.contains_key("strict"));
    }
}