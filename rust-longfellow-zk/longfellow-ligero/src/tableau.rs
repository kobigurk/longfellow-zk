/// Ligero tableau data structure
/// 
/// The tableau is a 2D array where each row represents encoded data
/// and columns are committed using Merkle trees.

use longfellow_algebra::traits::Field;
use longfellow_algebra::fft::FFT;
use longfellow_algebra::polynomial::DensePolynomial;
use longfellow_core::{LongfellowError, Result};
use longfellow_random::FieldRng;
use rand::{CryptoRng, RngCore};
use rayon::prelude::*;
use crate::parameters::{LigeroParams, row_indices};

/// Ligero tableau storing encoded rows
pub struct Tableau<F: Field> {
    /// Parameters
    params: LigeroParams,
    
    /// Tableau data (row-major order)
    data: Vec<Vec<F>>,
    
    /// Number of rows
    height: usize,
    
    /// Row width (encoded size)
    width: usize,
}

impl<F: Field> Tableau<F> {
    /// Create a new tableau
    pub fn new(params: LigeroParams, height: usize) -> Self {
        let width = params.block_enc_size();
        let data = vec![vec![F::zero(); width]; height];
        
        Self {
            params,
            data,
            height,
            width,
        }
    }
    
    /// Get tableau dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.height, self.width)
    }
    
    /// Access a row
    pub fn row(&self, i: usize) -> &[F] {
        &self.data[i]
    }
    
    /// Access a mutable row
    pub fn row_mut(&mut self, i: usize) -> &mut [F] {
        &mut self.data[i]
    }
    
    /// Get a column
    pub fn column(&self, j: usize) -> Vec<F> {
        (0..self.height).map(|i| self.data[i][j]).collect()
    }
    
    /// Set a value
    pub fn set(&mut self, row: usize, col: usize, value: F) {
        self.data[row][col] = value;
    }
    
    /// Get a value
    pub fn get(&self, row: usize, col: usize) -> F {
        self.data[row][col]
    }
    
    /// Fill blinding rows with random values
    pub fn randomize_blinding_rows<R: RngCore + CryptoRng>(&mut self, rng: &mut R) -> Result<()> {
        let mut field_rng = FieldRng::<F, _>::new(rng);
        
        // Fill the three blinding rows
        for row_idx in 0..self.params.num_blinding_rows {
            let row = self.row_mut(row_idx);
            for j in 0..self.params.block_size {
                row[j] = field_rng.random_field_element();
            }
        }
        
        Ok(())
    }
    
    /// Layout witnesses in the tableau with random blinding
    pub fn layout_witnesses<R: RngCore + CryptoRng>(
        &mut self,
        witnesses: &[F],
        rng: &mut R,
    ) -> Result<()> {
        let mut field_rng = FieldRng::<F, _>::new(rng);
        let num_blocks = self.params.num_witness_blocks(witnesses.len());
        
        for block_idx in 0..num_blocks {
            let row_idx = row_indices::WITNESS_START + block_idx;
            let row = self.row_mut(row_idx);
            
            // Fill witness values
            let start = block_idx * self.params.block_size;
            let end = std::cmp::min(start + self.params.block_size, witnesses.len());
            
            for (j, w_idx) in (start..end).enumerate() {
                row[j] = witnesses[w_idx];
            }
            
            // Add random padding if needed
            for j in (end - start)..self.params.block_size {
                row[j] = field_rng.random_field_element();
            }
        }
        
        Ok(())
    }
    
    /// Encode all rows using Reed-Solomon encoding
    pub fn encode_rows(&mut self) -> Result<()> {
        // Get FFT domain for encoding
        let domain_size = self.params.block_enc_size();
        let fft = FFT::<F>::new(domain_size)?;
        
        // Encode each row in parallel
        self.data.par_iter_mut().for_each(|row| {
            encode_row(&self.params, row, &fft);
        });
        
        Ok(())
    }
    
    /// Encode quadratic constraints
    pub fn encode_quadratic_constraints(
        &mut self,
        constraints: &[(usize, usize, usize)],
        witnesses: &[F],
        witness_row_start: usize,
    ) -> Result<()> {
        let block_size = self.params.block_size;
        let num_quad_rows = self.params.num_quadratic_rows(constraints.len());
        
        for row_idx in 0..num_quad_rows {
            let quad_row_idx = witness_row_start + row_idx;
            let row = self.row_mut(quad_row_idx);
            
            let start = row_idx * block_size;
            let end = std::cmp::min(start + block_size, constraints.len());
            
            for (j, c_idx) in (start..end).enumerate() {
                let (x, y, z) = constraints[c_idx];
                // Encode w[x] * w[y] - w[z]
                row[j] = witnesses[x] * witnesses[y] - witnesses[z];
            }
            
            // Zero padding
            for j in (end - start)..block_size {
                row[j] = F::zero();
            }
        }
        
        Ok(())
    }
}

