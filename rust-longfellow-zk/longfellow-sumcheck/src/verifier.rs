/// Sumcheck verifier implementation

use longfellow_algebra::traits::Field;
use longfellow_core::{LongfellowError, Result};

use crate::{
    SumcheckInstance, SumcheckProof, LayerProof,
    circuit::{Circuit, Layer},
    polynomial::UnivariatePoly,
    transcript::SumcheckTranscript,
};

/// Sumcheck verifier for a single layer
pub struct Verifier<F: Field> {
    layer: Layer<F>,
}

impl<F: Field> Verifier<F> {
    /// Create a new verifier
    pub fn new(layer: Layer<F>) -> Self {
        Self { layer }
    }
    
    /// Verify sumcheck proof for a layer
    pub fn verify_layer(
        &self,
        proof: &LayerProof<F>,
        claim: F,
        transcript: &mut SumcheckTranscript,
    ) -> Result<(bool, Vec<F>)> {
        let mut current_claim = claim;
        let mut round = 0;
        
        // Verify copy variable rounds
        let mut copy_bindings = Vec::new();
        for poly_coeffs in &proof.copy_polys {
            let poly = UnivariatePoly::new(poly_coeffs.clone());
            
            // Check sum: p(0) + p(1) + p(2) + p(3) = claim
            let sum = (0..=3)
                .map(|i| poly.evaluate(F::from(i as u64)))
                .sum::<F>();
            
            if sum != current_claim {
                return Ok((false, vec![]));
            }
            
            transcript.append_polynomial(round, &poly);
            let challenge = transcript.challenge_binding::<F>(round);
            
            current_claim = poly.evaluate(challenge);
            copy_bindings.push(challenge);
            round += 1;
        }
        
        // Verify hand variable rounds
        let mut hand_bindings = Vec::new();
        for poly_coeffs in &proof.hand_polys {
            let poly = UnivariatePoly::new(poly_coeffs.clone());
            
            // Check sum: p(0) + p(1) = claim
            let sum = poly.evaluate(F::zero()) + poly.evaluate(F::one());
            
            if sum != current_claim {
                return Ok((false, vec![]));
            }
            
            transcript.append_polynomial(round, &poly);
            let challenge = transcript.challenge_binding::<F>(round);
            
            current_claim = poly.evaluate(challenge);
            hand_bindings.push(challenge);
            round += 1;
        }
        
        // Verify wire claims
        transcript.append_wire_claims(0, &proof.wire_claims);
        
        // Check that wire claims sum to current claim
        let wire_sum: F = proof.wire_claims.iter().sum();
        if wire_sum != current_claim {
            return Ok((false, vec![]));
        }
        
        // Combine all bindings
        let mut all_bindings = copy_bindings;
        all_bindings.extend(hand_bindings);
        
        Ok((true, all_bindings))
    }
}

/// Sumcheck verifier for entire circuit
pub struct VerifierLayers<F: Field> {
    circuit: Circuit<F>,
}

impl<F: Field> VerifierLayers<F> {
    /// Create a new layered verifier
    pub fn new(circuit: Circuit<F>) -> Self {
        Self { circuit }
    }
    
