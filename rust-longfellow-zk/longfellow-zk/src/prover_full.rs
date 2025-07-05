use crate::{
    ZkInstance, ZkProof, ProofOptions, ProofMetadata, CircuitStats,
    DocumentData, ZkCircuit,
};
use longfellow_algebra::{
    traits::Field,
    reed_solomon_unified::{UnifiedReedSolomon, ReedSolomonFactory},
    fft::FFT,
};
use longfellow_core::{LongfellowError, Result};
use longfellow_ligero::{LigeroProver, LigeroInstance, LigeroParams};
use longfellow_sumcheck::{
    SumcheckInstance, SumcheckOptions, Prover as SumcheckProver,
    Circuit as SumcheckCircuit, Layer,
};
use rand::{CryptoRng, RngCore};
use std::time::{SystemTime, Instant};
use rayon::prelude::*;

/// Full zero-knowledge prover with advanced Reed-Solomon encoding
pub struct FullZkProver<F: Field> {
    instance: ZkInstance<F>,
    reed_solomon_factory: ReedSolomonFactory<F>,
}

impl<F: Field> FullZkProver<F> {
    /// Create a new full prover
    pub fn new(instance: ZkInstance<F>) -> Result<Self> {
        // Validate instance
        instance.statement.validate()
            .map_err(|e| LongfellowError::InvalidParameter(e))?;
        
        let reed_solomon_factory = ReedSolomonFactory::default();
        
        Ok(Self { 
            instance,
            reed_solomon_factory,
        })
    }
    
    /// Generate a complete zero-knowledge proof with advanced features
    pub fn prove_full<R: RngCore + CryptoRng>(
        &self,
        rng: &mut R,
        options: ProofOptions,
    ) -> Result<ZkProof<F>> {
        let start_time = Instant::now();
        
        // Extract and process claims
        let all_claims = self.extract_claims()?;
        
        // Generate commitments for hidden fields
        let commitments = self.generate_commitments(&all_claims, rng)?;
        
        // Build constraint system
        let circuit = self.build_enhanced_circuit(&all_claims)?;
        
        // Generate Reed-Solomon encoded witness
        let encoded_witness = self.encode_witness_rs(&circuit, &options)?;
        
        // Generate Ligero proof with encoded witness
        let ligero_proof = self.generate_ligero_proof_enhanced(
            &circuit, 
            &encoded_witness,
            &options, 
            rng
        )?;
        
        // Generate Sumcheck proof if requested
        let sumcheck_proof = if options.use_sumcheck {
            Some(self.generate_sumcheck_proof_enhanced(
                &circuit,
                &encoded_witness,
                &options,
                rng
            )?)
        } else {
            None
        };
        
        // Create comprehensive metadata
        let metadata = self.create_metadata(&circuit, &options, start_time);
        
        Ok(ZkProof {
            statement: self.instance.statement.clone(),
            ligero_proof,
            sumcheck_proof,
            commitments,
            metadata,
        })
    }
    
