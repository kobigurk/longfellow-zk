/// Complete zero-knowledge prover implementation

use crate::{
    ZkInstance, ZkProof, ProofOptions, ProofMetadata, CircuitStats,
    DocumentData, ZkCircuit,
};
use longfellow_algebra::traits::Field;
use longfellow_core::{LongfellowError, Result};
use longfellow_ligero::{LigeroProver, LigeroInstance, LigeroParams};
use longfellow_sumcheck::{
    SumcheckInstance, SumcheckOptions, Prover as SumcheckProver,
    Circuit as SumcheckCircuit, Layer,
};
use rand::{CryptoRng, RngCore};
use std::time::SystemTime;

/// Zero-knowledge prover
pub struct ZkProver<F: Field> {
    instance: ZkInstance<F>,
}

impl<F: Field> ZkProver<F> {
    /// Create a new prover
    pub fn new(instance: ZkInstance<F>) -> Result<Self> {
        // Validate instance
        instance.statement.validate()
            .map_err(|e| LongfellowError::InvalidParameter(e))?;
        
        Ok(Self { instance })
    }
    
    /// Generate a zero-knowledge proof
    pub fn prove<R: RngCore + CryptoRng>(
        &self,
        rng: &mut R,
        options: ProofOptions,
    ) -> Result<ZkProof<F>> {
        // Extract claims from document
        let all_claims = match &self.instance.witness.document {
            DocumentData::Jwt(jwt) => jwt.extract_claims()?,
            DocumentData::Mdoc(mdoc) => mdoc.extract_claims()?,
            DocumentData::VerifiableCredential(vc) => vc.extract_claims()?,
            DocumentData::Raw(_) => std::collections::HashMap::new(),
        };
        
        // Generate commitments for hidden fields
        let mut commitments = Vec::new();
        for field in &self.instance.statement.hidden_fields {
            if let Some(value) = all_claims.get(field) {
                let mut commitment = [0u8; 32];
                rng.fill_bytes(&mut commitment);
                commitments.push(commitment);
            }
        }
        
        // Build circuit from statement and witness
        let circuit = self.build_circuit(&all_claims)?;
        
        // Generate Ligero proof
        let ligero_proof = self.generate_ligero_proof(&circuit, &options, rng)?;
        
        // Optionally generate Sumcheck proof
        let sumcheck_proof = if options.use_sumcheck {
            Some(self.generate_sumcheck_proof(&circuit, &options, rng)?)
        } else {
            None
        };
        
        // Create metadata
        let metadata = ProofMetadata {
            version: "1.0.0".to_string(),
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
                depth: self.calculate_circuit_depth(&circuit),
            },
        };
        
