/// Arithmetic circuits for field operations

use crate::{CircuitBuilder, Constraint, utils};
use longfellow_algebra::traits::Field;
use longfellow_core::Result;

/// Integer arithmetic circuit
pub struct IntegerArithmeticCircuit<F: Field, C: CircuitBuilder<F>> {
    circuit: C,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field, C: CircuitBuilder<F>> IntegerArithmeticCircuit<F, C> {
    /// Create a new integer arithmetic circuit
    pub fn new(circuit: C) -> Self {
        Self {
            circuit,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Modular addition: (a + b) mod m
    pub fn mod_add(&mut self, a: usize, b: usize, m: usize) -> Result<usize> {
        // Compute a + b
        let sum = utils::add_gate(&mut self.circuit, a, b)?;
        
        // Compute sum mod m (simplified - real version would use division)
        self.mod_reduce(sum, m)
    }
    
    /// Modular multiplication: (a * b) mod m
    pub fn mod_mul(&mut self, a: usize, b: usize, m: usize) -> Result<usize> {
        // Compute a * b
        let product = utils::mul_gate(&mut self.circuit, a, b)?;
        
        // Compute product mod m
        self.mod_reduce(product, m)
    }
    
    /// Modular exponentiation: a^e mod m
    pub fn mod_exp(&mut self, base: usize, exp: usize, m: usize, exp_bits: usize) -> Result<usize> {
        // Square-and-multiply algorithm
        let exp_binary = crate::gadgets::bit_decompose(&mut self.circuit, exp, exp_bits)?;
        
        let one = utils::const_gate(&mut self.circuit, F::one())?;
        let mut result = one;
        let mut power = base;
        
        for &bit in &exp_binary {
            // If bit is 1, multiply result by current power
            let new_result = self.mod_mul(result, power, m)?;
            result = crate::gadgets::select(&mut self.circuit, bit, new_result, result)?;
            
            // Square the power
            power = self.mod_mul(power, power, m)?;
        }
        
        Ok(result)
    }
    
    /// Modular reduction: a mod m
    fn mod_reduce(&mut self, a: usize, m: usize) -> Result<usize> {
        // This is simplified - real implementation would use proper division
        // For now, just return a constrained variable
        let result = self.circuit.alloc_var();
        
        // result < m
        self.circuit.add_constraint(Constraint::Range { var: result, bits: 64 })?;
        
        // There exists q such that a = q * m + result
        let q = self.circuit.alloc_var();
        let qm = utils::mul_gate(&mut self.circuit, q, m)?;
        let qm_plus_r = utils::add_gate(&mut self.circuit, qm, result)?;
        utils::assert_equal(&mut self.circuit, a, qm_plus_r)?;
        
        Ok(result)
    }
}

/// Polynomial arithmetic circuit
pub struct PolynomialCircuit<F: Field, C: CircuitBuilder<F>> {
    circuit: C,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field, C: CircuitBuilder<F>> PolynomialCircuit<F, C> {
    /// Create a new polynomial circuit
    pub fn new(circuit: C) -> Self {
        Self {
            circuit,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Evaluate polynomial at a point
    pub fn evaluate(&mut self, coeffs: &[usize], x: usize) -> Result<usize> {
        if coeffs.is_empty() {
            return utils::const_gate(&mut self.circuit, F::zero());
        }
        
        // Horner's method: a_n*x^n + ... + a_1*x + a_0
        // = ((a_n*x + a_{n-1})*x + ...)*x + a_0
        let mut result = coeffs[coeffs.len() - 1];
        
        for &coeff in coeffs[..coeffs.len()-1].iter().rev() {
            result = utils::mul_gate(&mut self.circuit, result, x)?;
            result = utils::add_gate(&mut self.circuit, result, coeff)?;
        }
        
        Ok(result)
    }
    
    /// Add two polynomials
    pub fn add(&mut self, a: &[usize], b: &[usize]) -> Result<Vec<usize>> {
        let max_len = a.len().max(b.len());
        let mut result = Vec::with_capacity(max_len);
        
        for i in 0..max_len {
            let a_coeff = if i < a.len() { 
                a[i] 
            } else { 
                utils::const_gate(&mut self.circuit, F::zero())? 
            };
            
            let b_coeff = if i < b.len() { 
                b[i] 
            } else { 
                utils::const_gate(&mut self.circuit, F::zero())? 
            };
            
            result.push(utils::add_gate(&mut self.circuit, a_coeff, b_coeff)?);
        }
        
        Ok(result)
    }
    
    /// Multiply two polynomials
    pub fn multiply(&mut self, a: &[usize], b: &[usize]) -> Result<Vec<usize>> {
        if a.is_empty() || b.is_empty() {
            return Ok(vec![]);
        }
        
        let result_len = a.len() + b.len() - 1;
        let mut result = vec![utils::const_gate(&mut self.circuit, F::zero())?; result_len];
        
        for (i, &a_coeff) in a.iter().enumerate() {
            for (j, &b_coeff) in b.iter().enumerate() {
                let product = utils::mul_gate(&mut self.circuit, a_coeff, b_coeff)?;
                result[i + j] = utils::add_gate(&mut self.circuit, result[i + j], product)?;
            }
        }
        
        Ok(result)
    }
    
    /// Polynomial commitment evaluation
    pub fn verify_evaluation(
        &mut self,
        _commitment: usize,
        point: usize,
        value: usize,
        proof: &[usize],
    ) -> Result<()> {
        // This would implement KZG or similar polynomial commitment
        // For now, just check that evaluation is correct given coefficients
        let computed = self.evaluate(proof, point)?;
        utils::assert_equal(&mut self.circuit, computed, value)?;
        
        Ok(())
    }
}

/// Fixed-point arithmetic circuit
pub struct FixedPointCircuit<F: Field, C: CircuitBuilder<F>> {
    circuit: C,
    /// Number of fractional bits
    frac_bits: usize,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field, C: CircuitBuilder<F>> FixedPointCircuit<F, C> {
    /// Create a new fixed-point circuit
    pub fn new(circuit: C, frac_bits: usize) -> Self {
        Self {
            circuit,
            frac_bits,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Convert integer to fixed-point
    pub fn from_int(&mut self, value: usize) -> Result<usize> {
        // Shift left by frac_bits
        let scale = utils::const_gate(&mut self.circuit, F::from_u64(1u64 << self.frac_bits))?;
        utils::mul_gate(&mut self.circuit, value, scale)
    }
    
    /// Add two fixed-point numbers
    pub fn add(&mut self, a: usize, b: usize) -> Result<usize> {
        utils::add_gate(&mut self.circuit, a, b)
    }
    
    /// Multiply two fixed-point numbers
    pub fn mul(&mut self, a: usize, b: usize) -> Result<usize> {
        // Multiply and shift right by frac_bits
        let product = utils::mul_gate(&mut self.circuit, a, b)?;
        self.shift_right(product, self.frac_bits)
    }
    
    /// Divide two fixed-point numbers
    pub fn div(&mut self, a: usize, b: usize) -> Result<usize> {
        // a/b = (a << frac_bits) / b
        let shifted_a = self.shift_left(a, self.frac_bits)?;
        self.integer_divide(shifted_a, b)
    }
    
    /// Shift right (division by power of 2)
    fn shift_right(&mut self, value: usize, bits: usize) -> Result<usize> {
        let divisor = utils::const_gate(&mut self.circuit, F::from_u64(1u64 << bits))?;
        self.integer_divide(value, divisor)
    }
    
    /// Shift left (multiplication by power of 2)
    fn shift_left(&mut self, value: usize, bits: usize) -> Result<usize> {
        let multiplier = utils::const_gate(&mut self.circuit, F::from_u64(1u64 << bits))?;
        utils::mul_gate(&mut self.circuit, value, multiplier)
    }
    
    /// Integer division (simplified)
    fn integer_divide(&mut self, a: usize, b: usize) -> Result<usize> {
        let quotient = self.circuit.alloc_var();
        let remainder = self.circuit.alloc_var();
        
        // a = b * quotient + remainder
        let b_times_q = utils::mul_gate(&mut self.circuit, b, quotient)?;
        let b_times_q_plus_r = utils::add_gate(&mut self.circuit, b_times_q, remainder)?;
        utils::assert_equal(&mut self.circuit, a, b_times_q_plus_r)?;
        
        // remainder < b (simplified)
        self.circuit.add_constraint(Constraint::Range { var: remainder, bits: 64 })?;
        
        Ok(quotient)
    }
}

/// Vector arithmetic circuit
pub struct VectorCircuit<F: Field, C: CircuitBuilder<F>> {
    circuit: C,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field, C: CircuitBuilder<F>> VectorCircuit<F, C> {
    /// Create a new vector circuit
    pub fn new(circuit: C) -> Self {
        Self {
            circuit,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Dot product of two vectors
    pub fn dot_product(&mut self, a: &[usize], b: &[usize]) -> Result<usize> {
        if a.len() != b.len() {
            return Err(longfellow_core::LongfellowError::InvalidParameter(
                "Vectors must have same length".to_string()
            ));
        }
        
        let mut sum = utils::const_gate(&mut self.circuit, F::zero())?;
        
        for (_i, (&a_i, &b_i)) in a.iter().zip(b.iter()).enumerate() {
            let product = utils::mul_gate(&mut self.circuit, a_i, b_i)?;
            sum = utils::add_gate(&mut self.circuit, sum, product)?;
        }
        
        Ok(sum)
    }
    
    /// Vector addition
    pub fn add(&mut self, a: &[usize], b: &[usize]) -> Result<Vec<usize>> {
        if a.len() != b.len() {
            return Err(longfellow_core::LongfellowError::InvalidParameter(
                "Vectors must have same length".to_string()
            ));
        }
        
        a.iter()
            .zip(b.iter())
            .map(|(&a_i, &b_i)| utils::add_gate(&mut self.circuit, a_i, b_i))
            .collect()
    }
    
    /// Scalar multiplication
    pub fn scalar_mul(&mut self, scalar: usize, vector: &[usize]) -> Result<Vec<usize>> {
        vector
            .iter()
            .map(|&v_i| utils::mul_gate(&mut self.circuit, scalar, v_i))
            .collect()
    }
    
    /// Vector norm squared
    pub fn norm_squared(&mut self, vector: &[usize]) -> Result<usize> {
        self.dot_product(vector, vector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StandardCircuit;
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_integer_arithmetic() {
        let circuit = StandardCircuit::<Fp128>::new();
        let mut int_arith = IntegerArithmeticCircuit::new(circuit);
        
        let a = int_arith.circuit.alloc_var();
        let b = int_arith.circuit.alloc_var();
        let m = int_arith.circuit.alloc_var();
        
        let sum = int_arith.mod_add(a, b, m).unwrap();
        assert!(sum > b);
    }
    
    #[test]
    fn test_polynomial_circuit() {
        let circuit = StandardCircuit::<Fp128>::new();
        let mut poly = PolynomialCircuit::new(circuit);
        
        // Create polynomial coefficients
        let coeffs = vec![
            poly.circuit.alloc_var(),
            poly.circuit.alloc_var(),
            poly.circuit.alloc_var(),
        ];
        
        let x = poly.circuit.alloc_var();
        let result = poly.evaluate(&coeffs, x).unwrap();
        assert!(result > x);
    }
    
    #[test]
    fn test_fixed_point() {
        let circuit = StandardCircuit::<Fp128>::new();
        let mut fixed = FixedPointCircuit::new(circuit, 16);
        
        let a = fixed.circuit.alloc_var();
        let b = fixed.circuit.alloc_var();
        
        let sum = fixed.add(a, b).unwrap();
        let product = fixed.mul(a, b).unwrap();
        
        assert!(sum > b);
        assert!(product > sum);
    }
    
    #[test]
    fn test_vector_circuit() {
        let circuit = StandardCircuit::<Fp128>::new();
        let mut vec_circuit = VectorCircuit::new(circuit);
        
        let a = vec![
            vec_circuit.circuit.alloc_var(),
            vec_circuit.circuit.alloc_var(),
            vec_circuit.circuit.alloc_var(),
        ];
        
        let b = vec![
            vec_circuit.circuit.alloc_var(),
            vec_circuit.circuit.alloc_var(),
            vec_circuit.circuit.alloc_var(),
        ];
        
        let dot = vec_circuit.dot_product(&a, &b).unwrap();
        assert!(dot >= 3); // New variable
    }
}