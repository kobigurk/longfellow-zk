use crate::fft::FFT;
use crate::traits::Field;
use longfellow_core::{LongfellowError, Result};
use rayon::prelude::*;

/// Trait for convolution operations
pub trait Convolver<F: Field> {
    /// Compute convolution of two sequences
    /// Output[i] = sum_{j=0}^{n-1} a[j] * b[i-j] where b is padded with zeros
    fn convolution(&self, a: &[F], output: &mut [F]) -> Result<()>;
}

/// FFT-based convolution implementation
pub struct FftConvolver<F: Field> {
    fft: FFT<F>,
    inverse_sequence: Vec<F>,
    n: usize,
    m: usize,
}

impl<F: Field> FftConvolver<F> {
    /// Create a new FFT-based convolver
    /// n: input size
    /// m: output size  
    /// inverses: precomputed inverses[i] = 1/i for i=1..m-1, inverses[0] = 0
    pub fn new(n: usize, m: usize, inverses: &[F], omega: F) -> Result<Self> {
        if inverses.len() != m {
            return Err(LongfellowError::InvalidParameter(
                "Inverses array size mismatch".to_string()
            ));
        }

        // Prepare the inverse sequence for convolution
        // For Reed-Solomon: b[i] = 1/i for i=1..m-1
        let mut inverse_sequence = vec![F::zero(); m];
        inverse_sequence[1..].copy_from_slice(&inverses[1..]);

        // Find next power of 2 for FFT size
        let fft_size = (n + m - 1).next_power_of_two();
        let fft = FFT::new(fft_size, omega)?;

        Ok(Self {
            fft,
            inverse_sequence,
            n,
            m,
        })
    }

    /// Perform the convolution using FFT
    fn fft_convolution(&self, a: &[F], b: &[F], output: &mut [F]) -> Result<()> {
        let fft_size = self.fft.size();
        
        // Pad inputs to FFT size
        let mut a_padded = vec![F::zero(); fft_size];
        let mut b_padded = vec![F::zero(); fft_size];
        
        a_padded[..a.len()].copy_from_slice(a);
        b_padded[..b.len()].copy_from_slice(b);
        
        // Forward FFT
        self.fft.forward(&mut a_padded)?;
        self.fft.forward(&mut b_padded)?;
        
        // Pointwise multiplication in frequency domain
        a_padded.par_iter_mut()
            .zip(b_padded.par_iter())
            .for_each(|(a, b)| *a *= b);
        
        // Inverse FFT
        self.fft.inverse(&mut a_padded)?;
        
        // Copy result (only the relevant part)
        let output_len = output.len().min(a.len() + b.len() - 1);
        output[..output_len].copy_from_slice(&a_padded[..output_len]);
        
        Ok(())
    }
}

impl<F: Field> Convolver<F> for FftConvolver<F> {
    fn convolution(&self, a: &[F], output: &mut [F]) -> Result<()> {
        if a.len() != self.n {
            return Err(LongfellowError::InvalidParameter(
                format!("Input size {} does not match expected {}", a.len(), self.n)
            ));
        }
        
        if output.len() != self.m {
            return Err(LongfellowError::InvalidParameter(
                format!("Output size {} does not match expected {}", output.len(), self.m)
            ));
        }

        // For Reed-Solomon, we convolve with the inverse sequence
        self.fft_convolution(a, &self.inverse_sequence, output)?;
        
        Ok(())
    }
}

/// Factory for creating convolvers
pub trait ConvolutionFactory<F: Field> {
    type Convolver: Convolver<F>;
    
    /// Create a convolver for given input/output sizes
    fn make(&self, n: usize, m: usize, inverses: &[F]) -> Result<Self::Convolver>;
}

/// FFT-based convolution factory
pub struct FftConvolutionFactory<F: Field> {
    omega_provider: Box<dyn Fn(usize) -> F>,
}

impl<F: Field> FftConvolutionFactory<F> {
    /// Create a new factory with an omega provider function
    pub fn new<P: Fn(usize) -> F + 'static>(omega_provider: P) -> Self {
        Self {
            omega_provider: Box::new(omega_provider),
        }
    }
    
    /// Create with default omega computation
    pub fn default() -> Self {
        Self::new(|size| {
            // This is a placeholder - in practice, we'd compute proper root of unity
            // based on the field and required size
            F::from_u64(3).pow(&[((F::characteristic() - 1) / size as u64)])
        })
    }
}

impl<F: Field> ConvolutionFactory<F> for FftConvolutionFactory<F> {
    type Convolver = FftConvolver<F>;
    
    fn make(&self, n: usize, m: usize, inverses: &[F]) -> Result<Self::Convolver> {
        let fft_size = (n + m - 1).next_power_of_two();
        let omega = (self.omega_provider)(fft_size);
        FftConvolver::new(n, m, inverses, omega)
    }
}

/// Direct convolution for small sizes or testing
pub struct DirectConvolver<F: Field> {
    inverse_sequence: Vec<F>,
    n: usize,
    m: usize,
}

impl<F: Field> DirectConvolver<F> {
    pub fn new(n: usize, m: usize, inverses: &[F]) -> Result<Self> {
        if inverses.len() != m {
            return Err(LongfellowError::InvalidParameter(
                "Inverses array size mismatch".to_string()
            ));
        }

        let mut inverse_sequence = vec![F::zero(); m];
        inverse_sequence[1..].copy_from_slice(&inverses[1..]);

        Ok(Self {
            inverse_sequence,
            n,
            m,
        })
    }
}

impl<F: Field> Convolver<F> for DirectConvolver<F> {
    fn convolution(&self, a: &[F], output: &mut [F]) -> Result<()> {
        if a.len() != self.n || output.len() != self.m {
            return Err(LongfellowError::InvalidParameter(
                "Size mismatch".to_string()
            ));
        }

        // Direct O(n*m) convolution
        for i in 0..self.m {
            output[i] = F::zero();
            for j in 0..self.n.min(i + 1) {
                if i - j < self.inverse_sequence.len() {
                    output[i] += a[j] * self.inverse_sequence[i - j];
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::fp128::Fp128;
    
    #[test]
    fn test_direct_convolution() {
        let n = 4;
        let m = 8;
        
        // Create test inverses
        let mut inverses = vec![Fp128::zero(); m];
        for i in 1..m {
            inverses[i] = Fp128::from_u64(i as u64).invert().unwrap();
        }
        
        let conv = DirectConvolver::new(n, m, &inverses).unwrap();
        
        // Test input
        let input = vec![Fp128::one(); n];
        let mut output = vec![Fp128::zero(); m];
        
        conv.convolution(&input, &mut output).unwrap();
        
        // Verify output is non-zero
        assert!(!output.iter().all(|x| *x == Fp128::zero()));
    }
}