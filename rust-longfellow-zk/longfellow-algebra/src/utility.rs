use crate::traits::Field;
use longfellow_core::{LongfellowError, Result};
use rayon::prelude::*;

/// Compute batch inverses for arithmetic sequence 1, 2, ..., m-1
/// Returns array where inverses[i] = 1/i for i=1..m-1, inverses[0] = 0
pub fn batch_inverse_arithmetic<F: Field>(m: usize) -> Result<Vec<F>> {
    if m == 0 {
        return Ok(vec![]);
    }
    
    let mut inverses = vec![F::zero(); m];
    
    if m == 1 {
        return Ok(inverses);
    }
    
    // Compute cumulative products: products[i] = 1 * 2 * ... * i
    let mut products = vec![F::one(); m];
    for i in 2..m {
        products[i] = products[i - 1] * F::from_u64(i as u64);
    }
    
    // Invert the final product
    let final_inv = products[m - 1].invert()
        .ok_or_else(|| LongfellowError::ArithmeticError(
            "Failed to invert product".to_string()
        ))?;
    
    // Work backwards to compute individual inverses
    let mut running_inv = final_inv;
    for i in (1..m).rev() {
        if i == 1 {
            inverses[1] = running_inv;
        } else {
            inverses[i] = running_inv * products[i - 1];
            running_inv *= F::from_u64(i as u64);
        }
    }
    
    Ok(inverses)
}

/// Batch inversion of arbitrary field elements
/// Returns None for zero elements
pub fn batch_inverse<F: Field>(elements: &[F]) -> Result<Vec<Option<F>>> {
    let n = elements.len();
    if n == 0 {
        return Ok(vec![]);
    }
    
    let mut result = vec![None; n];
    let mut products = Vec::with_capacity(n);
    let mut acc = F::one();
    
    // Track non-zero elements and compute cumulative products
    let mut non_zero_indices = Vec::new();
    
    for (i, &elem) in elements.iter().enumerate() {
        if elem != F::zero() {
            products.push(acc);
            acc *= elem;
            non_zero_indices.push(i);
        }
    }
    
    if non_zero_indices.is_empty() {
        return Ok(result);
    }
    
    // Invert accumulated product
    let inv = acc.invert()
        .ok_or_else(|| LongfellowError::ArithmeticError(
            "Failed to invert accumulated product".to_string()
        ))?;
    
    // Work backwards to compute individual inverses
    let mut running_inv = inv;
    for (j, &i) in non_zero_indices.iter().enumerate().rev() {
        result[i] = Some(running_inv * products[j]);
        running_inv *= elements[i];
    }
    
    Ok(result)
}

/// Compute binomial coefficient (n choose k) mod field characteristic
pub fn binomial_coefficient<F: Field>(n: usize, k: usize) -> Result<F> {
    if k > n {
        return Ok(F::zero());
    }
    
    if k == 0 || k == n {
        return Ok(F::one());
    }
    
    // Use symmetry to reduce computation
    let k = k.min(n - k);
    
    // Compute using multiplicative formula
    let mut result = F::one();
    for i in 0..k {
        result *= F::from_u64((n - i) as u64);
        result *= F::from_u64((i + 1) as u64).invert()
            .ok_or_else(|| LongfellowError::ArithmeticError(
                format!("Failed to invert {}", i + 1)
            ))?;
    }
    
    Ok(result)
}

/// Precompute binomial coefficients (n choose k) for k=0..=n
pub fn precompute_binomial_row<F: Field>(n: usize) -> Result<Vec<F>> {
    let mut row = vec![F::zero(); n + 1];
    row[0] = F::one();
    
    for i in 1..=n {
        // Use Pascal's triangle recurrence
        for j in (1..=i).rev() {
            row[j] = row[j] + row[j - 1];
        }
    }
    
    Ok(row)
}

/// Parallel batch operations for large arrays
pub struct ParallelBatchOps;

impl ParallelBatchOps {
    /// Parallel batch inverse with configurable chunk size
    pub fn batch_inverse_parallel<F: Field + Send + Sync>(
        elements: &[F],
        chunk_size: usize,
    ) -> Result<Vec<Option<F>>> {
        if elements.len() <= chunk_size {
            return batch_inverse(elements);
        }
        
        // Process chunks in parallel
        let results: Result<Vec<_>> = elements
            .par_chunks(chunk_size)
            .map(batch_inverse)
            .collect();
        
        // Flatten results
        Ok(results?.into_iter().flatten().collect())
    }
    
