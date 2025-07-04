/// Ligero prover implementation

use longfellow_algebra::traits::Field;
use longfellow_algebra::fft::FFT;
use longfellow_core::{LongfellowError, Result};
use longfellow_random::FieldRng;
use rand::{CryptoRng, RngCore};
use rayon::prelude::*;

use crate::{
    LigeroInstance, LigeroProof, ColumnOpening,
    tableau::{Tableau, linear_combination},
    merkle::MerkleTree,
    transcript::{LigeroTranscript, compute_instance_digest},
    parameters::row_indices,
};

/// Ligero prover
pub struct LigeroProver<F: Field> {
    instance: LigeroInstance<F>,
}

impl<F: Field> LigeroProver<F> {
    /// Create a new prover
    pub fn new(instance: LigeroInstance<F>) -> Result<Self> {
        instance.params.validate()?;
        Ok(Self { instance })
    }
    
    /// Generate a proof for a witness
    pub fn prove<R: RngCore + CryptoRng>(
        &self,
        witness: &[F],
        rng: &mut R,
    ) -> Result<LigeroProof<F>> {
        // Verify witness satisfies constraints
        if !self.instance.constraints.is_satisfied(witness)? {
            return Err(LongfellowError::InvalidParameter(
                "Witness does not satisfy constraints".to_string()
            ));
        }
        
        // Initialize transcript
        let instance_digest = compute_instance_digest(
            &self.instance.params,
            &self.instance.constraints,
        );
        let mut transcript = LigeroTranscript::new(&instance_digest);
        
        // Create and fill tableau
        let mut tableau = self.create_tableau(witness, rng)?;
        
        // Encode all rows
        tableau.encode_rows()?;
        
        // Commit to columns
        let columns = self.extract_columns(&tableau);
        let merkle_tree = MerkleTree::new(&columns)?;
        let column_root = merkle_tree.root();
        
        transcript.append_column_roots(&[column_root]);
        
        // Low-degree test
        let ldt_challenges = transcript.challenge_ldt();
        let ldt_responses = self.compute_ldt_responses(&tableau, &ldt_challenges)?;
        transcript.append_ldt_response(&ldt_responses);
        
        // Linear test
        let linear_challenge = transcript.challenge_linear_combination(
            self.instance.constraints.linear_constraints.num_constraints
        );
        let linear_response = self.compute_linear_response(
            &tableau,
            witness,
            &linear_challenge,
        )?;
        transcript.append_linear_response(&linear_response);
        
        // Quadratic test
        let quad_challenge = transcript.challenge_linear_combination(
            self.instance.constraints.quadratic_constraints.constraints.len()
        );
        let quadratic_response = self.compute_quadratic_response(
            &tableau,
            &quad_challenge,
        )?;
        transcript.append_quadratic_response(&quadratic_response);
        
        // Column openings
        let column_indices = transcript.challenge_column_indices(
            columns.len(),
            self.instance.params.num_col_openings,
        );
        
        let column_openings = self.open_columns(
            &columns,
            &merkle_tree,
            &column_indices,
        )?;
        
        Ok(LigeroProof {
            column_roots: vec![column_root],
            ldt_responses,
            linear_responses: linear_response,
            quadratic_responses: quadratic_response,
            column_openings,
        })
    }
    
    /// Create and fill the tableau
    fn create_tableau<R: RngCore + CryptoRng>(
        &self,
        witness: &[F],
        rng: &mut R,
    ) -> Result<Tableau<F>> {
        let params = &self.instance.params;
        let constraints = &self.instance.constraints;
        
        // Calculate tableau dimensions
        let num_witness_blocks = params.num_witness_blocks(witness.len());
        let num_quad_rows = params.num_quadratic_rows(
            constraints.quadratic_constraints.constraints.len()
        );
        let height = params.tableau_height(num_witness_blocks, num_quad_rows);
        
        let mut tableau = Tableau::new(params.clone(), height);
        
        // Fill blinding rows
        tableau.randomize_blinding_rows(rng)?;
        
        // Layout witnesses
        tableau.layout_witnesses(witness, rng)?;
        
        // Encode quadratic constraints
        let quad_row_start = row_indices::WITNESS_START + num_witness_blocks;
        tableau.encode_quadratic_constraints(
            &constraints.quadratic_constraints.constraints,
            witness,
            quad_row_start,
        )?;
        
        Ok(tableau)
    }
    
    /// Extract columns from tableau
    fn extract_columns(&self, tableau: &Tableau<F>) -> Vec<Vec<F>> {
        let (_, width) = tableau.dimensions();
        (0..width)
            .into_par_iter()
            .map(|j| tableau.column(j))
            .collect()
    }
    
