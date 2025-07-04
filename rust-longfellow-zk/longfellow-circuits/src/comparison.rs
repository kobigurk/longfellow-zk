/// Comparison circuits for zero-knowledge proofs

use crate::{CircuitBuilder, Constraint, gadgets, utils};
use longfellow_algebra::traits::Field;
use longfellow_core::Result;

/// Range proof circuit
pub struct RangeProofCircuit<F: Field, C: CircuitBuilder<F>> {
    circuit: C,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field, C: CircuitBuilder<F>> RangeProofCircuit<F, C> {
    /// Create a new range proof circuit
    pub fn new(circuit: C) -> Self {
        Self {
            circuit,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Prove that value is in range [0, 2^bits)
    pub fn prove_range(&mut self, value: usize, bits: usize) -> Result<()> {
        // Decompose into bits
        let _bit_vars = gadgets::bit_decompose(&mut self.circuit, value, bits)?;
        
        // Each bit is already constrained to be boolean
        // The decomposition ensures value = sum(bit_i * 2^i)
        Ok(())
    }
    
    /// Prove that value is in range [min, max]
    pub fn prove_interval(&mut self, value: usize, min: F, max: F) -> Result<()> {
        // value - min >= 0
        let value_minus_min = self.circuit.alloc_var();
        self.circuit.add_constraint(Constraint::Linear {
            coeffs: vec![(value, F::one()), (value_minus_min, -F::one())],
            constant: -min,
        })?;
        
        // max - value >= 0
        let max_minus_value = self.circuit.alloc_var();
        self.circuit.add_constraint(Constraint::Linear {
            coeffs: vec![(value, -F::one()), (max_minus_value, -F::one())],
            constant: max,
        })?;
        
        // Prove both differences are non-negative
        // This requires bit decomposition
        let bits_needed = 64; // Adjust based on field size
        gadgets::bit_decompose(&mut self.circuit, value_minus_min, bits_needed)?;
        gadgets::bit_decompose(&mut self.circuit, max_minus_value, bits_needed)?;
        
        Ok(())
    }
}

/// Comparison circuit for general comparisons
pub struct ComparisonCircuit<F: Field, C: CircuitBuilder<F>> {
    circuit: C,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field, C: CircuitBuilder<F>> ComparisonCircuit<F, C> {
    /// Create a new comparison circuit
    pub fn new(circuit: C) -> Self {
        Self {
            circuit,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Assert a < b
    pub fn assert_less_than(&mut self, a: usize, b: usize, bits: usize) -> Result<()> {
        let is_less = gadgets::less_than(&mut self.circuit, a, b, bits)?;
        
        // Assert is_less = 1
        self.circuit.add_constraint(Constraint::Linear {
            coeffs: vec![(is_less, F::one())],
            constant: -F::one(),
        })?;
        
        Ok(())
    }
    
    /// Assert a <= b
    pub fn assert_less_equal(&mut self, a: usize, b: usize, bits: usize) -> Result<()> {
        // a <= b is equivalent to !(b < a)
        let b_less_a = gadgets::less_than(&mut self.circuit, b, a, bits)?;
        let not_b_less_a = gadgets::not_gate(&mut self.circuit, b_less_a)?;
        
        // Assert not_b_less_a = 1
        self.circuit.add_constraint(Constraint::Linear {
            coeffs: vec![(not_b_less_a, F::one())],
            constant: -F::one(),
        })?;
        
        Ok(())
    }
    
    /// Assert a == b
    pub fn assert_equal(&mut self, a: usize, b: usize) -> Result<()> {
        utils::assert_equal(&mut self.circuit, a, b)
    }
    
    /// Assert a != b
    pub fn assert_not_equal(&mut self, a: usize, b: usize) -> Result<()> {
        // We need to show that a - b has a multiplicative inverse
        let diff = self.circuit.alloc_var();
        self.circuit.add_constraint(Constraint::Linear {
            coeffs: vec![(a, F::one()), (b, -F::one()), (diff, -F::one())],
            constant: F::zero(),
        })?;
        
        // diff * inv = 1
        let inv = self.circuit.alloc_var();
        let one = utils::const_gate(&mut self.circuit, F::one())?;
        self.circuit.add_constraint(Constraint::Quadratic { x: diff, y: inv, z: one })?;
        
        Ok(())
    }
    
    /// Compare field elements with carry propagation
    pub fn field_compare(&mut self, a: usize, b: usize) -> Result<usize> {
        // This is a simplified version
        // Real implementation would handle field arithmetic properly
        gadgets::less_than(&mut self.circuit, a, b, 128)
    }
}

/// Sorting circuit
pub struct SortingCircuit<F: Field, C: CircuitBuilder<F>> {
    circuit: C,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field, C: CircuitBuilder<F>> SortingCircuit<F, C> {
    /// Create a new sorting circuit
    pub fn new(circuit: C) -> Self {
        Self {
            circuit,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Assert that a list is sorted
    pub fn assert_sorted(&mut self, values: &[usize], bits: usize) -> Result<()> {
        for i in 1..values.len() {
            self.assert_monotonic(values[i-1], values[i], bits)?;
        }
        Ok(())
    }
    
    /// Assert a <= b (for sorted list)
    fn assert_monotonic(&mut self, a: usize, b: usize, bits: usize) -> Result<()> {
        // Directly implement less_equal constraint using self.circuit
        // b - a >= 0, so we need to prove b - a is non-negative
        
        let difference = self.circuit.alloc_var();
        self.circuit.add_constraint(Constraint::Linear {
            coeffs: vec![(b, F::one()), (a, -F::one()), (difference, -F::one())],
            constant: F::zero(),
        })?;
        
        // Prove difference is non-negative via bit decomposition
        gadgets::bit_decompose(&mut self.circuit, difference, bits)?;
        
        Ok(())
    }
    
    /// Prove permutation: output is a sorted permutation of input
    pub fn prove_permutation(
        &mut self,
        input: &[usize],
        output: &[usize],
        bits: usize,
    ) -> Result<()> {
        if input.len() != output.len() {
            return Err(longfellow_core::LongfellowError::InvalidParameter(
                "Input and output must have same length".to_string()
            ));
        }
        
        // Assert output is sorted
        self.assert_sorted(output, bits)?;
        
        // Assert same multiset (simplified - real version would use polynomial equality)
        // For now, just check sums are equal
        let input_sum = self.sum_values(input)?;
        let output_sum = self.sum_values(output)?;
        utils::assert_equal(&mut self.circuit, input_sum, output_sum)?;
        
        Ok(())
    }
    
    /// Compute sum of values
    fn sum_values(&mut self, values: &[usize]) -> Result<usize> {
        if values.is_empty() {
            return utils::const_gate(&mut self.circuit, F::zero());
        }
        
        let mut sum = values[0];
        for &val in &values[1..] {
            sum = utils::add_gate(&mut self.circuit, sum, val)?;
        }
        
        Ok(sum)
    }
}

/// Membership proof circuit
pub struct MembershipCircuit<F: Field, C: CircuitBuilder<F>> {
    circuit: C,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field, C: CircuitBuilder<F>> MembershipCircuit<F, C> {
    /// Create a new membership circuit
    pub fn new(circuit: C) -> Self {
        Self {
            circuit,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Prove value is in set
    pub fn prove_membership(&mut self, value: usize, set: &[usize]) -> Result<()> {
        if set.is_empty() {
            return Err(longfellow_core::LongfellowError::InvalidParameter(
                "Set cannot be empty".to_string()
            ));
        }
        
        // Create indicator variables for each element
        let indicators = self.circuit.alloc_vars(set.len());
        
        // Each indicator is boolean
        for &ind in &indicators {
            self.circuit.add_constraint(Constraint::Boolean { var: ind })?;
        }
        
        // Exactly one indicator is 1
        let mut sum = utils::const_gate(&mut self.circuit, F::zero())?;
        for &ind in &indicators {
            sum = utils::add_gate(&mut self.circuit, sum, ind)?;
        }
        let one = utils::const_gate(&mut self.circuit, F::one())?;
        utils::assert_equal(&mut self.circuit, sum, one)?;
        
        // value = sum(indicator[i] * set[i])
        let mut weighted_sum = utils::const_gate(&mut self.circuit, F::zero())?;
        for (i, &element) in set.iter().enumerate() {
            let product = utils::mul_gate(&mut self.circuit, indicators[i], element)?;
            weighted_sum = utils::add_gate(&mut self.circuit, weighted_sum, product)?;
        }
        
        utils::assert_equal(&mut self.circuit, value, weighted_sum)?;
        
        Ok(())
    }
    
    /// Prove value is NOT in set
    pub fn prove_non_membership(&mut self, value: usize, set: &[usize]) -> Result<()> {
        // For each element in set, prove value != element
        for &element in set {
            // Implement assert_not_equal inline
            // value != element means (value - element) * inverse = 1
            // where inverse is the multiplicative inverse of (value - element)
            
            let difference = self.circuit.alloc_var();
            let element_var = utils::const_gate(&mut self.circuit, F::from_u64(element as u64))?;
            self.circuit.add_constraint(Constraint::Linear {
                coeffs: vec![(value, F::one()), (element_var, -F::one()), (difference, -F::one())],
                constant: F::zero(),
            })?;
            
            // For non-zero difference, we need to prove it has an inverse
            let inverse = self.circuit.alloc_var();
            let product = utils::mul_gate(&mut self.circuit, difference, inverse)?;
            let one = utils::const_gate(&mut self.circuit, F::one())?;
            utils::assert_equal(&mut self.circuit, product, one)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StandardCircuit;
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_range_proof() {
        let circuit = StandardCircuit::<Fp128>::new();
        let mut range_proof = RangeProofCircuit::new(circuit);
        
        let value = range_proof.circuit.alloc_var();
        range_proof.prove_range(value, 8).unwrap();
    }
    
    #[test]
    fn test_comparison_circuit() {
        let circuit = StandardCircuit::<Fp128>::new();
        let mut comp = ComparisonCircuit::new(circuit);
        
        let a = comp.circuit.alloc_var();
        let b = comp.circuit.alloc_var();
        
        comp.assert_less_than(a, b, 8).unwrap();
    }
    
    #[test]
    fn test_sorting_circuit() {
        let circuit = StandardCircuit::<Fp128>::new();
        let mut sort = SortingCircuit::new(circuit);
        
        let values = vec![
            sort.circuit.alloc_var(),
            sort.circuit.alloc_var(),
            sort.circuit.alloc_var(),
        ];
        
        sort.assert_sorted(&values, 8).unwrap();
    }
    
    #[test]
    fn test_membership_proof() {
        let circuit = StandardCircuit::<Fp128>::new();
        let mut membership = MembershipCircuit::new(circuit);
        
        let value = membership.circuit.alloc_var();
        let set = vec![
            membership.circuit.alloc_var(),
            membership.circuit.alloc_var(),
            membership.circuit.alloc_var(),
        ];
        
        membership.prove_membership(value, &set).unwrap();
    }
}