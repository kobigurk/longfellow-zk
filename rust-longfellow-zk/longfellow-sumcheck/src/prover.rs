/// Sumcheck prover implementation

use longfellow_algebra::traits::Field;
use longfellow_arrays::dense::DenseArray;
use longfellow_core::{LongfellowError, Result};
use longfellow_random::FieldRng;
use rand::{CryptoRng, RngCore};
use rayon::prelude::*;

use crate::{
    SumcheckInstance, SumcheckProof, LayerProof, SumcheckOptions,
    circuit::{Circuit, Layer},
    polynomial::{UnivariatePoly, MultilinearPoly, PolyHelper},
    transcript::SumcheckTranscript,
};

/// Sumcheck prover for a single layer
pub struct Prover<F: Field> {
    /// Wire values for this layer
    wires: DenseArray<F>,
    /// Number of copies
    num_copies: usize,
    /// Options
    options: SumcheckOptions,
}

impl<F: Field> Prover<F> {
    /// Create a new prover
    pub fn new(
        wires: DenseArray<F>,
        num_copies: usize,
        options: SumcheckOptions,
    ) -> Self {
        Self {
            wires,
            num_copies,
            options,
        }
    }
    
    /// Prove sumcheck for a layer
    pub fn prove_layer<R: RngCore + CryptoRng>(
        &self,
        layer: &Layer<F>,
        claim: F,
        transcript: &mut SumcheckTranscript,
        rng: &mut R,
    ) -> Result<LayerProof<F>> {
        let mut copy_polys = Vec::new();
        let mut hand_polys = Vec::new();
        let mut current_claim = claim;
        
        // Bind copy variables first
        let copy_vars = self.num_copy_vars();
        let mut copy_bindings = Vec::new();
        
        for round in 0..copy_vars {
            let poly = self.compute_copy_poly(layer, round, &copy_bindings)?;
            
            // Verify sum
            let sum = (0..=poly.degree())
                .map(|i| poly.evaluate(F::from(i as u64)))
                .sum::<F>();
            
            if sum != current_claim {
                return Err(LongfellowError::ProofError(
                    format!("Copy poly sum mismatch: {} != {}", sum, current_claim)
                ));
            }
            
            transcript.append_polynomial(round, &poly);
            let challenge = transcript.challenge_binding::<F>(round);
            
            copy_bindings.push(challenge);
            current_claim = poly.evaluate(challenge);
            copy_polys.push(poly.coeffs);
        }
        
        // Bind hand variables
        let hand_vars = layer.nin;
        let mut hand_bindings = Vec::new();
        
        for round in 0..hand_vars * 2 {
            let is_left = round < hand_vars;
            let var_idx = if is_left { round } else { round - hand_vars };
            
            let poly = self.compute_hand_poly(
                layer,
                var_idx,
                is_left,
                &copy_bindings,
                &hand_bindings,
            )?;
            
            // Verify sum
            let sum = poly.evaluate(F::zero()) + poly.evaluate(F::one());
            if sum != current_claim {
                return Err(LongfellowError::ProofError(
                    "Hand poly sum mismatch".to_string()
                ));
            }
            
            transcript.append_polynomial(copy_vars + round, &poly);
            let challenge = transcript.challenge_binding::<F>(copy_vars + round);
            
            hand_bindings.push(challenge);
            current_claim = poly.evaluate(challenge);
            hand_polys.push(poly.coeffs);
        }
        
        // Compute wire claims
        let wire_claims = self.compute_wire_claims(&copy_bindings, &hand_bindings)?;
        transcript.append_wire_claims(0, &wire_claims);
        
        Ok(LayerProof {
            copy_polys,
            hand_polys,
            wire_claims,
        })
    }
    
    /// Get number of copy variables
    fn num_copy_vars(&self) -> usize {
        if self.num_copies <= 1 {
            0
        } else {
            (self.num_copies - 1).next_power_of_two().trailing_zeros() as usize
        }
    }
    
