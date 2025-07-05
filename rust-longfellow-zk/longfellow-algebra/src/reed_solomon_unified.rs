use crate::traits::Field;
use crate::convolution::{FftConvolutionFactory, ConvolutionFactory};
use crate::reed_solomon_advanced::{ConvolutionReedSolomon, ConvolutionReedSolomonFactory};
use crate::reed_solomon_lch14::{LCH14ReedSolomon, LCH14ReedSolomonFactory};
use longfellow_core::{LongfellowError, Result};

/// Unified Reed-Solomon encoder that automatically selects the best implementation
pub enum UnifiedReedSolomon<F: Field> {
    /// Convolution-based implementation for prime fields
    Convolution(Box<ConvolutionReedSolomon<F, <FftConvolutionFactory<F> as ConvolutionFactory<F>>::Convolver>>),
    /// LCH14 implementation for binary fields
    Lch14(Box<LCH14ReedSolomon<F>>),
}

impl<F: Field> UnifiedReedSolomon<F> {
    /// Create a new Reed-Solomon encoder, automatically selecting the best implementation
    pub fn new(n: usize, m: usize) -> Result<Self> {
        if Self::is_binary_field() {
            // Use LCH14 for binary fields
            let factory = LCH14ReedSolomonFactory::new();
            let rs = factory.make(n, m)?;
            Ok(UnifiedReedSolomon::Lch14(Box::new(rs)))
        } else {
            // Use convolution-based for prime fields
            let omega_provider = |size: usize| {
                // This should compute proper root of unity for the field
                // For now, use a placeholder
                Self::compute_root_of_unity(size)
            };
            
            let conv_factory = FftConvolutionFactory::new(omega_provider);
            let rs_factory = ConvolutionReedSolomonFactory::new(conv_factory);
            let rs = rs_factory.make(n, m)?;
            Ok(UnifiedReedSolomon::Convolution(Box::new(rs)))
        }
    }
    
    /// Interpolate polynomial values
    pub fn interpolate(&self, y: &mut [F]) -> Result<()> {
        match self {
            UnifiedReedSolomon::Convolution(rs) => rs.interpolate(y),
            UnifiedReedSolomon::Lch14(rs) => rs.interpolate(y),
        }
    }
    
    /// Check if field has characteristic 2
    fn is_binary_field() -> bool {
        F::one() + F::one() == F::zero()
    }
    
    /// Compute primitive root of unity for given size
    fn compute_root_of_unity(size: usize) -> F {
        // This is field-specific and should be properly implemented
        // For now, return a placeholder
        // In practice, this would:
        // 1. Find a generator of the multiplicative group
        // 2. Raise it to the power (p-1)/size to get a size-th root of unity
        F::from_u64(3).pow(&[((F::characteristic() - 1) / size as u64)])
    }
}

/// Configuration for Reed-Solomon encoding
pub struct ReedSolomonConfig {
    /// Force use of specific implementation
    pub force_implementation: Option<ReedSolomonImpl>,
    /// Custom omega provider for FFT
    pub omega_provider: Option<Box<dyn Fn(usize) -> Box<dyn Fn() -> dyn std::any::Any>>>,
}

#[derive(Clone, Copy, Debug)]
pub enum ReedSolomonImpl {
    Convolution,
    Lch14,
    Auto,
}

impl Default for ReedSolomonConfig {
    fn default() -> Self {
        Self {
            force_implementation: None,
            omega_provider: None,
        }
    }
}

/// Factory for creating Reed-Solomon encoders with custom configuration
pub struct ReedSolomonFactory<F: Field> {
    config: ReedSolomonConfig,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field> ReedSolomonFactory<F> {
    pub fn new(config: ReedSolomonConfig) -> Self {
        Self {
            config,
            _phantom: std::marker::PhantomData,
        }
    }
    
    pub fn default() -> Self {
        Self::new(ReedSolomonConfig::default())
    }
    
    pub fn make(&self, n: usize, m: usize) -> Result<UnifiedReedSolomon<F>> {
        let impl_type = self.config.force_implementation.unwrap_or(ReedSolomonImpl::Auto);
        
        match impl_type {
            ReedSolomonImpl::Convolution => {
                let omega_provider = |size: usize| {
                    UnifiedReedSolomon::<F>::compute_root_of_unity(size)
                };
                
                let conv_factory = FftConvolutionFactory::new(omega_provider);
                let rs_factory = ConvolutionReedSolomonFactory::new(conv_factory);
                let rs = rs_factory.make(n, m)?;
                Ok(UnifiedReedSolomon::Convolution(Box::new(rs)))
            }
            ReedSolomonImpl::Lch14 => {
                if !UnifiedReedSolomon::<F>::is_binary_field() {
                    return Err(LongfellowError::InvalidParameter(
                        "LCH14 only works for binary fields".to_string()
                    ));
                }
                
                let factory = LCH14ReedSolomonFactory::new();
                let rs = factory.make(n, m)?;
                Ok(UnifiedReedSolomon::Lch14(Box::new(rs)))
            }
            ReedSolomonImpl::Auto => UnifiedReedSolomon::new(n, m),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::fp128::Fp128;
    
    #[test]
    fn test_unified_reed_solomon() {
        let n = 8;
        let m = 16;
        
        // Test with prime field
        let rs = UnifiedReedSolomon::<Fp128>::new(n, m).unwrap();
        
        // Test data
        let mut y = vec![Fp128::zero(); m];
        for i in 0..n {
            y[i] = Fp128::from_u64(i as u64 + 1);
        }
        
        rs.interpolate(&mut y).unwrap();
        
        // Verify we got values
        for i in n..m {
            println!("y[{}] = {:?}", i, y[i]);
        }
    }
    
    #[test]
    fn test_factory_with_config() {
        let config = ReedSolomonConfig {
            force_implementation: Some(ReedSolomonImpl::Convolution),
            ..Default::default()
        };
        
        let factory = ReedSolomonFactory::<Fp128>::new(config);
        let rs = factory.make(4, 8).unwrap();
        
        // Verify we got convolution implementation
        match rs {
            UnifiedReedSolomon::Convolution(_) => {},
            _ => panic!("Expected convolution implementation"),
        }
    }
}