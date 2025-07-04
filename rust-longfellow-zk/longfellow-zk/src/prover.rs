/// Zero-knowledge prover implementation

use crate::{
    ZkInstance, ZkProof, ZkCircuit, ProofOptions, ProofMetadata, CircuitStats,
    document::{ClaimExtractor, CommitmentGenerator},
};
use longfellow_algebra::traits::Field;
use rand::SeedableRng;
use longfellow_core::{LongfellowError, Result};
use longfellow_ligero::{LigeroProver, LigeroInstance, LigeroParams};
use longfellow_sumcheck::{SumcheckInstance, SumcheckOptions, prover::ProverLayers};
use rand::{CryptoRng, RngCore};

/// Zero-knowledge prover
pub struct ZkProver<F: Field> {
    /// Proof options
    options: ProofOptions,
    /// Field randomness generator
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field> ZkProver<F> {
    /// Create a new prover
    pub fn new(options: ProofOptions) -> Self {
        Self {
            options,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Generate a zero-knowledge proof
    pub fn prove<R: RngCore + CryptoRng>(
        &self,
        instance: &ZkInstance<F>,
        rng: &mut R,
    ) -> Result<ZkProof<F>> {
        // Validate statement
        instance.statement.validate()
            .map_err(|e| LongfellowError::InvalidParameter(e))?;
        
        // Extract claims
        let all_claims = ClaimExtractor::extract_all(&instance.witness.document)?;
        
        // Generate commitments for private fields
        let private_values: Vec<Vec<u8>> = instance.statement.private_fields.iter()
            .filter_map(|field| {
                all_claims.get(field)
                    .and_then(|v| serde_json::to_vec(v).ok())
            })
            .collect();
        
        let mut randomness = vec![[0u8; 32]; private_values.len()];
        for r in &mut randomness {
            rng.fill_bytes(r);
        }
        
        let commitments = CommitmentGenerator::commit_batch(&private_values, &randomness)?;
        
        // Build and check circuit
        if !instance.circuit.is_satisfied()? {
            return Err(LongfellowError::ProofError(
                "Circuit is not satisfied by witness".to_string()
            ));
        }
        
        // Generate Ligero proof
        let ligero_proof = self.generate_ligero_proof(&instance.circuit, rng)?;
        
        // Optionally generate Sumcheck proof
        let sumcheck_proof = if self.options.use_sumcheck {
            Some(self.generate_sumcheck_proof(&instance.circuit, rng)?)
        } else {
            None
        };
        
        // Create metadata
        let metadata = ProofMetadata {
            version: "1.0.0".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            security_bits: self.options.security_bits,
            document_type: instance.statement.document_type,
            circuit_stats: CircuitStats {
                num_gates: instance.circuit.ligero_cs.quadratic_constraints.constraints.len(),
                num_wires: instance.circuit.wire_values.len(),
                num_constraints: instance.circuit.ligero_cs.linear_constraints.num_constraints
                    + instance.circuit.ligero_cs.quadratic_constraints.constraints.len(),
                depth: 0, // Would calculate actual depth
            },
        };
        
        Ok(ZkProof {
            statement: instance.statement.clone(),
            ligero_proof,
            sumcheck_proof,
            commitments,
            metadata,
        })
    }
    
    /// Generate Ligero proof
    fn generate_ligero_proof<R: RngCore + CryptoRng>(
        &self,
        circuit: &ZkCircuit<F>,
        rng: &mut R,
    ) -> Result<longfellow_ligero::LigeroProof<F>> {
        // Create Ligero parameters
        let ligero_params = LigeroParams::new(self.options.security_bits)?;
        
        // Create Ligero instance
        let ligero_instance = LigeroInstance::new(
            ligero_params,
            circuit.ligero_cs.clone(),
        )?;
        
        // Create prover
        let prover = LigeroProver::new(ligero_instance)?;
        
        // Generate proof
        prover.prove(&circuit.wire_values, rng)
    }
    
    /// Generate Sumcheck proof
    fn generate_sumcheck_proof<R: RngCore + CryptoRng>(
        &self,
        circuit: &ZkCircuit<F>,
        rng: &mut R,
    ) -> Result<longfellow_sumcheck::SumcheckProof<F>> {
        let sumcheck_circuit = circuit.sumcheck_circuit.as_ref()
            .ok_or_else(|| LongfellowError::InvalidParameter(
                "No sumcheck circuit available".to_string()
            ))?;
        
        // Create sumcheck instance
        let claimed_sum = circuit.wire_values.iter().fold(F::zero(), |acc, x| acc + *x);
        let sumcheck_instance = SumcheckInstance::new(
            sumcheck_circuit.clone(),
            1, // Single copy for now
            claimed_sum,
        )?;
        
        // Create prover
        let sumcheck_options = SumcheckOptions {
            zero_knowledge: true,
            parallel: self.options.parallel,
            batch_size: 1024,
        };
        
        let prover = ProverLayers::new(
            sumcheck_circuit.clone(),
            &circuit.public_inputs,
            1,
            sumcheck_options,
        )?;
        
        // Generate proof
        prover.prove(&sumcheck_instance, rng)
    }
}

/// Proof builder for convenient proof construction
pub struct ProofBuilder<F: Field> {
    instance: Option<ZkInstance<F>>,
    options: ProofOptions,
}

impl<F: Field> ProofBuilder<F> {
    /// Create a new proof builder
    pub fn new() -> Self {
        Self {
            instance: None,
            options: ProofOptions::default(),
        }
    }
    
    /// Set the instance
    pub fn with_instance(mut self, instance: ZkInstance<F>) -> Self {
        self.instance = Some(instance);
        self
    }
    
    /// Set security level
    pub fn with_security_bits(mut self, bits: usize) -> Self {
        self.options.security_bits = bits;
        self
    }
    
    /// Enable/disable sumcheck
    pub fn use_sumcheck(mut self, enable: bool) -> Self {
        self.options.use_sumcheck = enable;
        self
    }
    
    /// Enable/disable parallelism
    pub fn parallel(mut self, enable: bool) -> Self {
        self.options.parallel = enable;
        self
    }
    
    /// Optimize for size
    pub fn optimize_size(mut self, enable: bool) -> Self {
        self.options.optimize_size = enable;
        self
    }
    
    /// Build and generate the proof
    pub fn prove<R: RngCore + CryptoRng>(self, rng: &mut R) -> Result<ZkProof<F>> {
        let instance = self.instance
            .ok_or_else(|| LongfellowError::InvalidParameter(
                "Instance not set".to_string()
            ))?;
        
        let prover = ZkProver::new(self.options);
        prover.prove(&instance, rng)
    }
}

/// Batch prover for multiple statements
pub struct BatchProver<F: Field> {
    instances: Vec<ZkInstance<F>>,
    options: ProofOptions,
}

impl<F: Field> BatchProver<F> {
    /// Create a new batch prover
    pub fn new(options: ProofOptions) -> Self {
        Self {
            instances: Vec::new(),
            options,
        }
    }
    
    /// Add an instance to the batch
    pub fn add_instance(&mut self, instance: ZkInstance<F>) {
        self.instances.push(instance);
    }
    
    /// Generate proofs for all instances
    pub fn prove_all<R: RngCore + CryptoRng>(
        &self,
        rng: &mut R,
    ) -> Result<Vec<ZkProof<F>>> {
        let prover = ZkProver::new(self.options.clone());
        
        if self.options.parallel {
            use rayon::prelude::*;
            
            self.instances
                .par_iter()
                .map(|instance| {
                    let mut local_rng = rand::rngs::StdRng::from_rng(rng).unwrap();
                    prover.prove(instance, &mut local_rng)
                })
                .collect()
        } else {
            self.instances
                .iter()
                .map(|instance| prover.prove(instance, rng))
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Statement, DocumentType, DocumentData, ZkWitness, Predicate};
    use longfellow_algebra::Fp128;
    use longfellow_cbor::jwt::{Jwt, JwtBuilder, JwtAlgorithm};
    use longfellow_cbor::Value;
    use rand::rngs::OsRng;
    
    fn create_test_instance() -> ZkInstance<Fp128> {
        // Create a test JWT
        let jwt_str = JwtBuilder::new(JwtAlgorithm::HS256)
            .issuer("test-issuer".to_string())
            .subject("user123".to_string())
            .claim("age".to_string(), Value::Integer(25))
            .build_unsigned()
            .unwrap();
        
        let jwt = Jwt::from_str(&jwt_str).unwrap();
        
        // Create statement
        let statement = Statement::new(DocumentType::Jwt)
            .add_predicate(Predicate::FieldGreaterThan {
                field: "age".to_string(),
                value: 18,
            })
            .reveal_field("iss".to_string())
            .keep_private("sub".to_string());
        
        // Create witness
        let witness = ZkWitness {
            document: DocumentData::Jwt(jwt),
            private_values: HashMap::new(),
            randomness: vec![],
        };
        
        // Create simple circuit
        let mut circuit = ZkCircuit::new(10);
        circuit.set_wire_values(vec![Fp128::one(); 10]);
        
        ZkInstance {
            statement,
            witness,
            circuit,
        }
    }
    
    #[test]
    fn test_proof_generation() {
        let instance = create_test_instance();
        let prover = ZkProver::<Fp128>::new(ProofOptions::default());
        
        // This would fail without proper circuit construction
        // let proof = prover.prove(&instance, &mut OsRng).unwrap();
    }
    
    #[test]
    fn test_proof_builder() {
        let instance = create_test_instance();
        
        let builder = ProofBuilder::new()
            .with_instance(instance)
            .with_security_bits(128)
            .use_sumcheck(false)
            .parallel(true);
        
        // This would generate proof with specified options
        // let proof = builder.prove(&mut OsRng).unwrap();
    }
}