    /// Compute polynomial for copy variable binding
    fn compute_copy_poly(
        &self,
        layer: &Layer<F>,
        round: usize,
        bindings: &[F],
    ) -> Result<UnivariatePoly<F>> {
        // For degree-3 polynomial, evaluate at 0, 1, 2, 3
        let mut evals = vec![F::zero(); 4];
        
        for eval_point in 0..4 {
            let point_val = F::from(eval_point as u64);
            let mut sum = F::zero();
            
            // Sum over all unbound copy indices
            let unbound_copies = 1 << (self.num_copy_vars() - bindings.len());
            
            for copy_idx in 0..unbound_copies {
                // Compute full copy index with bindings
                let mut full_idx = 0;
                let mut bit_pos = 0;
                
                for (i, &binding) in bindings.iter().enumerate() {
                    if binding == F::one() {
                        full_idx |= 1 << i;
                    }
                }
                
                // Add current round bit
                if round < self.num_copy_vars() {
                    if point_val == F::one() || point_val == F::from(3) {
                        full_idx |= 1 << round;
                    }
                }
                
                // Add remaining unbound bits
                for i in 0..self.num_copy_vars() - bindings.len() - 1 {
                    if (copy_idx >> i) & 1 == 1 {
                        full_idx |= 1 << (bindings.len() + 1 + i);
                    }
                }
                
                if full_idx < self.num_copies {
                    // Evaluate layer at this copy
                    let copy_sum = self.evaluate_layer_copy(layer, full_idx)?;
                    sum += copy_sum;
                }
            }
            
            evals[eval_point] = sum;
        }
        
        UnivariatePoly::interpolate(&evals)
    }
    
    /// Compute polynomial for hand variable binding
    fn compute_hand_poly(
        &self,
        layer: &Layer<F>,
        var_idx: usize,
        is_left: bool,
        copy_bindings: &[F],
        hand_bindings: &[F],
    ) -> Result<UnivariatePoly<F>> {
        // For degree-2 polynomial, evaluate at 0, 1, 2
        let mut evals = vec![F::zero(); 3];
        
        for eval_point in 0..3 {
            let point_val = F::from(eval_point as u64);
            
            // Evaluate quadratic form with partial bindings
            let mut quad = layer.quad.clone();
            
            // Apply existing hand bindings
            for (i, &binding) in hand_bindings.iter().enumerate() {
                if i < layer.nin {
                    quad = quad.bind_hand(i, binding, true)?;
                } else {
                    quad = quad.bind_hand(i - layer.nin, binding, false)?;
                }
            }
            
            // Apply current variable binding
            quad = quad.bind_hand(var_idx, point_val, is_left)?;
            
            // Sum over remaining variables
            evals[eval_point] = self.sum_quad_with_bindings(&quad, copy_bindings)?;
        }
        
        UnivariatePoly::interpolate(&evals)
    }
    
    /// Evaluate layer for a specific copy
    fn evaluate_layer_copy(&self, layer: &Layer<F>, copy_idx: usize) -> Result<F> {
        let offset = copy_idx * (1 << layer.nin);
        let mut sum = F::zero();
        
        // Sum over all gate evaluations
        for (g, h0, h1, coeff) in layer.quad.iter() {
            let left_val = if h0 == 0 {
                F::one()
            } else {
                self.wires.get(offset + h0 - 1)?
            };
            
            let right_val = if h1 == 0 {
                F::one()
            } else {
                self.wires.get(offset + h1 - 1)?
            };
            
            sum += coeff * left_val * right_val;
        }
        
        Ok(sum)
    }
    
    /// Sum quadratic form with partial bindings
    fn sum_quad_with_bindings(
        &self,
        quad: &crate::quad::Quad<F>,
        copy_bindings: &[F],
    ) -> Result<F> {
        // This is a placeholder - in practice would sum over remaining variables
        Ok(F::zero())
    }
    
    /// Compute wire claims after all bindings
    fn compute_wire_claims(
        &self,
        copy_bindings: &[F],
        hand_bindings: &[F],
    ) -> Result<Vec<F>> {
        // Evaluate wires at binding point
        let mut claims = Vec::new();
        
        // Determine which copy we're looking at based on bindings
        let mut copy_idx = 0;
        for (i, &b) in copy_bindings.iter().enumerate() {
            if b == F::one() {
                copy_idx |= 1 << i;
            }
        }
        
        if copy_idx < self.num_copies {
            let offset = copy_idx * self.wires.len() / self.num_copies;
            
            // Get wire values at hand binding points
            for i in 0..2 {
                let mut wire_idx = 0;
                for (j, &b) in hand_bindings[i * hand_bindings.len() / 2..(i + 1) * hand_bindings.len() / 2].iter().enumerate() {
                    if b == F::one() {
                        wire_idx |= 1 << j;
                    }
                }
                
                if offset + wire_idx < self.wires.len() {
                    claims.push(self.wires.get(offset + wire_idx)?);
                }
            }
        }
        
        Ok(claims)
    }
}

/// Sumcheck prover for entire circuit
pub struct ProverLayers<F: Field> {
    /// The circuit
    circuit: Circuit<F>,
    /// Wire values for all layers
    all_wires: Vec<DenseArray<F>>,
    /// Number of copies
    num_copies: usize,
    /// Options
    options: SumcheckOptions,
}

