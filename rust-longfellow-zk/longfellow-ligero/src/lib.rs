/// Ligero: Lightweight Sublinear Arguments Without a Trusted Setup
/// 
/// This module implements the Ligero proof system for proving satisfiability
/// of arithmetic circuits with linear and quadratic constraints.

pub mod tableau;
pub mod prover;
pub mod verifier;
pub mod transcript;
pub mod merkle;
pub mod parameters;

use longfellow_algebra::traits::Field;
use longfellow_core::{LongfellowError, Result};
use serde::{Deserialize, Serialize};

pub use prover::LigeroProver;
pub use verifier::LigeroVerifier;
pub use transcript::LigeroTranscript;
pub use parameters::LigeroParams;

/// Ligero proof structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LigeroProof<F: Field> {
    /// Merkle root of column commitments
    pub column_roots: Vec<[u8; 32]>,
    
    /// Low-degree test responses
    pub ldt_responses: Vec<Vec<F>>,
    
    /// Linear test responses
    pub linear_responses: Vec<F>,
    
    /// Quadratic test responses
    pub quadratic_responses: Vec<F>,
    
    /// Column opening proofs
    pub column_openings: Vec<ColumnOpening<F>>,
}

/// Column opening with Merkle proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColumnOpening<F: Field> {
    /// Column index
    pub index: usize,
    
    /// Column values
    pub values: Vec<F>,
    
    /// Merkle proof
    pub merkle_proof: Vec<[u8; 32]>,
}

/// Constraint system for Ligero
#[derive(Clone, Debug)]
pub struct ConstraintSystem<F: Field> {
    /// Number of witness variables
    pub num_witnesses: usize,
    
    /// Linear constraints: A * w = b
    pub linear_constraints: LinearConstraints<F>,
    
    /// Quadratic constraints: w[x] * w[y] = w[z]
    pub quadratic_constraints: QuadraticConstraints,
}

/// Linear constraints: A * w = b
#[derive(Clone, Debug)]
pub struct LinearConstraints<F: Field> {
    /// Constraint matrix A (sparse representation)
    pub matrix: Vec<(usize, usize, F)>, // (row, col, value)
    
    /// Right-hand side vector b
    pub rhs: Vec<F>,
    
    /// Number of constraints
    pub num_constraints: usize,
}

/// Quadratic constraints: w[x] * w[y] = w[z]
#[derive(Clone, Debug)]
pub struct QuadraticConstraints {
    /// Triples (x, y, z) where w[x] * w[y] = w[z]
    pub constraints: Vec<(usize, usize, usize)>,
}

impl<F: Field> ConstraintSystem<F> {
    pub fn new(num_witnesses: usize) -> Self {
        Self {
            num_witnesses,
            linear_constraints: LinearConstraints {
                matrix: Vec::new(),
                rhs: Vec::new(),
                num_constraints: 0,
            },
            quadratic_constraints: QuadraticConstraints {
                constraints: Vec::new(),
            },
        }
    }
    
    /// Add a linear constraint
    pub fn add_linear_constraint(&mut self, row: Vec<(usize, F)>, rhs: F) {
        for (col, value) in row {
            self.linear_constraints.matrix.push((
                self.linear_constraints.num_constraints,
                col,
                value,
            ));
        }
        self.linear_constraints.rhs.push(rhs);
        self.linear_constraints.num_constraints += 1;
    }
    
    /// Add a quadratic constraint: w[x] * w[y] = w[z]
    pub fn add_quadratic_constraint(&mut self, x: usize, y: usize, z: usize) {
        assert!(x < self.num_witnesses);
        assert!(y < self.num_witnesses);
        assert!(z < self.num_witnesses);
        self.quadratic_constraints.constraints.push((x, y, z));
    }
    
    /// Check if a witness satisfies all constraints
    pub fn is_satisfied(&self, witness: &[F]) -> Result<bool> {
        if witness.len() != self.num_witnesses {
            return Err(LongfellowError::InvalidParameter(
                format!("Expected {} witnesses, got {}", self.num_witnesses, witness.len())
            ));
        }
        
        // Check linear constraints
        for i in 0..self.linear_constraints.num_constraints {
            let mut sum = F::zero();
            for &(row, col, ref value) in &self.linear_constraints.matrix {
                if row == i {
                    sum += *value * witness[col];
                }
            }
            if sum != self.linear_constraints.rhs[i] {
                return Ok(false);
            }
        }
        
        // Check quadratic constraints
        for &(x, y, z) in &self.quadratic_constraints.constraints {
            if witness[x] * witness[y] != witness[z] {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

/// Ligero instance combining parameters and constraint system
pub struct LigeroInstance<F: Field> {
    pub params: LigeroParams,
    pub constraints: ConstraintSystem<F>,
}

impl<F: Field> LigeroInstance<F> {
    pub fn new(params: LigeroParams, constraints: ConstraintSystem<F>) -> Result<Self> {
        // Validate parameters match constraint system
        if constraints.num_witnesses > params.max_witnesses() {
            return Err(LongfellowError::InvalidParameter(
                format!("Too many witnesses: {} > {}", 
                    constraints.num_witnesses, 
                    params.max_witnesses())
            ));
        }
        
        Ok(Self { params, constraints })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_constraint_system() {
        let mut cs = ConstraintSystem::<Fp128>::new(4);
        
        // Add linear constraint: w[0] + 2*w[1] = 3
        cs.add_linear_constraint(
            vec![(0, Fp128::one()), (1, Fp128::from(2))],
            Fp128::from(3),
        );
        
        // Add quadratic constraint: w[0] * w[1] = w[2]
        cs.add_quadratic_constraint(0, 1, 2);
        
        // Test satisfying witness: w = [1, 1, 1, 0]
        let witness = vec![
            Fp128::one(),
            Fp128::one(),
            Fp128::one(),
            Fp128::zero(),
        ];
        
        assert!(cs.is_satisfied(&witness).unwrap());
        
        // Test non-satisfying witness
        let bad_witness = vec![
            Fp128::one(),
            Fp128::from(2),
            Fp128::one(), // Should be 2
            Fp128::zero(),
        ];
        
        assert!(!cs.is_satisfied(&bad_witness).unwrap());
    }
}