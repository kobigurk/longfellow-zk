use crate::convolution::{Convolver, ConvolutionFactory};
use crate::traits::Field;
use crate::utility::batch_inverse_arithmetic;
use crate::reed_solomon_advanced::ConvolutionReedSolomon;
use longfellow_core::{LongfellowError, Result};
use rayon::prelude::*;

/// Reed-Solomon decoding using backward interpolation
/// 
/// This implements the inverse of the convolution-based interpolation:
/// Given evaluations at 0, 1, ..., m-1, recover the polynomial values at 0, 1, ..., n-1
pub trait ReedSolomonDecoder<F: Field> {
    /// Decode from m points back to n points
    fn decode(&self, encoded: &[F], erasures: &[usize]) -> Result<Vec<F>>;
    
    /// Verify that a codeword is valid
    fn verify(&self, codeword: &[F]) -> Result<bool>;
}

impl<F: Field, C: Convolver<F>> ConvolutionReedSolomon<F, C> {
    /// Backward interpolation to recover original n points from m points
    /// This is the inverse of the forward interpolation
    pub fn backward_interpolate(&self, y: &[F], output: &mut [F]) -> Result<()> {
        let n = self.degree_bound + 1;
        
        if y.len() < self.m {
            return Err(LongfellowError::InvalidParameter(
                "Input array too small".to_string()
            ));
        }
        
        if output.len() < n {
            return Err(LongfellowError::InvalidParameter(
                "Output array too small".to_string()
            ));
        }
        
        // For backward interpolation, we need to solve the linear system
        // This involves inverting the Vandermonde-like matrix
        
        // Step 1: Extract the known evaluations at n, n+1, ..., m-1
        let extended_evals = &y[n..self.m];
        
        // Step 2: Use the inverse of the convolution formula
        // We need to solve for the original polynomial coefficients
        
        // Create Vandermonde matrix for points 0, 1, ..., n-1
        let mut vandermonde = vec![vec![F::zero(); n]; n];
        for i in 0..n {
            let mut power = F::one();
            for j in 0..n {
                vandermonde[i][j] = power;
                power *= F::from_u64(i as u64);
            }
        }
        
        // Solve the linear system using Gaussian elimination
        let coeffs = solve_linear_system(&vandermonde, &y[..n])?;
        
        // Evaluate polynomial at original points
        for i in 0..n {
            output[i] = evaluate_polynomial(&coeffs, F::from_u64(i as u64));
        }
        
        Ok(())
    }
    
    /// Decode with erasure correction
    pub fn decode_with_erasures(&self, encoded: &[F], erasures: &[usize]) -> Result<Vec<F>> {
        if encoded.len() != self.m {
            return Err(LongfellowError::InvalidParameter(
                format!("Encoded length {} does not match m={}", encoded.len(), self.m)
            ));
        }
        
        let n = self.degree_bound + 1;
        let num_erasures = erasures.len();
        
        if num_erasures > self.m - n {
            return Err(LongfellowError::InvalidParameter(
                "Too many erasures for recovery".to_string()
            ));
        }
        
        // Create a copy with erasures set to zero
        let mut working = encoded.to_vec();
        for &pos in erasures {
            if pos < self.m {
                working[pos] = F::zero();
            }
        }
        
        // Use the non-erased positions to recover the polynomial
        let mut valid_positions = Vec::new();
        let mut valid_values = Vec::new();
        
        for i in 0..self.m {
            if !erasures.contains(&i) {
                valid_positions.push(i);
                valid_values.push(working[i]);
            }
        }
        
        // We need at least n valid positions to recover
        if valid_positions.len() < n {
            return Err(LongfellowError::InvalidParameter(
                "Not enough valid positions for recovery".to_string()
            ));
        }
        
        // Use the first n valid positions for recovery
        let recovery_positions = &valid_positions[..n];
        let recovery_values = &valid_values[..n];
        
        // Build Vandermonde matrix for recovery positions
        let mut vandermonde = vec![vec![F::zero(); n]; n];
        for (i, &pos) in recovery_positions.iter().enumerate() {
            let mut power = F::one();
            let point = F::from_u64(pos as u64);
            for j in 0..n {
                vandermonde[i][j] = power;
                power *= point;
            }
        }
        
        // Solve for polynomial coefficients
        let coeffs = solve_linear_system(&vandermonde, recovery_values)?;
        
        // Evaluate at original points 0, 1, ..., n-1
        let mut result = vec![F::zero(); n];
        for i in 0..n {
            result[i] = evaluate_polynomial(&coeffs, F::from_u64(i as u64));
        }
        
        Ok(result)
    }
    