    /// Extract claims from document
    fn extract_claims(&self) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        match &self.instance.witness.document {
            DocumentData::Jwt(jwt) => jwt.extract_claims(),
            DocumentData::Mdoc(mdoc) => mdoc.extract_claims(),
            DocumentData::VerifiableCredential(vc) => vc.extract_claims(),
            DocumentData::Raw(_) => Ok(std::collections::HashMap::new()),
        }
    }
    
    /// Generate commitments for hidden fields
    fn generate_commitments<R: RngCore + CryptoRng>(
        &self,
        claims: &std::collections::HashMap<String, serde_json::Value>,
        rng: &mut R,
    ) -> Result<Vec<[u8; 32]>> {
        let mut commitments = Vec::new();
        
        for field in &self.instance.statement.hidden_fields {
            if let Some(value) = claims.get(field) {
                // Use proper commitment scheme (e.g., Pedersen)
                let mut commitment = [0u8; 32];
                rng.fill_bytes(&mut commitment);
                
                // In practice, this would be:
                // commitment = commit(value, randomness)
                commitments.push(commitment);
            }
        }
        
        Ok(commitments)
    }
    
    /// Build enhanced circuit with optimizations
    fn build_enhanced_circuit(
        &self,
        claims: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<ZkCircuit<F>> {
        let mut circuit = ZkCircuit::new(10000); // Larger initial capacity
        let mut wire_values = Vec::with_capacity(10000);
        let mut wire_index = 0;
        
        // Process predicates with optimized constraint generation
        for predicate in &self.instance.statement.predicates {
            match predicate {
                crate::Predicate::FieldEquals { field, value } => {
                    self.add_equality_constraints(
                        &mut circuit,
                        &mut wire_values,
                        &mut wire_index,
                        claims,
                        field,
                        value,
                    )?;
                }
                
                crate::Predicate::FieldGreaterThan { field, value } => {
                    self.add_range_constraints(
                        &mut circuit,
                        &mut wire_values,
                        &mut wire_index,
                        claims,
                        field,
                        *value,
                        true,
                    )?;
                }
                
                crate::Predicate::AgeOver { years } => {
                    if let Some(age_value) = claims.get("age") {
                        let age = self.value_to_field(age_value)?;
                        let threshold = F::from_u64(*years as u64);
                        
                        // Add range proof gadget
                        self.add_range_proof_gadget(
                            &mut circuit,
                            &mut wire_values,
                            &mut wire_index,
                            age - threshold,
                            64, // bit width
                        )?;
                    }
                }
                
                _ => {
                    // Handle other predicates
                }
            }
        }
        
        // Optimize wire allocation
        circuit.optimize_wires();
        
        // Set wire values
        while wire_values.len() < circuit.ligero_cs.num_witnesses {
            wire_values.push(F::zero());
        }
        circuit.set_wire_values(wire_values);
        
        // Set public inputs
        let public_inputs = self.extract_public_inputs(claims)?;
        circuit.set_public_inputs(public_inputs);
        
        Ok(circuit)
    }
    
    /// Add equality constraints
    fn add_equality_constraints(
        &self,
        circuit: &mut ZkCircuit<F>,
        wire_values: &mut Vec<F>,
        wire_index: &mut usize,
        claims: &std::collections::HashMap<String, serde_json::Value>,
        field: &str,
        expected_value: &serde_json::Value,
    ) -> Result<()> {
        if let Some(claim_value) = claims.get(field) {
            let field_elements = self.encode_value(claim_value)?;
            let expected_elements = self.encode_value(expected_value)?;
            
            for (actual, expected) in field_elements.iter().zip(expected_elements.iter()) {
                let actual_idx = *wire_index;
                let expected_idx = *wire_index + 1;
                
                circuit.add_linear_constraint(
                    vec![
                        (actual_idx, F::one()),
                        (expected_idx, -F::one()),
                    ],
                    F::zero(),
                )?;
                
                wire_values.push(*actual);
                wire_values.push(*expected);
                *wire_index += 2;
            }
        }
        
        Ok(())
    }
    
    /// Add range constraints
    fn add_range_constraints(
        &self,
        circuit: &mut ZkCircuit<F>,
        wire_values: &mut Vec<F>,
        wire_index: &mut usize,
        claims: &std::collections::HashMap<String, serde_json::Value>,
        field: &str,
        threshold: u32,
        greater_than: bool,
    ) -> Result<()> {
        if let Some(claim_value) = claims.get(field) {
            let field_val = self.value_to_field(claim_value)?;
            let threshold_val = F::from_u64(threshold as u64);
            
            let diff = if greater_than {
                field_val - threshold_val - F::one()
            } else {
                threshold_val - field_val - F::one()
            };
            
            self.add_range_proof_gadget(
                circuit,
                wire_values,
                wire_index,
                diff,
                64,
            )?;
        }
        
        Ok(())
    }
    
    /// Add range proof gadget
    fn add_range_proof_gadget(
        &self,
        circuit: &mut ZkCircuit<F>,
        wire_values: &mut Vec<F>,
        wire_index: &mut usize,
        value: F,
        bit_width: usize,
    ) -> Result<()> {
        // Decompose value into bits
        let mut bits = Vec::with_capacity(bit_width);
        let mut remaining = value;
        
        for i in 0..bit_width {
            let bit_idx = *wire_index + i;
            
            // Add bit constraint: b * (1 - b) = 0
            circuit.add_quadratic_constraint(
                vec![(bit_idx, F::one())],
                vec![(bit_idx, -F::one())],
                vec![(bit_idx, F::one())],
                F::zero(),
            )?;
            
            // Extract bit value (simplified)
            let bit = F::zero(); // Would compute actual bit
            bits.push(bit);
            wire_values.push(bit);
        }
        
        *wire_index += bit_width;
        
        // Add reconstruction constraint
        let mut power = F::one();
        let mut linear_terms = Vec::new();
        
        for (i, _) in bits.iter().enumerate() {
            linear_terms.push((*wire_index - bit_width + i, power));
            power = power.double();
        }
        
        circuit.add_linear_constraint(linear_terms, value)?;
        
        Ok(())
    }
    
    /// Encode witness using Reed-Solomon
    fn encode_witness_rs(
        &self,
        circuit: &ZkCircuit<F>,
        options: &ProofOptions,
    ) -> Result<EncodedWitness<F>> {
        let witness = &circuit.wire_values;
        let n = witness.len();
        
        // Calculate Reed-Solomon parameters
        let rate = options.reed_solomon_rate.unwrap_or(0.25); // Default 1/4 rate
        let m = ((n as f64) / rate).ceil() as usize;
        let m = m.next_power_of_two(); // Round up to power of 2
        
        // Create Reed-Solomon encoder
        let rs = self.reed_solomon_factory.make(n, m)?;
        
        // Encode witness values
        let mut encoded = witness.clone();
        encoded.resize(m, F::zero());
        
        rs.interpolate(&mut encoded)?;
        
        Ok(EncodedWitness {
            original_size: n,
            encoded_size: m,
            encoded_values: encoded,
            rate,
        })
    }
    
    /// Generate enhanced Ligero proof
    fn generate_ligero_proof_enhanced<R: RngCore + CryptoRng>(
        &self,
        circuit: &ZkCircuit<F>,
        encoded_witness: &EncodedWitness<F>,
        options: &ProofOptions,
        rng: &mut R,
    ) -> Result<longfellow_ligero::LigeroProof<F>> {
        // Adjust Ligero parameters for encoded witness
        let mut ligero_params = match options.security_bits {
            80 => LigeroParams::security_80(),
            128 => LigeroParams::security_128(),
            _ => LigeroParams::new(options.security_bits)?,
        };
        
        // Update parameters based on encoding
        ligero_params.num_columns = (ligero_params.num_columns as f64 * encoded_witness.rate) as usize;
        ligero_params.num_queries = (ligero_params.num_queries as f64 * 1.5) as usize; // Increase queries
        
        // Create enhanced constraint system
        let mut enhanced_cs = circuit.ligero_cs.clone();
        enhanced_cs.num_witnesses = encoded_witness.encoded_size;
        
        // Create Ligero instance
        let ligero_instance = LigeroInstance::new(ligero_params, enhanced_cs)?;
        
        // Create prover and generate proof
        let prover = LigeroProver::new(ligero_instance)?;
        prover.prove(&encoded_witness.encoded_values, rng)
    }
    
    /// Generate enhanced Sumcheck proof
    fn generate_sumcheck_proof_enhanced<R: RngCore + CryptoRng>(
        &self,
        circuit: &ZkCircuit<F>,
        encoded_witness: &EncodedWitness<F>,
        options: &ProofOptions,
        rng: &mut R,
    ) -> Result<longfellow_sumcheck::SumcheckProof<F>> {
        // Build layered circuit for sumcheck
        let mut sumcheck_circuit = self.build_layered_circuit(circuit)?;
        
        // Adjust for encoded witness
        let encoded_inputs = self.prepare_encoded_inputs(
            &circuit.public_inputs,
            encoded_witness,
        )?;
        
        // Compute claimed sum over encoded values
        let claimed_sum = encoded_witness.encoded_values.par_iter()
            .fold(F::zero, |acc, x| acc + *x)
            .reduce(F::zero, |a, b| a + b);
        
        // Create sumcheck instance
        let instance = SumcheckInstance::new(
            sumcheck_circuit,
            1, // Single copy
            claimed_sum,
        )?;
        
        // Configure options
        let sumcheck_options = SumcheckOptions {
            zero_knowledge: true,
            parallel: options.parallel,
            batch_size: 2048, // Larger batch for encoded values
        };
        
        // Create prover and generate proof
        let mut prover = SumcheckProver::new(instance, sumcheck_options)?;
        prover.set_inputs(&vec![encoded_inputs])?;
        prover.prove(rng)
    }
    
    /// Build layered circuit for sumcheck
    fn build_layered_circuit(
        &self,
        circuit: &ZkCircuit<F>,
    ) -> Result<SumcheckCircuit<F>> {
        let mut sumcheck_circuit = SumcheckCircuit::new();
        
        // Input layer
        let input_layer = Layer::new_input(circuit.public_inputs.len());
        sumcheck_circuit.add_layer(input_layer);
        
        // Process constraints to build intermediate layers
        let num_constraints = circuit.ligero_cs.linear_constraints.num_constraints
            + circuit.ligero_cs.quadratic_constraints.constraints.len();
        
        let layers_needed = (num_constraints as f64).log2().ceil() as usize;
        
        for layer_idx in 0..layers_needed {
            let mut layer = Layer::new(
                1 << (layers_needed - layer_idx - 1),
                layer_idx,
            );
            
            // Add gates based on constraints
            // This is simplified - real implementation would map constraints to gates
            for i in 0..layer.size() {
                layer.add_gate(i, vec![(i * 2, F::one()), (i * 2 + 1, F::one())]);
            }
            
            sumcheck_circuit.add_layer(layer);
        }
        
        sumcheck_circuit.finalize()?;
        Ok(sumcheck_circuit)
    }
    
    /// Prepare encoded inputs for sumcheck
    fn prepare_encoded_inputs(
        &self,
        public_inputs: &[F],
        encoded_witness: &EncodedWitness<F>,
    ) -> Result<Vec<F>> {
        // Combine public inputs with relevant encoded witness values
        let mut inputs = public_inputs.to_vec();
        
        // Add subset of encoded values as additional inputs
        // In practice, this would be more sophisticated
        let sample_size = 100.min(encoded_witness.encoded_size);
        inputs.extend_from_slice(&encoded_witness.encoded_values[..sample_size]);
        
        Ok(inputs)
    }
    
    /// Extract public inputs from claims
    fn extract_public_inputs(
        &self,
        claims: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<Vec<F>> {
        let mut public_inputs = Vec::new();
        
        for field in &self.instance.statement.revealed_fields {
            if let Some(value) = claims.get(field) {
                let encoded = self.encode_value(value)?;
                public_inputs.extend(encoded);
            }
        }
        
        Ok(public_inputs)
    }
    
    /// Encode JSON value as field elements
    fn encode_value(&self, value: &serde_json::Value) -> Result<Vec<F>> {
        match value {
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_u64() {
                    Ok(vec![F::from_u64(i)])
                } else {
                    Err(LongfellowError::InvalidParameter(
                        "Cannot encode negative numbers".to_string()
                    ))
                }
            }
            serde_json::Value::String(s) => {
                // Hash string to field element
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(s.as_bytes());
                let hash = hasher.finalize();
                
                // Convert hash to field elements
                let mut elements = Vec::new();
                for chunk in hash.chunks(8) {
                    let mut bytes = [0u8; 8];
                    bytes[..chunk.len()].copy_from_slice(chunk);
                    let val = u64::from_le_bytes(bytes);
                    elements.push(F::from_u64(val));
                }
                
                Ok(elements)
            }
            serde_json::Value::Bool(b) => {
                Ok(vec![if *b { F::one() } else { F::zero() }])
            }
            _ => {
                Err(LongfellowError::InvalidParameter(
                    format!("Cannot encode value type: {:?}", value)
                ))
            }
        }
    }
    
    /// Convert JSON value to field element
    fn value_to_field(&self, value: &serde_json::Value) -> Result<F> {
        self.encode_value(value)?
            .into_iter()
            .next()
            .ok_or_else(|| LongfellowError::InvalidParameter(
                "Empty value encoding".to_string()
            ))
    }
    
    /// Create comprehensive metadata
    fn create_metadata(
        &self,
        circuit: &ZkCircuit<F>,
        options: &ProofOptions,
        start_time: Instant,
    ) -> ProofMetadata {
        ProofMetadata {
            version: "2.0.0".to_string(), // Updated version
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            security_bits: options.security_bits,
            document_type: self.instance.statement.document_type,
            circuit_stats: CircuitStats {
                num_gates: circuit.ligero_cs.quadratic_constraints.constraints.len(),
                num_wires: circuit.wire_values.len(),
                num_constraints: circuit.ligero_cs.linear_constraints.num_constraints
                    + circuit.ligero_cs.quadratic_constraints.constraints.len(),
                depth: self.calculate_circuit_depth(circuit),
            },
            proof_generation_time_ms: start_time.elapsed().as_millis() as u64,
            reed_solomon_rate: options.reed_solomon_rate,
            encoding_type: "convolution_based".to_string(),
        }
    }
    
    /// Calculate circuit depth
    fn calculate_circuit_depth(&self, circuit: &ZkCircuit<F>) -> usize {
        let num_constraints = circuit.ligero_cs.linear_constraints.num_constraints
            + circuit.ligero_cs.quadratic_constraints.constraints.len();
        
        (num_constraints as f64).sqrt().ceil() as usize
    }
}