    /// Verify complete sumcheck proof
    pub fn verify(
        &self,
        instance: &SumcheckInstance<F>,
        proof: &SumcheckProof<F>,
    ) -> Result<bool> {
        // Check proof structure
        if proof.layer_proofs.len() != self.circuit.layers.len() {
            return Ok(false);
        }
        
        let mut transcript = SumcheckTranscript::new(b"sumcheck");
        transcript.append_circuit_info(
            self.circuit.layers.len(),
            instance.num_copies,
            &instance.claimed_sum.to_bytes_le(),
        );
        
        let mut current_claim = instance.claimed_sum;
        let mut all_bindings = Vec::new();
        
        // Verify each layer
        for (layer_idx, (layer, layer_proof)) in self.circuit.layers.iter()
            .zip(&proof.layer_proofs)
            .enumerate() 
        {
            let verifier = Verifier::new(layer.clone());
            let (valid, bindings) = verifier.verify_layer(
                layer_proof,
                current_claim,
                &mut transcript,
            )?;
            
            if !valid {
                return Ok(false);
            }
            
            all_bindings.extend(bindings);
            
            // Update claim for next layer
            current_claim = layer_proof.wire_claims.iter().sum();
        }
        
        // Verify final input evaluation
        let expected_input_eval = self.compute_expected_input_eval(
            &all_bindings,
            &proof.input_eval,
        )?;
        
        if expected_input_eval != current_claim {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Compute expected input evaluation at binding point
    fn compute_expected_input_eval(
        &self,
        bindings: &[F],
        input_eval: &[F],
    ) -> Result<F> {
        // This would compute the multilinear extension of inputs
        // evaluated at the binding point
        if input_eval.is_empty() {
            return Err(LongfellowError::InvalidParameter(
                "Empty input evaluation".to_string()
            ));
        }
        
        // For now, return sum of input evaluations
        Ok(input_eval.iter().sum())
    }
    
    /// Verify with public inputs
    pub fn verify_with_public_inputs(
        &self,
        instance: &SumcheckInstance<F>,
        proof: &SumcheckProof<F>,
        public_inputs: &[F],
    ) -> Result<bool> {
        // First do standard verification
        if !self.verify(instance, proof)? {
            return Ok(false);
        }
        
        // Additionally check public inputs match
        if public_inputs.len() > self.circuit.num_public_inputs {
            return Ok(false);
        }
        
        // Would check that public inputs are consistent with proof
        // This requires evaluating the multilinear extension
        
        Ok(true)
    }
}

/// Helper functions for verification
impl<F: Field> VerifierLayers<F> {
    /// Check polynomial degree bounds
    pub fn check_degree_bounds(proof: &SumcheckProof<F>) -> bool {
        for layer_proof in &proof.layer_proofs {
            // Copy polynomials should have degree at most 3
            for poly in &layer_proof.copy_polys {
                if poly.len() > 4 {
                    return false;
                }
            }
            
            // Hand polynomials should have degree at most 2
            for poly in &layer_proof.hand_polys {
                if poly.len() > 3 {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Extract all challenges from a proof transcript
    pub fn extract_challenges(
        &self,
        instance: &SumcheckInstance<F>,
        proof: &SumcheckProof<F>,
    ) -> Result<Vec<F>> {
        let mut transcript = SumcheckTranscript::new(b"sumcheck");
        transcript.append_circuit_info(
            self.circuit.layers.len(),
            instance.num_copies,
            &instance.claimed_sum.to_bytes_le(),
        );
        
        let mut challenges = Vec::new();
        let mut round = 0;
        
        for layer_proof in &proof.layer_proofs {
            // Copy rounds
            for poly_coeffs in &layer_proof.copy_polys {
                let poly = UnivariatePoly::new(poly_coeffs.clone());
                transcript.append_polynomial(round, &poly);
                challenges.push(transcript.challenge_binding::<F>(round));
                round += 1;
            }
            
            // Hand rounds
            for poly_coeffs in &layer_proof.hand_polys {
                let poly = UnivariatePoly::new(poly_coeffs.clone());
                transcript.append_polynomial(round, &poly);
                challenges.push(transcript.challenge_binding::<F>(round));
                round += 1;
            }
            
            transcript.append_wire_claims(0, &layer_proof.wire_claims);
        }
        
        Ok(challenges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::{CircuitBuilder, GateType};
    use crate::prover::ProverLayers;
    use longfellow_algebra::Fp128;
    use rand::rngs::OsRng;
    
    #[test]
    fn test_simple_verification() {
        // Create a simple circuit: output = input[0] + input[1]
        let mut builder = CircuitBuilder::<Fp128>::new();
        builder.begin_layer(0, 1, 0).unwrap();
        builder.add_gate(0, 0, 1, GateType::Add(Fp128::one())).unwrap();
        builder.finalize_layer().unwrap();
        
        let circuit = builder.build().unwrap();
        
        // Create instance
        let inputs = vec![Fp128::from(3), Fp128::from(5)];
        let expected_output = Fp128::from(8);
        
        let instance = SumcheckInstance::new(
            circuit.clone(),
            1,
            expected_output,
        ).unwrap();
        
        // Generate proof
        let prover = ProverLayers::new(
            circuit.clone(),
            &inputs,
            1,
            SumcheckOptions::default(),
        ).unwrap();
        
        let proof = prover.prove(&instance, &mut OsRng).unwrap();
        
        // Verify
        let verifier = VerifierLayers::new(circuit);
        assert!(verifier.verify(&instance, &proof).unwrap());
    }
    
    #[test]
    fn test_degree_bound_check() {
        let proof = SumcheckProof {
            layer_proofs: vec![
                LayerProof {
                    copy_polys: vec![vec![Fp128::one(); 4]], // degree 3 - ok
                    hand_polys: vec![vec![Fp128::one(); 3]], // degree 2 - ok
                    wire_claims: vec![Fp128::one()],
                },
                LayerProof {
                    copy_polys: vec![vec![Fp128::one(); 5]], // degree 4 - too high!
                    hand_polys: vec![vec![Fp128::one(); 3]],
                    wire_claims: vec![Fp128::one()],
                },
            ],
            input_eval: vec![Fp128::one()],
        };
        
        assert!(!VerifierLayers::<Fp128>::check_degree_bounds(&proof));
    }
}