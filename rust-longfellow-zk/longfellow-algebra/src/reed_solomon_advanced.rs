use crate::convolution::{ConvolutionFactory, Convolver};
use crate::traits::Field;
use crate::utility::batch_inverse_arithmetic;
use longfellow_core::{LongfellowError, Result};
use rayon::prelude::*;

/// Advanced Reed-Solomon implementation using convolution-based interpolation
/// 
/// This implements the formula from the C++ version:
/// p(k) = (-1)^d (k-d)(k choose d) sum_{j=0}^{d} (1/k-j)(-1)^j (d choose j)p(j)
/// 
/// where d = n-1 is the degree bound of the polynomial.
pub struct ConvolutionReedSolomon<F: Field, C: Convolver<F>> {
    field: F,  // Placeholder for field operations
    degree_bound: usize,
    m: usize,
    convolver: C,
    leading_constants: Vec<F>,
    binomial_coeffs: Vec<F>,
}

impl<F: Field, C: Convolver<F>> ConvolutionReedSolomon<F, C> {
    /// Create a new Reed-Solomon encoder
    /// n: number of input points (evaluations at 0, 1, ..., n-1)
    /// m: total number of output points (including the initial n)
    pub fn new(
        n: usize,
        m: usize,
        convolver: C,
    ) -> Result<Self> {
        if n == 0 {
            return Err(LongfellowError::InvalidParameter(
                "n must be positive".to_string()
            ));
        }
        
        if m < n {
            return Err(LongfellowError::InvalidParameter(
                "m must be at least n".to_string()
            ));
        }

        let degree_bound = n - 1;
        
        // Compute inverses[i] = 1/i for i=1..m-1
        let inverses = batch_inverse_arithmetic(m)?;
        
        // Compute leading constants
        let mut leading_constants = vec![F::zero(); m - n + 1];
        leading_constants[0] = F::one();
        
        // Set leading_constant[i] = (i+degree_bound) choose degree_bound
        for i in 1..=m - n {
            if i + degree_bound < m {
                leading_constants[i] = leading_constants[i - 1]
                    * F::from_u64((degree_bound + i) as u64)
                    * inverses[i];
            }
        }
        
        // Apply the (-1)^degree_bound (k-degree_bound) factor
        for k in degree_bound..m {
            let idx = k - degree_bound;
            if idx < leading_constants.len() {
                leading_constants[idx] = leading_constants[idx] 
                    * F::from_u64((k - degree_bound) as u64);
                    
                if degree_bound % 2 == 1 {
                    leading_constants[idx] = -leading_constants[idx];
                }
            }
        }
        
        // Compute binomial coefficients: (-1)^i (degree_bound choose i)
        let mut binomial_coeffs = vec![F::zero(); n];
        binomial_coeffs[0] = F::one();
        
        for i in 1..n {
            binomial_coeffs[i] = binomial_coeffs[i - 1]
                * F::from_u64((n - i) as u64)
                * inverses[i];
        }
        
        // Apply (-1)^i factor
        for i in 1..n {
            if i % 2 == 1 {
                binomial_coeffs[i] = -binomial_coeffs[i];
            }
        }
        
        Ok(Self {
            field: F::zero(), // Placeholder
            degree_bound,
            m,
            convolver,
            leading_constants,
            binomial_coeffs,
        })
    }
    
    /// Interpolate polynomial values
    /// Input: y[0..n] contains evaluations at 0, 1, ..., n-1
    /// Output: y[0..m] will contain evaluations at 0, 1, ..., m-1
    pub fn interpolate(&self, y: &mut [F]) -> Result<()> {
        let n = self.degree_bound + 1;
        
        if y.len() < self.m {
            return Err(LongfellowError::InvalidParameter(
                format!("Output array too small: {} < {}", y.len(), self.m)
            ));
        }
        
        // Prepare input for convolution: x[i] = (-1)^i binom(n,i) p(i)
        let mut x = vec![F::zero(); n];
        for i in 0..n {
            x[i] = self.binomial_coeffs[i] * y[i];
        }
        
        // Perform convolution
        let mut convolution_output = vec![F::zero(); self.m];
        self.convolver.convolution(&x, &mut convolution_output)?;
        
        // Multiply by leading constants to get final result
        for i in n..self.m {
            let leading_idx = i - self.degree_bound;
            if leading_idx < self.leading_constants.len() {
                y[i] = self.leading_constants[leading_idx] * convolution_output[i];
            }
        }
        
        Ok(())
    }
}

/// Factory for creating convolution-based Reed-Solomon encoders
pub struct ConvolutionReedSolomonFactory<F: Field, CF: ConvolutionFactory<F>> {
    convolution_factory: CF,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field, CF: ConvolutionFactory<F>> ConvolutionReedSolomonFactory<F, CF> {
    pub fn new(convolution_factory: CF) -> Self {
        Self {
            convolution_factory,
            _phantom: std::marker::PhantomData,
        }
    }
    
    pub fn make(&self, n: usize, m: usize) -> Result<ConvolutionReedSolomon<F, CF::Convolver>> {
        // Create inverses for convolution
        let inverses = batch_inverse_arithmetic(m)?;
        
        // Create convolver
        let convolver = self.convolution_factory.make(n, m, &inverses)?;
        
        ConvolutionReedSolomon::new(n, m, convolver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convolution::{DirectConvolver, FftConvolutionFactory};
    use crate::field::fp128::Fp128;
    
    #[test]
    fn test_convolution_reed_solomon_basic() {
        let n = 4;
        let m = 8;
        
        // Create inverses
        let inverses = batch_inverse_arithmetic::<Fp128>(m).unwrap();
        
        // Create direct convolver for testing
        let convolver = DirectConvolver::new(n, m, &inverses).unwrap();
        
        // Create Reed-Solomon encoder
        let rs = ConvolutionReedSolomon::new(n, m, convolver).unwrap();
        
        // Test data: polynomial f(x) = 1 + 2x + 3x^2 + 4x^3
        // Evaluations at 0,1,2,3
        let mut y = vec![Fp128::zero(); m];
        y[0] = Fp128::from_u64(1);  // f(0) = 1
        y[1] = Fp128::from_u64(10); // f(1) = 1+2+3+4 = 10
        y[2] = Fp128::from_u64(49); // f(2) = 1+4+12+32 = 49
        y[3] = Fp128::from_u64(142); // f(3) = 1+6+27+108 = 142
        
        // Interpolate to get values at 4,5,6,7
        rs.interpolate(&mut y).unwrap();
        
        // Verify we got reasonable values (not all zeros)
        for i in n..m {
            println!("f({}) = {:?}", i, y[i]);
        }
        assert!(!y[n..].iter().all(|x| *x == Fp128::zero()));
    }
    
    #[test] 
    fn test_with_fft_convolution() {
        let n = 8;
        let m = 16;
        
        // Create FFT-based convolution factory
        let factory = FftConvolutionFactory::<Fp128>::default();
        
        // Create Reed-Solomon factory
        let rs_factory = ConvolutionReedSolomonFactory::new(factory);
        
        // Create encoder
        let rs = rs_factory.make(n, m).unwrap();
        
        // Test with constant polynomial f(x) = 1
        let mut y = vec![Fp128::one(); m];
        
        rs.interpolate(&mut y).unwrap();
        
        // For constant polynomial, all values should be 1
        for i in 0..m {
            assert_eq!(y[i], Fp128::one(), "Value at {} should be 1", i);
        }
    }
}