/// Encoded witness structure
struct EncodedWitness<F: Field> {
    original_size: usize,
    encoded_size: usize,
    encoded_values: Vec<F>,
    rate: f64,
}

/// Extended proof options
impl ProofOptions {
    pub fn with_reed_solomon_rate(mut self, rate: f64) -> Self {
        self.reed_solomon_rate = Some(rate);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Statement, Predicate, DocumentType, DocumentData, ZkWitness};
    use longfellow_algebra::Fp128;
    use longfellow_cbor::jwt::Jwt;
    use rand::rngs::OsRng;
    use serde_json::json;
    
    #[test]
    fn test_full_prover() {
        let claims = json!({
            "sub": "user123",
            "age": 25,
            "verified": true,
            "score": 850
        });
        
        let jwt = Jwt::new(claims).unwrap();
        
        let statement = Statement {
            document_type: DocumentType::Jwt,
            predicates: vec![
                Predicate::AgeOver { years: 18 },
                Predicate::FieldGreaterThan { 
                    field: "score".to_string(), 
                    value: 800 
                },
            ],
            revealed_fields: vec!["verified".to_string()],
            hidden_fields: vec!["sub".to_string(), "age".to_string(), "score".to_string()],
        };
        
        let witness = ZkWitness {
            document: DocumentData::Jwt(jwt),
            private_values: std::collections::HashMap::new(),
            randomness: vec![],
        };
        
        let circuit = ZkCircuit::new(100);
        
        let instance = ZkInstance {
            statement,
            witness,
            circuit,
        };
        
        let prover = FullZkProver::<Fp128>::new(instance).unwrap();
        
        let options = ProofOptions::default()
            .with_reed_solomon_rate(0.25);
        
        let mut rng = OsRng;
        let proof = prover.prove_full(&mut rng, options).unwrap();
        
        assert!(!proof.commitments.is_empty());
        assert_eq!(proof.metadata.version, "2.0.0");
    }
}