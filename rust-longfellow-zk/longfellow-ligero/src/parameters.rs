/// Ligero protocol parameters

use longfellow_core::{LongfellowError, Result};
use serde::{Deserialize, Serialize};

/// Ligero protocol parameters
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LigeroParams {
    /// Block size (number of field elements per block)
    pub block_size: usize,
    
    /// Extension factor for Reed-Solomon encoding
    pub extension_factor: usize,
    
    /// Number of blinding rows for zero-knowledge
    pub num_blinding_rows: usize,
    
    /// Number of column openings for soundness
    pub num_col_openings: usize,
    
    /// Number of low-degree test queries
    pub num_ldt_queries: usize,
    
    /// Security parameter (bits)
    pub security_bits: usize,
    
    /// Use subfield optimization
    pub use_subfield: bool,
}

impl LigeroParams {
    /// Create parameters for a given security level
    pub fn new(security_bits: usize) -> Result<Self> {
        match security_bits {
            80 => Ok(Self::security_80()),
            128 => Ok(Self::security_128()),
            256 => Ok(Self::security_256()),
            _ => Err(LongfellowError::InvalidParameter(
                format!("Unsupported security level: {} bits", security_bits)
            )),
        }
    }
    
    /// 80-bit security parameters
    pub fn security_80() -> Self {
        Self {
            block_size: 64,
            extension_factor: 4,
            num_blinding_rows: 3,
            num_col_openings: 80,
            num_ldt_queries: 40,
            security_bits: 80,
            use_subfield: false,
        }
    }
    
    /// 128-bit security parameters
    pub fn security_128() -> Self {
        Self {
            block_size: 128,
            extension_factor: 4,
            num_blinding_rows: 3,
            num_col_openings: 189,
            num_ldt_queries: 64,
            security_bits: 128,
            use_subfield: false,
        }
    }
    
    /// 256-bit security parameters
    pub fn security_256() -> Self {
        Self {
            block_size: 256,
            extension_factor: 4,
            num_blinding_rows: 3,
            num_col_openings: 400,
            num_ldt_queries: 128,
            security_bits: 256,
            use_subfield: false,
        }
    }
    
    /// Get row size after encoding
    pub fn encoded_row_size(&self) -> usize {
        self.block_enc_size()
    }
    
    /// Get block encoding size: 2 * block - 1 + block_ext
    pub fn block_enc_size(&self) -> usize {
        2 * self.block_size - 1 + self.block_ext_size()
    }
    
    /// Get extension block size
    pub fn block_ext_size(&self) -> usize {
        self.block_size * (self.extension_factor - 1)
    }
    
    /// Get number of blocks needed for witnesses
    pub fn num_witness_blocks(&self, num_witnesses: usize) -> usize {
        (num_witnesses + self.block_size - 1) / self.block_size
    }
    
    /// Get maximum number of witnesses supported
    pub fn max_witnesses(&self) -> usize {
        // Limit based on practical tableau size
        self.block_size * 10000
    }
    
    /// Get number of rows needed for quadratic constraints
    pub fn num_quadratic_rows(&self, num_constraints: usize) -> usize {
        (num_constraints + self.block_size - 1) / self.block_size
    }
    
    /// Get total tableau height
    pub fn tableau_height(&self, num_witness_blocks: usize, num_quad_rows: usize) -> usize {
        self.num_blinding_rows + num_witness_blocks + num_quad_rows
    }
    
    /// Validate parameters
    pub fn validate(&self) -> Result<()> {
        if self.block_size == 0 || (self.block_size & (self.block_size - 1)) != 0 {
            return Err(LongfellowError::InvalidParameter(
                "Block size must be a power of 2".to_string()
            ));
        }
        
        if self.extension_factor < 2 {
            return Err(LongfellowError::InvalidParameter(
                "Extension factor must be at least 2".to_string()
            ));
        }
        
        if self.num_blinding_rows < 3 {
            return Err(LongfellowError::InvalidParameter(
                "Need at least 3 blinding rows for security".to_string()
            ));
        }
        
        if self.num_col_openings == 0 {
            return Err(LongfellowError::InvalidParameter(
                "Must have at least one column opening".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Get soundness error probability (approximate)
    pub fn soundness_error(&self) -> f64 {
        // Simplified estimate: 1/2^num_col_openings
        (0.5_f64).powi(self.num_col_openings as i32)
    }
}

/// Row indices for special purposes
pub mod row_indices {
    /// Low-degree test blinding row
    pub const ILDT: usize = 0;
    
    /// Dot-product check blinding row
    pub const IDOT: usize = 1;
    
    /// Quadratic check blinding row
    pub const IQUAD: usize = 2;
    
    /// First witness row
    pub const WITNESS_START: usize = 3;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parameter_creation() {
        let params = LigeroParams::security_128();
        assert_eq!(params.security_bits, 128);
        assert_eq!(params.block_size, 128);
        assert!(params.validate().is_ok());
    }
    
    #[test]
    fn test_size_calculations() {
        let params = LigeroParams::security_128();
        
        // Test block encoding size
        let block_enc = params.block_enc_size();
        assert_eq!(block_enc, 2 * 128 - 1 + 128 * 3); // 639
        
        // Test witness blocks calculation
        assert_eq!(params.num_witness_blocks(100), 1);
        assert_eq!(params.num_witness_blocks(128), 1);
        assert_eq!(params.num_witness_blocks(129), 2);
        assert_eq!(params.num_witness_blocks(1000), 8);
    }
    
    #[test]
    fn test_parameter_validation() {
        let mut params = LigeroParams::security_128();
        
        // Invalid block size
        params.block_size = 0;
        assert!(params.validate().is_err());
        
        params.block_size = 100; // Not power of 2
        assert!(params.validate().is_err());
        
        // Invalid extension factor
        params = LigeroParams::security_128();
        params.extension_factor = 1;
        assert!(params.validate().is_err());
        
        // Invalid blinding rows
        params = LigeroParams::security_128();
        params.num_blinding_rows = 2;
        assert!(params.validate().is_err());
    }
}