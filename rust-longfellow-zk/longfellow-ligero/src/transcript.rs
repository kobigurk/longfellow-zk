/// Ligero protocol transcript for Fiat-Shamir transform

use longfellow_algebra::traits::Field;
use longfellow_random::Transcript;
use sha3::{Digest, Sha3_256};

/// Ligero-specific transcript
pub struct LigeroTranscript {
    /// Base transcript
    base: Transcript,
}

impl LigeroTranscript {
    /// Create a new Ligero transcript
    pub fn new(instance_digest: &[u8]) -> Self {
        let mut base = Transcript::new(b"Ligero-v1");
        base.append_message(b"instance", instance_digest);
        
        Self { base }
    }
    
    /// Append column roots
    pub fn append_column_roots(&mut self, roots: &[[u8; 32]]) {
        self.base.append_message(b"num_roots", &(roots.len() as u64).to_le_bytes());
        for (i, root) in roots.iter().enumerate() {
            let mut label = b"root".to_vec();
            label.extend_from_slice(&(i as u64).to_le_bytes());
            self.base.append_message(&label, root);
        }
    }
    
    /// Get challenge for low-degree test
    pub fn challenge_ldt<F: Field>(&mut self) -> Vec<F> {
        // For Ligero, we need multiple challenges for the linear combination
        self.base.challenge_scalars(b"ldt", 3)
    }
    
    /// Get random linear combination coefficients
    pub fn challenge_linear_combination<F: Field>(&mut self, num_coeffs: usize) -> Vec<F> {
        self.base.challenge_scalars(b"linear_comb", num_coeffs)
    }
    
    /// Get column indices to open
    pub fn challenge_column_indices(&mut self, num_columns: usize, num_openings: usize) -> Vec<usize> {
        let mut indices = Vec::with_capacity(num_openings);
        let mut seen = std::collections::HashSet::new();
        
        let mut counter = 0u64;
        while indices.len() < num_openings {
            let mut label = b"col_index".to_vec();
            label.extend_from_slice(&counter.to_le_bytes());
            
            let hash = self.base.challenge_scalar::<longfellow_algebra::Fp128>(&label);
            let index = (hash.to_u64() as usize) % num_columns;
            
            if seen.insert(index) {
                indices.push(index);
            }
            counter += 1;
        }
        
        indices
    }
    
    /// Append prover messages
    pub fn append_ldt_response<F: Field>(&mut self, responses: &[Vec<F>]) {
        for (i, response) in responses.iter().enumerate() {
            let mut label = b"ldt_response".to_vec();
            label.extend_from_slice(&(i as u64).to_le_bytes());
            self.base.append_field_elements(&label, response);
        }
    }
    
    /// Append linear test response
    pub fn append_linear_response<F: Field>(&mut self, response: &[F]) {
        self.base.append_field_elements(b"linear_response", response);
    }
    
    /// Append quadratic test response
    pub fn append_quadratic_response<F: Field>(&mut self, response: &[F]) {
        self.base.append_field_elements(b"quadratic_response", response);
    }
    
    /// Get the final transcript hash
    pub fn finalize(self) -> [u8; 32] {
        self.base.finalize()
    }
}

/// Compute instance digest for deterministic transcript initialization
pub fn compute_instance_digest<F: Field>(
    params: &crate::LigeroParams,
    constraints: &crate::ConstraintSystem<F>,
) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    
    // Hash parameters
    hasher.update(b"LigeroInstance");
    hasher.update(&(params.block_size as u64).to_le_bytes());
    hasher.update(&(params.extension_factor as u64).to_le_bytes());
    hasher.update(&(params.num_blinding_rows as u64).to_le_bytes());
    hasher.update(&(params.num_col_openings as u64).to_le_bytes());
    hasher.update(&(params.security_bits as u64).to_le_bytes());
    
    // Hash constraint system dimensions
    hasher.update(&(constraints.num_witnesses as u64).to_le_bytes());
    hasher.update(&(constraints.linear_constraints.num_constraints as u64).to_le_bytes());
    hasher.update(&(constraints.quadratic_constraints.constraints.len() as u64).to_le_bytes());
    
    // Hash linear constraint matrix (just the structure, not values for efficiency)
    hasher.update(&(constraints.linear_constraints.matrix.len() as u64).to_le_bytes());
    
    // Hash quadratic constraints
    for &(x, y, z) in &constraints.quadratic_constraints.constraints {
        hasher.update(&(x as u64).to_le_bytes());
        hasher.update(&(y as u64).to_le_bytes());
        hasher.update(&(z as u64).to_le_bytes());
    }
    
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LigeroParams, ConstraintSystem};
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_transcript_deterministic() {
        let params = LigeroParams::security_128();
        let cs = ConstraintSystem::<Fp128>::new(100);
        let digest = compute_instance_digest(&params, &cs);
        
        let mut t1 = LigeroTranscript::new(&digest);
        let mut t2 = LigeroTranscript::new(&digest);
        
        // Append same messages
        let roots = vec![[1u8; 32], [2u8; 32]];
        t1.append_column_roots(&roots);
        t2.append_column_roots(&roots);
        
        // Get challenges
        let c1: Vec<Fp128> = t1.challenge_ldt();
        let c2: Vec<Fp128> = t2.challenge_ldt();
        
        assert_eq!(c1, c2);
    }
    
    #[test]
    fn test_column_index_selection() {
        let params = LigeroParams::security_128();
        let cs = ConstraintSystem::<Fp128>::new(100);
        let digest = compute_instance_digest(&params, &cs);
        
        let mut transcript = LigeroTranscript::new(&digest);
        
        let indices = transcript.challenge_column_indices(1000, 50);
        assert_eq!(indices.len(), 50);
        
        // Check all indices are valid
        for &idx in &indices {
            assert!(idx < 1000);
        }
        
        // Check no duplicates
        let unique: std::collections::HashSet<_> = indices.iter().collect();
        assert_eq!(unique.len(), indices.len());
    }
}