impl<F: Field> ProverLayers<F> {
    /// Create a new layered prover
    pub fn new(
        circuit: Circuit<F>,
        inputs: &[F],
        num_copies: usize,
        options: SumcheckOptions,
    ) -> Result<Self> {
        // Evaluate circuit to get all wire values
        let all_wires = Self::evaluate_all_layers(&circuit, inputs, num_copies)?;
        
        Ok(Self {
            circuit,
            all_wires,
            num_copies,
            options,
        })
    }
    
    /// Generate complete sumcheck proof
    pub fn prove<R: RngCore + CryptoRng>(
        &self,
        instance: &SumcheckInstance<F>,
        rng: &mut R,
    ) -> Result<SumcheckProof<F>> {
        let mut transcript = SumcheckTranscript::new(b"sumcheck");
        transcript.append_circuit_info(
            self.circuit.layers.len(),
            self.num_copies,
            &instance.claimed_sum.to_bytes_le(),
        );
        
        let mut layer_proofs = Vec::new();
        let mut current_claim = instance.claimed_sum;
        
        // Process each layer from output to input
        for (layer_idx, layer) in self.circuit.layers.iter().enumerate() {
            let wires = &self.all_wires[layer_idx];
            let prover = Prover::new(wires.clone(), self.num_copies, self.options.clone());
            
            let layer_proof = prover.prove_layer(
                layer,
                current_claim,
                &mut transcript,
                rng,
            )?;
            
            // Update claim for next layer
            current_claim = layer_proof.wire_claims.iter().sum();
            layer_proofs.push(layer_proof);
        }
        
        // Final input evaluation
        let input_eval = self.evaluate_inputs_at_binding(&transcript)?;
        
        Ok(SumcheckProof {
            layer_proofs,
            input_eval,
        })
    }
    
    /// Evaluate all layers of the circuit
    fn evaluate_all_layers(
        circuit: &Circuit<F>,
        inputs: &[F],
        num_copies: usize,
    ) -> Result<Vec<DenseArray<F>>> {
        let mut all_wires = Vec::new();
        let mut current = DenseArray::from_vec(inputs.to_vec());
        
        // Process layers in reverse (input to output)
        for layer in circuit.layers.iter().rev() {
            let next_size = (1 << layer.nout) * num_copies;
            let mut next_wires = vec![F::zero(); next_size];
            
            // Evaluate layer
            for copy in 0..num_copies {
                let in_offset = copy * (1 << layer.nin);
                let out_offset = copy * (1 << layer.nout);
                
                for (g, h0, h1, coeff) in layer.quad.iter() {
                    let left = if h0 == 0 {
                        F::one()
                    } else {
                        current.get(in_offset + h0 - 1)?
                    };
                    
                    let right = if h1 == 0 {
                        F::one()
                    } else {
                        current.get(in_offset + h1 - 1)?
                    };
                    
                    next_wires[out_offset + g] += coeff * left * right;
                }
            }
            
            current = DenseArray::from_vec(next_wires);
            all_wires.push(current.clone());
        }
        
        all_wires.reverse(); // Back to output-to-input order
        Ok(all_wires)
    }
    
    /// Evaluate inputs at the binding point from transcript
    fn evaluate_inputs_at_binding(
        &self,
        transcript: &SumcheckTranscript,
    ) -> Result<Vec<F>> {
        // This would extract bindings from transcript and evaluate
        // For now, return placeholder
        Ok(vec![F::zero()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::{CircuitBuilder, GateType};
    use longfellow_algebra::Fp128;
    use rand::rngs::OsRng;
    
    #[test]
    fn test_simple_layer_proof() {
        // Create a simple layer: output = input[0] + input[1]
        let mut layer = Layer::<Fp128>::new(0, 1, 0); // 1 output, 2 inputs
        layer.add_gate(0, 0, 1, GateType::Add(Fp128::one())).unwrap();
        
        // Create wire values
        let wires = DenseArray::from_vec(vec![Fp128::from(3), Fp128::from(5)]);
        let prover = Prover::new(wires, 1, SumcheckOptions::default());
        
        // Expected claim: 3 + 5 = 8
        let claim = Fp128::from(8);
        let mut transcript = SumcheckTranscript::new(b"test");
        
        let proof = prover.prove_layer(&layer, claim, &mut transcript, &mut OsRng).unwrap();
        
        assert!(!proof.hand_polys.is_empty());
        assert!(!proof.wire_claims.is_empty());
    }
}