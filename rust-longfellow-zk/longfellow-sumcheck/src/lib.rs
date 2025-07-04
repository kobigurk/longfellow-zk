/// Sumcheck protocol implementation for arithmetic circuits
/// 
/// This module implements the sumcheck protocol optimized for layered arithmetic circuits
/// with support for parallel evaluation of multiple circuit copies.

pub mod circuit;
pub mod quad;
pub mod prover;
pub mod verifier;
pub mod transcript;
pub mod polynomial;

use longfellow_algebra::traits::Field;
use longfellow_core::{LongfellowError, Result};
use serde::{Deserialize, Serialize};

pub use circuit::{Circuit, Layer};
pub use quad::{Quad, QuadCorner};
pub use prover::{Prover, ProverLayers};
pub use verifier::{Verifier, VerifierLayers};
pub use transcript::SumcheckTranscript;
pub use polynomial::{UnivariatePoly, MultilinearPoly};

/// Maximum number of variable bindings per layer (2^40)
pub const MAX_BINDINGS: usize = 40;

/// Sumcheck proof for a single layer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerProof<F: Field> {
    /// Polynomial evaluations for copy variable rounds
    pub copy_polys: Vec<Vec<F>>,
    
    /// Polynomial evaluations for hand/wire variable rounds
    pub hand_polys: Vec<Vec<F>>,
    
    /// Wire claims (evaluations at binding points)
    pub wire_claims: Vec<F>,
}

/// Complete sumcheck proof for a circuit
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SumcheckProof<F: Field> {
    /// Proofs for each layer (from output to input)
    pub layer_proofs: Vec<LayerProof<F>>,
    
    /// Final input evaluation
    pub input_eval: Vec<F>,
}

/// Sumcheck instance representing the claim to be proven
#[derive(Clone, Debug)]
pub struct SumcheckInstance<F: Field> {
    /// The circuit being evaluated
    pub circuit: Circuit<F>,
    
    /// Number of parallel copies
    pub num_copies: usize,
    
    /// Claimed output sum
    pub claimed_sum: F,
    
    /// Output binding point (if partially bound)
    pub output_binding: Option<Vec<F>>,
}

impl<F: Field> SumcheckInstance<F> {
    /// Create a new sumcheck instance
    pub fn new(
        circuit: Circuit<F>,
        num_copies: usize,
        claimed_sum: F,
    ) -> Result<Self> {
        if num_copies == 0 {
            return Err(LongfellowError::InvalidParameter(
                "Number of copies must be positive".to_string()
            ));
        }
        
        circuit.validate()?;
        
        Ok(Self {
            circuit,
            num_copies,
            claimed_sum,
            output_binding: None,
        })
    }
    
    /// Set output binding for verifying at a specific point
    pub fn with_output_binding(mut self, binding: Vec<F>) -> Result<Self> {
        let expected_len = self.circuit.num_output_vars();
        if binding.len() != expected_len {
            return Err(LongfellowError::InvalidParameter(
                format!("Output binding has wrong length: {} vs {}", 
                    binding.len(), expected_len)
            ));
        }
        
        self.output_binding = Some(binding);
        Ok(self)
    }
    
    /// Get total number of variables in the instance
    pub fn num_vars(&self) -> usize {
        let copy_vars = (self.num_copies - 1).next_power_of_two().trailing_zeros() as usize;
        copy_vars + self.circuit.num_vars()
    }
}

/// Options for proof generation/verification
#[derive(Clone, Debug)]
pub struct SumcheckOptions {
    /// Enable zero-knowledge (adds randomness)
    pub zero_knowledge: bool,
    
    /// Use parallel computation
    pub parallel: bool,
    
    /// Batch size for parallel operations
    pub batch_size: usize,
}

impl Default for SumcheckOptions {
    fn default() -> Self {
        Self {
            zero_knowledge: false,
            parallel: true,
            batch_size: 1024,
        }
    }
}

/// Helper function to compute multilinear extension
pub fn multilinear_extension<F: Field>(
    values: &[F],
    point: &[F],
) -> Result<F> {
    let num_vars = point.len();
    if values.len() != 1 << num_vars {
        return Err(LongfellowError::InvalidParameter(
            format!("Values length {} doesn't match 2^{}", values.len(), num_vars)
        ));
    }
    
    let mut result = values.to_vec();
    
    for (i, &r) in point.iter().enumerate() {
        let step = 1 << (num_vars - i - 1);
        for j in (0..result.len()).step_by(2 * step) {
            for k in 0..step {
                let low = result[j + k];
                let high = result[j + k + step];
                result[j + k] = low + r * (high - low);
            }
        }
    }
    
    Ok(result[0])
}

/// Compute the number of variables needed for n elements
pub fn num_vars_for_size(n: usize) -> usize {
    if n == 0 {
        0
    } else {
        (n - 1).next_power_of_two().trailing_zeros() as usize + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_multilinear_extension() {
        // Test on a simple function f(x,y) = x + 2y
        // Values at boolean cube: f(0,0)=0, f(1,0)=1, f(0,1)=2, f(1,1)=3
        let values = vec![
            Fp128::from(0),
            Fp128::from(1),
            Fp128::from(2),
            Fp128::from(3),
        ];
        
        // Evaluate at (0.5, 0.5)
        let point = vec![Fp128::from(1) / Fp128::from(2); 2];
        let result = multilinear_extension(&values, &point).unwrap();
        
        // Should be 1.5
        assert_eq!(result, Fp128::from(3) / Fp128::from(2));
    }
    
    #[test]
    fn test_num_vars_for_size() {
        assert_eq!(num_vars_for_size(0), 0);
        assert_eq!(num_vars_for_size(1), 1);
        assert_eq!(num_vars_for_size(2), 1);
        assert_eq!(num_vars_for_size(3), 2);
        assert_eq!(num_vars_for_size(4), 2);
        assert_eq!(num_vars_for_size(5), 3);
        assert_eq!(num_vars_for_size(8), 3);
        assert_eq!(num_vars_for_size(9), 4);
    }
}