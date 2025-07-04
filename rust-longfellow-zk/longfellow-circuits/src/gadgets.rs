/// Common circuit gadgets

use crate::{CircuitBuilder, Constraint, utils};
use longfellow_algebra::traits::Field;
use longfellow_core::Result;

/// Conditional selection: if cond then a else b
pub fn select<F: Field, C: CircuitBuilder<F>>(
    circuit: &mut C,
    cond: usize,
    a: usize,
    b: usize,
) -> Result<usize> {
    // result = cond * a + (1 - cond) * b
    // result = cond * (a - b) + b
    
    // Ensure cond is boolean
    circuit.add_constraint(Constraint::Boolean { var: cond })?;
    
    // Compute a - b
    let a_minus_b = circuit.alloc_var();
    circuit.add_constraint(Constraint::Linear {
        coeffs: vec![(a, F::one()), (b, -F::one()), (a_minus_b, -F::one())],
        constant: F::zero(),
    })?;
    
    // Compute cond * (a - b)
    let cond_times_diff = utils::mul_gate(circuit, cond, a_minus_b)?;
    
    // Compute result = cond * (a - b) + b
    utils::add_gate(circuit, cond_times_diff, b)
}

/// Bitwise decomposition of a field element
pub fn bit_decompose<F: Field, C: CircuitBuilder<F>>(
    circuit: &mut C,
    value: usize,
    num_bits: usize,
) -> Result<Vec<usize>> {
    let bits = circuit.alloc_vars(num_bits);
    
    // Constrain each bit to be boolean
    for &bit in &bits {
        circuit.add_constraint(Constraint::Boolean { var: bit })?;
    }
    
    // Constrain sum of bits * powers of 2 = value
    let mut coeffs = vec![];
    for (i, &bit) in bits.iter().enumerate() {
        coeffs.push((bit, F::from(1u64 << i)));
    }
    coeffs.push((value, -F::one()));
    
    circuit.add_constraint(Constraint::Linear {
        coeffs,
        constant: F::zero(),
    })?;
    
    Ok(bits)
}

/// Pack bits into a field element
pub fn bit_pack<F: Field, C: CircuitBuilder<F>>(
    circuit: &mut C,
    bits: &[usize],
) -> Result<usize> {
    let packed = circuit.alloc_var();
    
    let mut coeffs = vec![];
    for (i, &bit) in bits.iter().enumerate() {
        coeffs.push((bit, F::from(1u64 << i)));
    }
    coeffs.push((packed, -F::one()));
    
    circuit.add_constraint(Constraint::Linear {
        coeffs,
        constant: F::zero(),
    })?;
    
    Ok(packed)
}

/// Boolean AND gate
pub fn and_gate<F: Field, C: CircuitBuilder<F>>(
    circuit: &mut C,
    a: usize,
    b: usize,
) -> Result<usize> {
    // Ensure inputs are boolean
    circuit.add_constraint(Constraint::Boolean { var: a })?;
    circuit.add_constraint(Constraint::Boolean { var: b })?;
    
    // c = a * b (which is AND for booleans)
    utils::mul_gate(circuit, a, b)
}

/// Boolean OR gate
pub fn or_gate<F: Field, C: CircuitBuilder<F>>(
    circuit: &mut C,
    a: usize,
    b: usize,
) -> Result<usize> {
    // Ensure inputs are boolean
    circuit.add_constraint(Constraint::Boolean { var: a })?;
    circuit.add_constraint(Constraint::Boolean { var: b })?;
    
    // c = a + b - a*b
    let sum = utils::add_gate(circuit, a, b)?;
    let product = utils::mul_gate(circuit, a, b)?;
    
    let result = circuit.alloc_var();
    circuit.add_constraint(Constraint::Linear {
        coeffs: vec![(sum, F::one()), (product, -F::one()), (result, -F::one())],
        constant: F::zero(),
    })?;
    
    Ok(result)
}

/// Boolean NOT gate
pub fn not_gate<F: Field, C: CircuitBuilder<F>>(
    circuit: &mut C,
    a: usize,
) -> Result<usize> {
    // Ensure input is boolean
    circuit.add_constraint(Constraint::Boolean { var: a })?;
    
    // b = 1 - a
    let result = circuit.alloc_var();
    circuit.add_constraint(Constraint::Linear {
        coeffs: vec![(a, F::one()), (result, F::one())],
        constant: F::one(),
    })?;
    
    Ok(result)
}