/// Encode a single row using Reed-Solomon
fn encode_row<F: Field>(params: &LigeroParams, row: &mut [F], fft: &FFT<F>) {
    let block_size = params.block_size;
    let block_enc_size = params.block_enc_size();
    
    // Extract the block values
    let mut block_values = vec![F::zero(); block_size];
    block_values.copy_from_slice(&row[..block_size]);
    
    // Interpolate polynomial
    let poly = DensePolynomial::interpolate_fft(&block_values, fft).unwrap();
    
    // Evaluate at encoding points
    let mut encoded = vec![F::zero(); block_enc_size];
    
    // First part: degree doubling (evaluate at roots of unity)
    for i in 0..2 * block_size - 1 {
        let point = fft.get_root_of_unity(i);
        encoded[i] = poly.evaluate(&point);
    }
    
    // Second part: extension (evaluate at additional points)
    let ext_start = 2 * block_size - 1;
    for i in 0..params.block_ext_size() {
        // Use systematic encoding points outside the FFT domain
        let point = F::from((block_enc_size + i) as u64);
        encoded[ext_start + i] = poly.evaluate(&point);
    }
    
    // Copy back to row
    row.copy_from_slice(&encoded);
}

/// Helper to compute linear combination of rows
pub fn linear_combination<F: Field>(rows: &[Vec<F>], coeffs: &[F]) -> Result<Vec<F>> {
    if rows.is_empty() || rows.len() != coeffs.len() {
        return Err(LongfellowError::InvalidParameter(
            "Invalid dimensions for linear combination".to_string()
        ));
    }
    
    let row_len = rows[0].len();
    let mut result = vec![F::zero(); row_len];
    
    for (row, &coeff) in rows.iter().zip(coeffs.iter()) {
        for (j, &val) in row.iter().enumerate() {
            result[j] += coeff * val;
        }
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use longfellow_algebra::Fp128;
    use rand::rngs::OsRng;
    
    #[test]
    fn test_tableau_creation() {
        let params = LigeroParams::security_128();
        let tableau = Tableau::<Fp128>::new(params.clone(), 10);
        
        assert_eq!(tableau.dimensions(), (10, params.block_enc_size()));
    }
    
    #[test]
    fn test_witness_layout() {
        let params = LigeroParams::security_128();
        let mut tableau = Tableau::<Fp128>::new(params.clone(), 10);
        
        // Create test witnesses
        let witnesses: Vec<Fp128> = (0..200)
            .map(|i| Fp128::from(i as u64))
            .collect();
        
        tableau.layout_witnesses(&witnesses, &mut OsRng).unwrap();
        
        // Check first witness values are correctly placed
        assert_eq!(tableau.get(row_indices::WITNESS_START, 0), Fp128::from(0));
        assert_eq!(tableau.get(row_indices::WITNESS_START, 1), Fp128::from(1));
        assert_eq!(tableau.get(row_indices::WITNESS_START + 1, 0), Fp128::from(128));
    }
    
    #[test]
    fn test_row_encoding() {
        let params = LigeroParams {
            block_size: 8,
            extension_factor: 2,
            ..LigeroParams::security_80()
        };
        
        let mut tableau = Tableau::<Fp128>::new(params.clone(), 5);
        
        // Set some values in first row
        for i in 0..8 {
            tableau.set(0, i, Fp128::from(i as u64));
        }
        
        // Encode rows
        tableau.encode_rows().unwrap();
        
        // Check that encoding modified the row
        let row = tableau.row(0);
        assert_eq!(row.len(), params.block_enc_size());
        
        // First block_size values should be unchanged
        for i in 0..8 {
            assert_eq!(row[i], Fp128::from(i as u64));
        }
    }
}