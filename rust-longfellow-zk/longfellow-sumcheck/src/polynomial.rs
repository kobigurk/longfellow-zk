/// Polynomial utilities for sumcheck protocol

use longfellow_algebra::traits::Field;
use longfellow_core::{LongfellowError, Result};
use serde::{Deserialize, Serialize};

/// Univariate polynomial for sumcheck rounds
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnivariatePoly<F: Field> {
    /// Coefficients from low to high degree
    pub coeffs: Vec<F>,
}

impl<F: Field> UnivariatePoly<F> {
    /// Create a polynomial from coefficients
    pub fn new(coeffs: Vec<F>) -> Self {
        Self { coeffs }
    }
    
    /// Create a zero polynomial
    pub fn zero() -> Self {
        Self { coeffs: vec![F::zero()] }
    }
    
    /// Create a constant polynomial
    pub fn constant(c: F) -> Self {
        Self { coeffs: vec![c] }
    }
    
    /// Get the degree of the polynomial
    pub fn degree(&self) -> usize {
        if self.coeffs.is_empty() {
            0
        } else {
            self.coeffs.len() - 1
        }
    }
    
    /// Evaluate at a point
    pub fn evaluate(&self, x: F) -> F {
        if self.coeffs.is_empty() {
            return F::zero();
        }
        
        // Horner's method
        let mut result = self.coeffs[self.coeffs.len() - 1];
        for i in (0..self.coeffs.len() - 1).rev() {
            result = result * x + self.coeffs[i];
        }
        result
    }
    
    /// Create polynomial from evaluations at 0, 1, 2, ...
    pub fn interpolate(evals: &[F]) -> Result<Self> {
        if evals.is_empty() {
            return Ok(Self::zero());
        }
        
        let n = evals.len();
        let mut coeffs = vec![F::zero(); n];
        
        // Lagrange interpolation
        for (i, &y_i) in evals.iter().enumerate() {
            let mut basis_coeffs = vec![F::zero(); n];
            basis_coeffs[0] = F::one();
            
            // Build Lagrange basis polynomial
            for j in 0..n {
                if i != j {
                    // Multiply by (x - j) / (i - j)
                    let denom = F::from(i as u64) - F::from(j as u64);
                    let inv_denom = denom.invert()
                        .ok_or_else(|| LongfellowError::InvalidParameter(
                            "Interpolation points not distinct".to_string()
                        ))?;
                    
                    // Multiply polynomial by (x - j)
                    for k in (1..=basis_coeffs.len()).rev() {
                        if k > 0 {
                            basis_coeffs[k] = basis_coeffs[k-1] - F::from(j as u64) * basis_coeffs[k];
                        }
                    }
                    basis_coeffs[0] = -F::from(j as u64) * basis_coeffs[0];
                    
                    // Divide by (i - j)
                    for coeff in &mut basis_coeffs {
                        *coeff *= inv_denom;
                    }
                }
            }
            
            // Add contribution
            for (k, &basis_coeff) in basis_coeffs.iter().enumerate() {
                coeffs[k] += y_i * basis_coeff;
            }
        }
        
        Ok(Self::new(coeffs))
    }
    
    /// Add two polynomials
    pub fn add(&self, other: &Self) -> Self {
        let max_len = self.coeffs.len().max(other.coeffs.len());
        let mut result = vec![F::zero(); max_len];
        
        for (i, &c) in self.coeffs.iter().enumerate() {
            result[i] += c;
        }
        for (i, &c) in other.coeffs.iter().enumerate() {
            result[i] += c;
        }
        
        Self::new(result)
    }
    
    /// Scale polynomial by a constant
    pub fn scale(&self, s: F) -> Self {
        Self::new(self.coeffs.iter().map(|&c| c * s).collect())
    }
}

/// Multilinear polynomial representation
#[derive(Clone, Debug)]
pub struct MultilinearPoly<F: Field> {
    /// Evaluations at boolean hypercube
    pub evals: Vec<F>,
    /// Number of variables
    pub num_vars: usize,
}

impl<F: Field> MultilinearPoly<F> {
    /// Create from evaluations
    pub fn new(evals: Vec<F>) -> Result<Self> {
        let n = evals.len();
        if n == 0 || (n & (n - 1)) != 0 {
            return Err(LongfellowError::InvalidParameter(
                "Evaluation count must be a power of 2".to_string()
            ));
        }
        
        let num_vars = n.trailing_zeros() as usize;
        Ok(Self { evals, num_vars })
    }
    
    /// Evaluate at a point
    pub fn evaluate(&self, point: &[F]) -> Result<F> {
        if point.len() != self.num_vars {
            return Err(LongfellowError::InvalidParameter(
                format!("Point dimension {} doesn't match polynomial vars {}", 
                    point.len(), self.num_vars)
            ));
        }
        
        crate::multilinear_extension(&self.evals, point)
    }
    
    /// Bind the first variable to a value
    pub fn bind_first(&self, value: F) -> Result<Self> {
        if self.num_vars == 0 {
            return Err(LongfellowError::InvalidParameter(
                "Cannot bind variable of constant polynomial".to_string()
            ));
        }
        
        let new_size = self.evals.len() / 2;
        let mut new_evals = vec![F::zero(); new_size];
        
        for i in 0..new_size {
            let low = self.evals[i];
            let high = self.evals[i + new_size];
            new_evals[i] = low + value * (high - low);
        }
        
        Ok(Self {
            evals: new_evals,
            num_vars: self.num_vars - 1,
        })
    }
    