    /// Parallel computation of arithmetic sequence inverses
    pub fn batch_inverse_arithmetic_parallel<F: Field + Send + Sync>(
        m: usize,
        num_threads: usize,
    ) -> Result<Vec<F>> {
        if m <= 1000 || num_threads <= 1 {
            return batch_inverse_arithmetic(m);
        }
        
        let chunk_size = (m + num_threads - 1) / num_threads;
        let mut results = vec![F::zero(); m];
        
        // Compute products of chunks in parallel
        let chunk_products: Vec<F> = (0..num_threads)
            .into_par_iter()
            .map(|i| {
                let start = i * chunk_size + 1;
                let end = ((i + 1) * chunk_size).min(m);
                
                let mut product = F::one();
                for j in start..end {
                    product *= F::from_u64(j as u64);
                }
                product
            })
            .collect();
        
        // Compute cumulative products
        let mut prefix_products = vec![F::one(); num_threads];
        for i in 1..num_threads {
            prefix_products[i] = prefix_products[i - 1] * chunk_products[i - 1];
        }
        
        // Invert final product
        let final_product = prefix_products[num_threads - 1] * chunk_products[num_threads - 1];
        let final_inv = final_product.invert()
            .ok_or_else(|| LongfellowError::ArithmeticError(
                "Failed to invert final product".to_string()
            ))?;
        
        // Process chunks in parallel
        results.par_chunks_mut(chunk_size)
            .enumerate()
            .for_each(|(i, chunk)| {
                let start = i * chunk_size + 1;
                let end = start + chunk.len();
                
                // Compute suffix product inverse for this chunk
                let mut suffix_inv = final_inv;
                for j in (i + 1)..num_threads {
                    suffix_inv *= chunk_products[j];
                }
                
                // Compute inverses within chunk
                let mut running_inv = suffix_inv * prefix_products[i];
                for j in (start..end).rev() {
                    if j - start < chunk.len() {
                        chunk[j - start] = running_inv;
                    }
                    if j > start {
                        running_inv *= F::from_u64(j as u64);
                    }
                }
            });
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::fp128::Fp128;
    
    #[test]
    fn test_batch_inverse_arithmetic() {
        let m = 10;
        let inverses = batch_inverse_arithmetic::<Fp128>(m).unwrap();
        
        assert_eq!(inverses[0], Fp128::zero());
        
        for i in 1..m {
            let expected = Fp128::from_u64(i as u64).invert().unwrap();
            assert_eq!(inverses[i], expected, "Inverse of {} incorrect", i);
        }
    }
    
    #[test]
    fn test_batch_inverse() {
        let elements = vec![
            Fp128::from_u64(2),
            Fp128::from_u64(3),
            Fp128::zero(),
            Fp128::from_u64(5),
        ];
        
        let inverses = batch_inverse(&elements).unwrap();
        
        assert_eq!(inverses[0], Some(Fp128::from_u64(2).invert().unwrap()));
        assert_eq!(inverses[1], Some(Fp128::from_u64(3).invert().unwrap()));
        assert_eq!(inverses[2], None);
        assert_eq!(inverses[3], Some(Fp128::from_u64(5).invert().unwrap()));
    }
    
    #[test]
    fn test_binomial_coefficient() {
        // Test (5 choose 2) = 10
        let result = binomial_coefficient::<Fp128>(5, 2).unwrap();
        assert_eq!(result, Fp128::from_u64(10));
        
        // Test (10 choose 0) = 1
        let result = binomial_coefficient::<Fp128>(10, 0).unwrap();
        assert_eq!(result, Fp128::one());
        
        // Test (7 choose 7) = 1
        let result = binomial_coefficient::<Fp128>(7, 7).unwrap();
        assert_eq!(result, Fp128::one());
    }
    
    #[test]
    fn test_precompute_binomial_row() {
        let n = 5;
        let row = precompute_binomial_row::<Fp128>(n).unwrap();
        
        // Row 5 of Pascal's triangle: [1, 5, 10, 10, 5, 1]
        assert_eq!(row[0], Fp128::from_u64(1));
        assert_eq!(row[1], Fp128::from_u64(5));
        assert_eq!(row[2], Fp128::from_u64(10));
        assert_eq!(row[3], Fp128::from_u64(10));
        assert_eq!(row[4], Fp128::from_u64(5));
        assert_eq!(row[5], Fp128::from_u64(1));
    }
}