    /// Verify that encoded data is a valid codeword
    pub fn verify_codeword(&self, codeword: &[F]) -> Result<bool> {
        if codeword.len() != self.m {
            return Ok(false);
        }
        
        let n = self.degree_bound + 1;
        
        // Extract first n points and interpolate
        let mut interpolated = codeword[..n].to_vec();
        interpolated.resize(self.m, F::zero());
        
        // Apply forward interpolation
        self.interpolate(&mut interpolated)?;
        
        // Check if interpolated values match the codeword
        for i in 0..self.m {
            if interpolated[i] != codeword[i] {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

/// Solve linear system Ax = b using Gaussian elimination with partial pivoting
fn solve_linear_system<F: Field>(a: &[Vec<F>], b: &[F]) -> Result<Vec<F>> {
    let n = a.len();
    if n == 0 || a[0].len() != n || b.len() != n {
        return Err(LongfellowError::InvalidParameter(
            "Invalid matrix dimensions".to_string()
        ));
    }
    
    // Create augmented matrix
    let mut aug = vec![vec![F::zero(); n + 1]; n];
    for i in 0..n {
        for j in 0..n {
            aug[i][j] = a[i][j];
        }
        aug[i][n] = b[i];
    }
    
    // Forward elimination with partial pivoting
    for col in 0..n {
        // Find pivot
        let mut max_row = col;
        for row in (col + 1)..n {
            if aug[row][col] != F::zero() {
                max_row = row;
                break;
            }
        }
        
        if aug[max_row][col] == F::zero() {
            return Err(LongfellowError::ArithmeticError(
                "Matrix is singular".to_string()
            ));
        }
        
        // Swap rows
        if max_row != col {
            aug.swap(col, max_row);
        }
        
        // Eliminate column
        let pivot = aug[col][col];
        let pivot_inv = pivot.invert()
            .ok_or_else(|| LongfellowError::ArithmeticError("Cannot invert pivot".to_string()))?;
        
        for row in (col + 1)..n {
            let factor = aug[row][col] * pivot_inv;
            for j in col..=n {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }
    
    // Back substitution
    let mut x = vec![F::zero(); n];
    for i in (0..n).rev() {
        x[i] = aug[i][n];
        for j in (i + 1)..n {
            x[i] -= aug[i][j] * x[j];
        }
        let diag_inv = aug[i][i].invert()
            .ok_or_else(|| LongfellowError::ArithmeticError("Cannot invert diagonal".to_string()))?;
        x[i] *= diag_inv;
    }
    
    Ok(x)
}

/// Evaluate polynomial with given coefficients at a point
fn evaluate_polynomial<F: Field>(coeffs: &[F], x: F) -> F {
    let mut result = F::zero();
    let mut power = F::one();
    
    for &coeff in coeffs {
        result += coeff * power;
        power *= x;
    }
    
    result
}

/// Syndrome-based decoding for systematic Reed-Solomon codes
pub struct SyndromeDecoder<F: Field> {
    n: usize,
    k: usize,
    generator_roots: Vec<F>,
}

impl<F: Field> SyndromeDecoder<F> {
    pub fn new(n: usize, k: usize, primitive_root: F) -> Result<Self> {
        if k >= n {
            return Err(LongfellowError::InvalidParameter(
                "k must be less than n".to_string()
            ));
        }
        
        // Generator polynomial has roots at powers of primitive_root
        let mut generator_roots = Vec::with_capacity(n - k);
        let mut root = F::one();
        for _ in 0..n - k {
            generator_roots.push(root);
            root *= primitive_root;
        }
        
        Ok(Self {
            n,
            k,
            generator_roots,
        })
    }
    
    /// Compute syndromes for error detection
    pub fn compute_syndromes(&self, received: &[F]) -> Vec<F> {
        if received.len() != self.n {
            return vec![];
        }
        
        let mut syndromes = Vec::with_capacity(self.n - self.k);
        
        for &root in &self.generator_roots {
            let mut syndrome = F::zero();
            let mut power = F::one();
            
            for &symbol in received {
                syndrome += symbol * power;
                power *= root;
            }
            
            syndromes.push(syndrome);
        }
        
        syndromes
    }
    
    /// Check if codeword has errors
    pub fn has_errors(&self, received: &[F]) -> bool {
        let syndromes = self.compute_syndromes(received);
        !syndromes.iter().all(|&s| s == F::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::fp128::Fp128;
    use crate::convolution::DirectConvolver;
    
    #[test]
    fn test_solve_linear_system() {
        // Test 2x2 system: x + 2y = 5, 3x + 4y = 11
        let a = vec![
            vec![Fp128::from_u64(1), Fp128::from_u64(2)],
            vec![Fp128::from_u64(3), Fp128::from_u64(4)],
        ];
        let b = vec![Fp128::from_u64(5), Fp128::from_u64(11)];
        
        let solution = solve_linear_system(&a, &b).unwrap();
        
        // Solution should be x=1, y=2
        assert_eq!(solution[0], Fp128::from_u64(1));
        assert_eq!(solution[1], Fp128::from_u64(2));
    }
    
    #[test]
    fn test_polynomial_evaluation() {
        // Test polynomial f(x) = 1 + 2x + 3x^2
        let coeffs = vec![
            Fp128::from_u64(1),
            Fp128::from_u64(2),
            Fp128::from_u64(3),
        ];
        
        // f(0) = 1
        assert_eq!(evaluate_polynomial(&coeffs, Fp128::zero()), Fp128::from_u64(1));
        
        // f(1) = 1 + 2 + 3 = 6
        assert_eq!(evaluate_polynomial(&coeffs, Fp128::one()), Fp128::from_u64(6));
        
        // f(2) = 1 + 4 + 12 = 17
        assert_eq!(evaluate_polynomial(&coeffs, Fp128::from_u64(2)), Fp128::from_u64(17));
    }
    
    #[test]
    fn test_syndrome_computation() {
        let decoder = SyndromeDecoder::<Fp128>::new(7, 4, Fp128::from_u64(3)).unwrap();
        
        // Valid codeword should have zero syndromes
        let valid_codeword = vec![Fp128::one(); 7];
        assert!(!decoder.has_errors(&valid_codeword));
    }
}