    /// Get univariate polynomial by fixing all but one variable
    pub fn to_univariate(&self, var_index: usize, point: &[F]) -> Result<UnivariatePoly<F>> {
        if var_index >= self.num_vars {
            return Err(LongfellowError::InvalidParameter(
                format!("Variable index {} out of range", var_index)
            ));
        }
        
        if point.len() != self.num_vars - 1 {
            return Err(LongfellowError::InvalidParameter(
                "Point dimension mismatch".to_string()
            ));
        }
        
        // Evaluate at different values of the variable
        let mut evals = Vec::new();
        for val in 0..=self.degree_bound() {
            let mut full_point = vec![F::zero(); self.num_vars];
            
            // Fill in the fixed variables
            let mut j = 0;
            for i in 0..self.num_vars {
                if i == var_index {
                    full_point[i] = F::from(val as u64);
                } else {
                    full_point[i] = point[j];
                    j += 1;
                }
            }
            
            evals.push(self.evaluate(&full_point)?);
        }
        
        UnivariatePoly::interpolate(&evals)
    }
    
    /// Get the degree bound (always 1 for multilinear)
    pub fn degree_bound(&self) -> usize {
        1
    }
}

/// Helper functions for polynomial operations in sumcheck
pub struct PolyHelper;

impl PolyHelper {
    /// Compute sum over boolean hypercube
    pub fn sum_over_boolean_hypercube<F: Field>(poly: &MultilinearPoly<F>) -> F {
        poly.evals.iter().sum()
    }
    
    /// Create eq polynomial: eq(x, r) = prod_i (x_i * r_i + (1-x_i) * (1-r_i))
    pub fn eq_polynomial<F: Field>(r: &[F]) -> Result<MultilinearPoly<F>> {
        let num_vars = r.len();
        let size = 1 << num_vars;
        let mut evals = vec![F::zero(); size];
        
        for i in 0..size {
            let mut prod = F::one();
            for j in 0..num_vars {
                let bit = (i >> j) & 1;
                if bit == 1 {
                    prod *= r[j];
                } else {
                    prod *= F::one() - r[j];
                }
            }
            evals[i] = prod;
        }
        
        MultilinearPoly::new(evals)
    }
    
    /// Combine polynomials for parallel sumcheck (degree 3)
    pub fn combine_copy_polys<F: Field>(
        polys: &[UnivariatePoly<F>],
        challenge: F,
    ) -> UnivariatePoly<F> {
        if polys.is_empty() {
            return UnivariatePoly::zero();
        }
        
        let mut result = polys[0].clone();
        let mut power = challenge;
        
        for poly in &polys[1..] {
            result = result.add(&poly.scale(power));
            power *= challenge;
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_univariate_poly() {
        // p(x) = 2x^2 + 3x + 1
        let poly = UnivariatePoly::new(vec![
            Fp128::from(1),
            Fp128::from(3),
            Fp128::from(2),
        ]);
        
        assert_eq!(poly.degree(), 2);
        assert_eq!(poly.evaluate(Fp128::zero()), Fp128::from(1));
        assert_eq!(poly.evaluate(Fp128::one()), Fp128::from(6)); // 2 + 3 + 1
        assert_eq!(poly.evaluate(Fp128::from(2)), Fp128::from(15)); // 8 + 6 + 1
    }
    
    #[test]
    fn test_interpolation() {
        // Interpolate through points (0,1), (1,3), (2,7)
        let evals = vec![Fp128::from(1), Fp128::from(3), Fp128::from(7)];
        let poly = UnivariatePoly::interpolate(&evals).unwrap();
        
        // Should get p(x) = x^2 + x + 1
        assert_eq!(poly.evaluate(Fp128::zero()), Fp128::from(1));
        assert_eq!(poly.evaluate(Fp128::one()), Fp128::from(3));
        assert_eq!(poly.evaluate(Fp128::from(2)), Fp128::from(7));
        assert_eq!(poly.evaluate(Fp128::from(3)), Fp128::from(13)); // 9 + 3 + 1
    }
    
    #[test]
    fn test_multilinear_poly() {
        // f(x,y) = 2xy + x + y
        // Evaluations: f(0,0)=0, f(1,0)=1, f(0,1)=1, f(1,1)=4
        let evals = vec![
            Fp128::from(0),
            Fp128::from(1),
            Fp128::from(1),
            Fp128::from(4),
        ];
        
        let poly = MultilinearPoly::new(evals).unwrap();
        
        // Test evaluation at (0.5, 0.5)
        let point = vec![Fp128::from(1) / Fp128::from(2); 2];
        let result = poly.evaluate(&point).unwrap();
        
        // f(0.5, 0.5) = 2*0.5*0.5 + 0.5 + 0.5 = 0.5 + 1 = 1.5
        assert_eq!(result, Fp128::from(3) / Fp128::from(2));
    }
    
    #[test]
    fn test_eq_polynomial() {
        let r = vec![Fp128::from(2), Fp128::from(3)];
        let eq_poly = PolyHelper::eq_polynomial(&r).unwrap();
        
        // eq((0,0), (2,3)) = (1-0)*(1-2) * (1-0)*(1-3) = (-1) * (-2) = 2
        // But in field arithmetic this is different
        
        // eq((1,1), (2,3)) = 1*2 * 1*3 = 6
        let point = vec![Fp128::one(), Fp128::one()];
        let val = eq_poly.evaluate(&point).unwrap();
        assert_eq!(val, Fp128::from(6));
    }
}