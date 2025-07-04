/// Ligero verifier implementation

use longfellow_algebra::traits::Field;
// use longfellow_algebra::fft::FFT;  // Currently unused
// use longfellow_algebra::polynomial::Polynomial;  // Currently unused
use longfellow_core::Result;
use std::collections::HashMap;

use crate::{
    LigeroInstance, LigeroProof,
    merkle::MerkleTree,
    transcript::{LigeroTranscript, compute_instance_digest},
    parameters::row_indices,
};

/// Ligero verifier
pub struct LigeroVerifier<F: Field> {
    instance: LigeroInstance<F>,
}

impl<F: Field> LigeroVerifier<F> {
    /// Create a new verifier
    pub fn new(instance: LigeroInstance<F>) -> Result<Self> {
        instance.params.validate()?;
        Ok(Self { instance })
    }
    
    /// Verify a proof
    pub fn verify(&self, proof: &LigeroProof<F>) -> Result<bool> {
        // Initialize transcript
        let instance_digest = compute_instance_digest(
            &self.instance.params,
            &self.instance.constraints,
        );
        let mut transcript = LigeroTranscript::new(&instance_digest);
        
        // Add column roots to transcript
        transcript.append_column_roots(&proof.column_roots);
        
        // Get challenges
        let ldt_challenges = transcript.challenge_ldt();
        transcript.append_ldt_response(&proof.ldt_responses);
        
        let linear_challenge = transcript.challenge_linear_combination(
            self.instance.constraints.linear_constraints.num_constraints
        );
        transcript.append_linear_response(&proof.linear_responses);
        
        let quad_challenge = transcript.challenge_linear_combination(
            self.instance.constraints.quadratic_constraints.constraints.len()
        );
        transcript.append_quadratic_response(&proof.quadratic_responses);
        
        let column_indices = transcript.challenge_column_indices(
            self.instance.params.block_enc_size(),
            self.instance.params.num_col_openings,
        );
        
        // Verify column openings
        if !self.verify_column_openings(proof, &column_indices)? {
            return Ok(false);
        }
        
        // Reconstruct opened columns
        let opened_columns = self.reconstruct_columns(proof)?;
        
        // Verify low-degree test
        if !self.verify_ldt(&opened_columns, &ldt_challenges, &proof.ldt_responses)? {
            return Ok(false);
        }
        
        // Verify linear constraints
        if !self.verify_linear_constraints(
            &opened_columns,
            &linear_challenge,
            &proof.linear_responses,
        )? {
            return Ok(false);
        }
        
        // Verify quadratic constraints
        if !self.verify_quadratic_constraints(
            &opened_columns,
            &quad_challenge,
            &proof.quadratic_responses,
        )? {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Verify column openings with Merkle proofs
    fn verify_column_openings(
        &self,
        proof: &LigeroProof<F>,
        expected_indices: &[usize],
    ) -> Result<bool> {
        if proof.column_openings.len() != expected_indices.len() {
            return Ok(false);
        }
        
        if proof.column_roots.len() != 1 {
            return Ok(false);
        }
        
        let root = &proof.column_roots[0];
        
        for (opening, &expected_idx) in proof.column_openings.iter().zip(expected_indices.iter()) {
            if opening.index != expected_idx {
                return Ok(false);
            }
            
            // Verify Merkle proof
            if !MerkleTree::verify(
                root,
                opening.index,
                &opening.values,
                &opening.merkle_proof,
            ) {
                return Ok(false);
            }
            
            // Verify column has correct height
            let expected_height = self.calculate_expected_height();
            if opening.values.len() != expected_height {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Reconstruct opened columns into a map
    fn reconstruct_columns(
        &self,
        proof: &LigeroProof<F>,
    ) -> Result<HashMap<usize, Vec<F>>> {
        let mut columns = HashMap::new();
        
        for opening in &proof.column_openings {
            columns.insert(opening.index, opening.values.clone());
        }
        
        Ok(columns)
    }
    
    /// Verify low-degree test
    fn verify_ldt(
        &self,
        opened_columns: &HashMap<usize, Vec<F>>,
        challenges: &[F],
        responses: &[Vec<F>],
    ) -> Result<bool> {
        // For each opened column, verify the low-degree test
        for (&col_idx, column) in opened_columns {
            // Extract relevant rows from the column
            let blinding_values: Vec<F> = (0..self.instance.params.num_blinding_rows)
                .map(|i| column[i])
                .collect();
            
            // Compute expected value from linear combination
            let mut expected = F::zero();
            for (_i, (&val, &challenge)) in blinding_values.iter().zip(challenges.iter()).enumerate() {
                expected += val * challenge;
            }
            
            // Compare with response at this column
            if responses.len() > 0 && col_idx < responses[0].len() {
                if expected != responses[0][col_idx] {
                    return Ok(false);
                }
            }
        }
        
        // Verify each response is low-degree
        for response in responses {
            if !self.is_low_degree(response)? {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Check if a row is low-degree
    fn is_low_degree(&self, row: &[F]) -> Result<bool> {
        let block_size = self.instance.params.block_size;
        
        if row.len() < block_size {
            return Ok(false);
        }
        
        // Extract the systematic part
        let systematic: Vec<_> = row[..block_size].to_vec();
        
        // Interpolate polynomial using evaluation points
        let eval_points: Vec<(F, F)> = (0..block_size)
            .map(|i| (F::from_u64(i as u64), systematic[i]))
            .collect();
        
        let poly = longfellow_algebra::interpolation::lagrange_interpolate(&eval_points)?;
        
        // Verify degree bound
        if let Some(degree) = poly.degree() {
            if degree >= block_size {
                return Ok(false);
            }
        }
        
        // Verify encoding matches
        for i in block_size..std::cmp::min(2 * block_size - 1, row.len()) {
            let point = F::from_u64(i as u64);
            let expected = poly.evaluate(&point);
            if row[i] != expected {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Verify linear constraints
    fn verify_linear_constraints(
        &self,
        opened_columns: &HashMap<usize, Vec<F>>,
        challenges: &[F],
        response: &[F],
    ) -> Result<bool> {
        // Compute expected response from opened witness values
        let mut computed_response = vec![F::zero(); self.instance.params.block_size];
        
        // For each opened column
        for (&col_idx, column) in opened_columns {
            if col_idx >= computed_response.len() {
                continue;
            }
            
            // Extract witness values from appropriate rows
            let witness_start = row_indices::WITNESS_START;
            let num_witness_blocks = self.instance.params.num_witness_blocks(
                self.instance.constraints.num_witnesses
            );
            
            for block_idx in 0..num_witness_blocks {
                let row_idx = witness_start + block_idx;
                if row_idx < column.len() {
                    let witness_idx = block_idx * self.instance.params.block_size + col_idx;
                    if witness_idx < self.instance.constraints.num_witnesses {
                        // Apply linear constraints
                        for (i, &challenge) in challenges.iter().enumerate() {
                            for &(row, col, ref coeff) in &self.instance.constraints.linear_constraints.matrix {
                                if row == i && col == witness_idx {
                                    computed_response[col_idx % self.instance.params.block_size] += 
                                        challenge * *coeff * column[row_idx];
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Compare with prover's response
        for (&col_idx, _) in opened_columns {
            if col_idx < response.len() && col_idx < computed_response.len() {
                if response[col_idx] != computed_response[col_idx] {
                    return Ok(false);
                }
            }
        }
        
        Ok(true)
    }
    
    /// Verify quadratic constraints
    fn verify_quadratic_constraints(
        &self,
        opened_columns: &HashMap<usize, Vec<F>>,
        challenges: &[F],
        response: &[F],
    ) -> Result<bool> {
        if self.instance.constraints.quadratic_constraints.constraints.is_empty() {
            return Ok(response.is_empty());
        }
        
        // For each opened column, verify quadratic constraint values
        let witness_blocks = self.instance.params.num_witness_blocks(
            self.instance.constraints.num_witnesses
        );
        let quad_start = row_indices::WITNESS_START + witness_blocks;
        
        for (&col_idx, column) in opened_columns {
            if col_idx >= response.len() {
                continue;
            }
            
            let mut expected = F::zero();
            
            // Sum up quadratic constraint contributions
            let num_quad_rows = self.instance.params.num_quadratic_rows(
                self.instance.constraints.quadratic_constraints.constraints.len()
            );
            
            for quad_row in 0..num_quad_rows {
                let row_idx = quad_start + quad_row;
                if row_idx < column.len() {
                    let constraint_start = quad_row * self.instance.params.block_size;
                    let constraint_end = std::cmp::min(
                        constraint_start + self.instance.params.block_size,
                        challenges.len()
                    );
                    
                    if col_idx < self.instance.params.block_size {
                        let local_idx = constraint_start + (col_idx % self.instance.params.block_size);
                        if local_idx < constraint_end {
                            expected += challenges[local_idx] * column[row_idx];
                        }
                    }
                }
            }
            
            if response[col_idx] != expected {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Calculate expected tableau height
    fn calculate_expected_height(&self) -> usize {
        let witness_blocks = self.instance.params.num_witness_blocks(
            self.instance.constraints.num_witnesses
        );
        let quad_rows = self.instance.params.num_quadratic_rows(
            self.instance.constraints.quadratic_constraints.constraints.len()
        );
        
        self.instance.params.tableau_height(witness_blocks, quad_rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LigeroParams, ConstraintSystem, prover::LigeroProver};
    use longfellow_algebra::Fp128;
    use rand::rngs::OsRng;
    
    #[test]
    fn test_verify_valid_proof() {
        // Create constraint system
        let mut cs = ConstraintSystem::<Fp128>::new(3);
        cs.add_linear_constraint(
            vec![(0, Fp128::one()), (1, Fp128::one()), (2, -Fp128::one())],
            Fp128::zero(),
        );
        cs.add_quadratic_constraint(0, 1, 2);
        
        // Create instance
        let params = LigeroParams::security_80();
        let instance = LigeroInstance::new(params, cs).unwrap();
        
        // Generate proof
        let prover = LigeroProver::new(instance.clone()).unwrap();
        let witness = vec![Fp128::from(2), Fp128::from(3), Fp128::from(6)];
        let proof = prover.prove(&witness, &mut OsRng).unwrap();
        
        // Verify proof
        let verifier = LigeroVerifier::new(instance).unwrap();
        assert!(verifier.verify(&proof).unwrap());
    }
    
    #[test]
    fn test_reject_invalid_proof() {
        // Create constraint system
        let mut cs = ConstraintSystem::<Fp128>::new(3);
        cs.add_linear_constraint(
            vec![(0, Fp128::one()), (1, Fp128::one()), (2, -Fp128::one())],
            Fp128::zero(),
        );
        
        let params = LigeroParams::security_80();
        let instance = LigeroInstance::new(params, cs).unwrap();
        
        // Generate valid proof
        let prover = LigeroProver::new(instance.clone()).unwrap();
        let witness = vec![Fp128::from(2), Fp128::from(3), Fp128::from(5)];
        let mut proof = prover.prove(&witness, &mut OsRng).unwrap();
        
        // Corrupt the proof
        if !proof.linear_responses.is_empty() {
            proof.linear_responses[0] += Fp128::one();
        }
        
        // Verify should fail
        let verifier = LigeroVerifier::new(instance).unwrap();
        assert!(!verifier.verify(&proof).unwrap());
    }
}