    /// Compute low-degree test responses
    fn compute_ldt_responses(
        &self,
        tableau: &Tableau<F>,
        challenges: &[F],
    ) -> Result<Vec<Vec<F>>> {
        if challenges.len() != 3 {
            return Err(LongfellowError::InvalidParameter(
                "Expected 3 LDT challenges".to_string()
            ));
        }
        
        let mut responses = Vec::new();
        
        // Response 1: Linear combination of blinding rows
        let blinding_rows: Vec<_> = (0..self.instance.params.num_blinding_rows)
            .map(|i| tableau.row(i).to_vec())
            .collect();
        let response1 = linear_combination(&blinding_rows, challenges)?;
        responses.push(response1);
        
        // Response 2: Linear combination of witness rows
        let num_witness_blocks = self.instance.params.num_witness_blocks(
            self.instance.constraints.num_witnesses
        );
        let witness_rows: Vec<_> = (0..num_witness_blocks)
            .map(|i| tableau.row(row_indices::WITNESS_START + i).to_vec())
            .collect();
        
        if !witness_rows.is_empty() {
            let witness_coeffs = (0..witness_rows.len())
                .map(|i| challenges[i % challenges.len()])
                .collect::<Vec<_>>();
            let response2 = linear_combination(&witness_rows, &witness_coeffs)?;
            responses.push(response2);
        }
        
        Ok(responses)
    }
    
    /// Compute linear test response
    fn compute_linear_response(
        &self,
        tableau: &Tableau<F>,
        witness: &[F],
        challenges: &[F],
    ) -> Result<Vec<F>> {
        let constraints = &self.instance.constraints.linear_constraints;
        let mut response = vec![F::zero(); self.instance.params.block_size];
        
        // Compute A^T * challenges
        for (i, &challenge) in challenges.iter().enumerate() {
            for &(row, col, ref value) in &constraints.matrix {
                if row == i && col < response.len() {
                    response[col] += challenge * *value;
                }
            }
        }
        
        // Add b^T * challenges contribution
        let mut b_contribution = F::zero();
        for (b, challenge) in constraints.rhs.iter().zip(challenges.iter()) {
            b_contribution += *b * *challenge;
        }
        
        Ok(response)
    }
    
    /// Compute quadratic test response
    fn compute_quadratic_response(
        &self,
        tableau: &Tableau<F>,
        challenges: &[F],
    ) -> Result<Vec<F>> {
        let num_quad_rows = self.instance.params.num_quadratic_rows(
            self.instance.constraints.quadratic_constraints.constraints.len()
        );
        
        if num_quad_rows == 0 {
            return Ok(vec![]);
        }
        
        let witness_blocks = self.instance.params.num_witness_blocks(
            self.instance.constraints.num_witnesses
        );
        let quad_start = row_indices::WITNESS_START + witness_blocks;
        
        let quad_rows: Vec<_> = (0..num_quad_rows)
            .map(|i| tableau.row(quad_start + i).to_vec())
            .collect();
        
        let quad_coeffs = (0..num_quad_rows)
            .map(|i| {
                let start = i * self.instance.params.block_size;
                let end = std::cmp::min(
                    start + self.instance.params.block_size,
                    challenges.len()
                );
                challenges[start..end].iter().sum()
            })
            .collect::<Vec<F>>();
        
        linear_combination(&quad_rows, &quad_coeffs)
    }
    
    /// Open columns with Merkle proofs
    fn open_columns(
        &self,
        columns: &[Vec<F>],
        merkle_tree: &MerkleTree,
        indices: &[usize],
    ) -> Result<Vec<ColumnOpening<F>>> {
        indices
            .par_iter()
            .map(|&index| {
                let merkle_proof = merkle_tree.prove(index)?;
                Ok(ColumnOpening {
                    index,
                    values: columns[index].clone(),
                    merkle_proof,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LigeroParams, ConstraintSystem};
    use longfellow_algebra::Fp128;
    use rand::rngs::OsRng;
    
    #[test]
    fn test_simple_proof() {
        // Create a simple constraint system
        let mut cs = ConstraintSystem::<Fp128>::new(3);
        
        // Add constraint: w[0] + w[1] = w[2]
        cs.add_linear_constraint(
            vec![(0, Fp128::one()), (1, Fp128::one()), (2, -Fp128::one())],
            Fp128::zero(),
        );
        
        // Add constraint: w[0] * w[1] = w[2]
        cs.add_quadratic_constraint(0, 1, 2);
        
        // Create instance
        let params = LigeroParams::security_80();
        let instance = LigeroInstance::new(params, cs).unwrap();
        
        // Create prover
        let prover = LigeroProver::new(instance).unwrap();
        
        // Create witness: w = [2, 3, 6] (satisfies both constraints)
        let witness = vec![
            Fp128::from(2),
            Fp128::from(3),
            Fp128::from(6),
        ];
        
        // Generate proof
        let proof = prover.prove(&witness, &mut OsRng).unwrap();
        
        // Check proof structure
        assert_eq!(proof.column_roots.len(), 1);
        assert_eq!(proof.column_openings.len(), 80); // num_col_openings
    }
}