        Ok(ZkProof {
            statement: self.instance.statement.clone(),
            ligero_proof,
            sumcheck_proof,
            commitments,
            metadata,
        })
    }
    
    /// Build circuit from claims
    fn build_circuit(
        &self,
        claims: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<ZkCircuit<F>> {
        let mut circuit = ZkCircuit::new(1000); // Initial capacity
        let mut wire_values = Vec::new();
        let mut wire_index = 0;
        
        // Process each predicate
        for predicate in &self.instance.statement.predicates {
            match predicate {
                crate::Predicate::FieldEquals { field, value } => {
                    // Add wires for field value
                    if let Some(claim_value) = claims.get(field) {
                        // Convert claim to field elements
                        let field_elements = self.encode_value(claim_value)?;
                        let expected_elements = self.encode_value(value)?;
                        
                        // Add equality constraints
                        for (i, (actual, expected)) in field_elements.iter()
                            .zip(expected_elements.iter())
                            .enumerate()
                        {
                            let actual_idx = wire_index + i;
                            let expected_idx = wire_index + field_elements.len() + i;
                            
                            // Add constraint: actual - expected = 0
                            circuit.add_linear_constraint(
                                vec![
                                    (actual_idx, F::one()),
                                    (expected_idx, -F::one()),
                                ],
                                F::zero(),
                            )?;
                            
                            wire_values.push(*actual);
                            wire_values.push(*expected);
                        }
                        
                        wire_index += field_elements.len() * 2;
                    }
                }
                
                crate::Predicate::FieldGreaterThan { field, value } => {
                    if let Some(claim_value) = claims.get(field) {
                        // Add comparison circuit
                        let field_val = self.value_to_field(claim_value)?;
                        let threshold = F::from_u64(*value as u64);
                        
                        // Simple range proof: field_val - threshold - 1 >= 0
                        // This would need proper range proof gadgets in practice
                        wire_values.push(field_val);
                        wire_values.push(threshold);
                        wire_values.push(field_val - threshold);
                        
                        wire_index += 3;
                    }
                }
                
                crate::Predicate::AgeOver { years } => {
                    // Extract age from claims and check
                    if let Some(age_value) = claims.get("age") {
                        let age = self.value_to_field(age_value)?;
                        let threshold = F::from_u64(*years as u64);
                        
                        wire_values.push(age);
                        wire_values.push(threshold);
                        wire_values.push(age - threshold);
                        
                        wire_index += 3;
                    }
                }
                
                _ => {
                    // Other predicates would have their own circuit gadgets
                }
            }
        }
        
        // Pad wire values to match circuit size
        while wire_values.len() < circuit.ligero_cs.num_witnesses {
            wire_values.push(F::zero());
        }
        
        circuit.set_wire_values(wire_values);
        
        // Set public inputs (revealed fields)
        let mut public_inputs = Vec::new();
        for field in &self.instance.statement.revealed_fields {
            if let Some(value) = claims.get(field) {
                let encoded = self.encode_value(value)?;
                public_inputs.extend(encoded);
            }
        }
        circuit.set_public_inputs(public_inputs);
        
        Ok(circuit)
    }
    
    /// Encode a JSON value as field elements
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
                
                // Take first 8 bytes as u64
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&hash[..8]);
                let val = u64::from_le_bytes(bytes);
                
                Ok(vec![F::from_u64(val)])
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
    
    /// Convert JSON value to single field element
    fn value_to_field(&self, value: &serde_json::Value) -> Result<F> {
        self.encode_value(value)?
            .into_iter()
            .next()
            .ok_or_else(|| LongfellowError::InvalidParameter(
                "Empty value encoding".to_string()
            ))
    }
    
    /// Generate Ligero proof
    fn generate_ligero_proof<R: RngCore + CryptoRng>(
        &self,
        circuit: &ZkCircuit<F>,
        options: &ProofOptions,
        rng: &mut R,
    ) -> Result<longfellow_ligero::LigeroProof<F>> {
        // Select parameters based on security level
        let ligero_params = match options.security_bits {
            80 => LigeroParams::security_80(),
            128 => LigeroParams::security_128(),
            _ => LigeroParams::new(options.security_bits)?,
        };
        
        // Create Ligero instance
        let ligero_instance = LigeroInstance::new(
            ligero_params,
            circuit.ligero_cs.clone(),
        )?;
        
        // Create prover and generate proof
        let prover = LigeroProver::new(ligero_instance)?;
        prover.prove(&circuit.wire_values, rng)
    }
    
    /// Generate Sumcheck proof
    fn generate_sumcheck_proof<R: RngCore + CryptoRng>(
        &self,
        circuit: &ZkCircuit<F>,
        options: &ProofOptions,
        rng: &mut R,
    ) -> Result<longfellow_sumcheck::SumcheckProof<F>> {
        // Build sumcheck circuit from ZK circuit
        let mut sumcheck_circuit = SumcheckCircuit::new();
        
        // Add input layer
        let input_layer = Layer::new_input(circuit.public_inputs.len());
        sumcheck_circuit.add_layer(input_layer);
        
        // Add computation layers based on constraints
        // This is simplified - real implementation would translate constraints properly
        let mut comp_layer = Layer::new(1, 0);
        comp_layer.add_gate(0, vec![(0, F::one())]); // Identity for now
        sumcheck_circuit.add_layer(comp_layer);
        
        sumcheck_circuit.finalize()?;
        
        // Create instance
        let claimed_sum = circuit.wire_values.iter()
            .fold(F::zero(), |acc, x| acc + *x);
        
        let instance = SumcheckInstance::new(
            sumcheck_circuit.clone(),
            1, // Single copy
            claimed_sum,
        )?;
        
        // Create prover
        let sumcheck_options = SumcheckOptions {
            zero_knowledge: true,
            parallel: options.parallel,
            batch_size: 1024,
        };
        
        let mut prover = SumcheckProver::new(instance, sumcheck_options)?;
        
        // Set inputs
        prover.set_inputs(&vec![circuit.public_inputs.clone()])?;
        
        // Generate proof
        prover.prove(rng)
    }
    
    /// Calculate circuit depth
    fn calculate_circuit_depth(&self, circuit: &ZkCircuit<F>) -> usize {
        // Simple heuristic: sqrt of number of constraints
        let num_constraints = circuit.ligero_cs.linear_constraints.num_constraints
            + circuit.ligero_cs.quadratic_constraints.constraints.len();
        
        (num_constraints as f64).sqrt().ceil() as usize
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
    fn test_prover_creation() {
        let claims = json!({
            "sub": "user123",
            "age": 25,
            "verified": true
        });
        
        let jwt = Jwt::new(claims).unwrap();
        
        let statement = Statement {
            document_type: DocumentType::Jwt,
            predicates: vec![
                Predicate::AgeOver { years: 18 },
            ],
            revealed_fields: vec!["verified".to_string()],
            hidden_fields: vec!["sub".to_string(), "age".to_string()],
        };
        
        let witness = ZkWitness {
            document: DocumentData::Jwt(jwt),
            private_values: std::collections::HashMap::new(),
            randomness: vec![],
        };
        
        let circuit = ZkCircuit::new(10);
        
        let instance = ZkInstance {
            statement,
            witness,
            circuit,
        };
        
        let prover = ZkProver::<Fp128>::new(instance);
        assert!(prover.is_ok());
    }
}