/// XOR gate
pub fn xor_gate<F: Field, C: CircuitBuilder<F>>(
    circuit: &mut C,
    a: usize,
    b: usize,
) -> Result<usize> {
    // XOR = (a + b) - 2*a*b
    circuit.add_constraint(Constraint::Boolean { var: a })?;
    circuit.add_constraint(Constraint::Boolean { var: b })?;
    
    let sum = utils::add_gate(circuit, a, b)?;
    let product = utils::mul_gate(circuit, a, b)?;
    
    let double_product = circuit.alloc_var();
    circuit.add_constraint(Constraint::Linear {
        coeffs: vec![(product, F::from(2)), (double_product, -F::one())],
        constant: F::zero(),
    })?;
    
    let result = circuit.alloc_var();
    circuit.add_constraint(Constraint::Linear {
        coeffs: vec![(sum, F::one()), (double_product, -F::one()), (result, -F::one())],
        constant: F::zero(),
    })?;
    
    Ok(result)
}

/// Equality check
pub fn is_equal<F: Field, C: CircuitBuilder<F>>(
    circuit: &mut C,
    a: usize,
    b: usize,
) -> Result<usize> {
    // Check if a - b = 0
    // We use a trick: result = 1 if a = b, 0 otherwise
    
    let diff = circuit.alloc_var();
    circuit.add_constraint(Constraint::Linear {
        coeffs: vec![(a, F::one()), (b, -F::one()), (diff, -F::one())],
        constant: F::zero(),
    })?;
    
    // If diff = 0, result = 1
    // If diff != 0, result = 0
    // This requires an is_zero gadget which is more complex
    
    // For now, return a placeholder
    let result = circuit.alloc_var();
    circuit.add_constraint(Constraint::Boolean { var: result })?;
    
    Ok(result)
}

/// Less than comparison for small values
pub fn less_than<F: Field, C: CircuitBuilder<F>>(
    circuit: &mut C,
    a: usize,
    b: usize,
    num_bits: usize,
) -> Result<usize> {
    // Decompose both values into bits
    let a_bits = bit_decompose(circuit, a, num_bits)?;
    let b_bits = bit_decompose(circuit, b, num_bits)?;
    
    // Compare from MSB to LSB
    let mut result = utils::const_gate(circuit, F::zero())?;
    let mut prefix_equal = utils::const_gate(circuit, F::one())?;
    
    for i in (0..num_bits).rev() {
        // If prefix is equal and a[i] < b[i], then a < b
        let not_a_i = not_gate(circuit, a_bits[i])?;
        let a_less_i = and_gate(circuit, not_a_i, b_bits[i])?;
        let contrib = and_gate(circuit, prefix_equal, a_less_i)?;
        result = or_gate(circuit, result, contrib)?;
        
        // Update prefix_equal
        let bits_equal = xor_gate(circuit, a_bits[i], b_bits[i])?;
        let bits_equal = not_gate(circuit, bits_equal)?;
        prefix_equal = and_gate(circuit, prefix_equal, bits_equal)?;
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StandardCircuit;
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_select_gadget() {
        let mut circuit = StandardCircuit::<Fp128>::new();
        
        let cond = circuit.alloc_var();
        let a = circuit.alloc_var();
        let b = circuit.alloc_var();
        
        let result = select(&mut circuit, cond, a, b).unwrap();
        assert_eq!(result, 3); // Should be the 4th variable
    }
    
    #[test]
    fn test_bit_decompose() {
        let mut circuit = StandardCircuit::<Fp128>::new();
        
        let value = circuit.alloc_var();
        let bits = bit_decompose(&mut circuit, value, 8).unwrap();
        
        assert_eq!(bits.len(), 8);
    }
    
    #[test]
    fn test_boolean_gates() {
        let mut circuit = StandardCircuit::<Fp128>::new();
        
        let a = circuit.alloc_var();
        let b = circuit.alloc_var();
        
        let and_result = and_gate(&mut circuit, a, b).unwrap();
        let or_result = or_gate(&mut circuit, a, b).unwrap();
        let not_result = not_gate(&mut circuit, a).unwrap();
        let xor_result = xor_gate(&mut circuit, a, b).unwrap();
        
        assert!(and_result > b);
        assert!(or_result > and_result);
        assert!(not_result > or_result);
        assert!(xor_result > not_result);
    }
}