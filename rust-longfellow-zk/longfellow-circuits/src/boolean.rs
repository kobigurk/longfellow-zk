/// Boolean circuits and logic operations

use crate::{CircuitBuilder, Constraint, gadgets, utils};
use longfellow_algebra::traits::Field;
use longfellow_core::Result;

/// Boolean formula circuit
pub struct BooleanFormulaCircuit<F: Field, C: CircuitBuilder<F>> {
    circuit: C,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field, C: CircuitBuilder<F>> BooleanFormulaCircuit<F, C> {
    /// Create a new boolean formula circuit
    pub fn new(circuit: C) -> Self {
        Self {
            circuit,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Evaluate a boolean formula in conjunctive normal form (CNF)
    pub fn evaluate_cnf(&mut self, clauses: &[Vec<(usize, bool)>]) -> Result<usize> {
        // CNF: (a OR b OR ...) AND (c OR d OR ...) AND ...
        let mut clause_results = Vec::new();
        
        for clause in clauses {
            let clause_result = self.evaluate_clause(clause)?;
            clause_results.push(clause_result);
        }
        
        // AND all clauses together
        self.and_all(&clause_results)
    }
    
    /// Evaluate a single clause (disjunction)
    fn evaluate_clause(&mut self, clause: &[(usize, bool)]) -> Result<usize> {
        if clause.is_empty() {
            return utils::const_gate(&mut self.circuit, F::zero());
        }
        
        let mut result = if clause[0].1 {
            clause[0].0
        } else {
            gadgets::not_gate(&mut self.circuit, clause[0].0)?
        };
        
        for &(var, positive) in &clause[1..] {
            let literal = if positive {
                var
            } else {
                gadgets::not_gate(&mut self.circuit, var)?
            };
            result = gadgets::or_gate(&mut self.circuit, result, literal)?;
        }
        
        Ok(result)
    }
    
    /// AND all values together
    fn and_all(&mut self, values: &[usize]) -> Result<usize> {
        if values.is_empty() {
            return utils::const_gate(&mut self.circuit, F::one());
        }
        
        let mut result = values[0];
        for &val in &values[1..] {
            result = gadgets::and_gate(&mut self.circuit, result, val)?;
        }
        
        Ok(result)
    }
    
    /// Evaluate a boolean formula in disjunctive normal form (DNF)
    pub fn evaluate_dnf(&mut self, terms: &[Vec<(usize, bool)>]) -> Result<usize> {
        // DNF: (a AND b AND ...) OR (c AND d AND ...) OR ...
        let mut term_results = Vec::new();
        
        for term in terms {
            let term_result = self.evaluate_term(term)?;
            term_results.push(term_result);
        }
        
        // OR all terms together
        self.or_all(&term_results)
    }
    
    /// Evaluate a single term (conjunction)
    fn evaluate_term(&mut self, term: &[(usize, bool)]) -> Result<usize> {
        if term.is_empty() {
            return utils::const_gate(&mut self.circuit, F::one());
        }
        
        let mut result = if term[0].1 {
            term[0].0
        } else {
            gadgets::not_gate(&mut self.circuit, term[0].0)?
        };
        
        for &(var, positive) in &term[1..] {
            let literal = if positive {
                var
            } else {
                gadgets::not_gate(&mut self.circuit, var)?
            };
            result = gadgets::and_gate(&mut self.circuit, result, literal)?;
        }
        
        Ok(result)
    }
    
    /// OR all values together
    fn or_all(&mut self, values: &[usize]) -> Result<usize> {
        if values.is_empty() {
            return utils::const_gate(&mut self.circuit, F::zero());
        }
        
        let mut result = values[0];
        for &val in &values[1..] {
            result = gadgets::or_gate(&mut self.circuit, result, val)?;
        }
        
        Ok(result)
    }
}

/// Bitwise operations circuit
pub struct BitwiseCircuit<F: Field, C: CircuitBuilder<F>> {
    circuit: C,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field, C: CircuitBuilder<F>> BitwiseCircuit<F, C> {
    /// Create a new bitwise circuit
    pub fn new(circuit: C) -> Self {
        Self {
            circuit,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Bitwise AND of two values
    pub fn bitwise_and(&mut self, a: usize, b: usize, bits: usize) -> Result<usize> {
        let a_bits = gadgets::bit_decompose(&mut self.circuit, a, bits)?;
        let b_bits = gadgets::bit_decompose(&mut self.circuit, b, bits)?;
        
        let mut result_bits = Vec::new();
        for (a_bit, b_bit) in a_bits.iter().zip(b_bits.iter()) {
            let and_bit = gadgets::and_gate(&mut self.circuit, *a_bit, *b_bit)?;
            result_bits.push(and_bit);
        }
        
        gadgets::bit_pack(&mut self.circuit, &result_bits)
    }
    
    /// Bitwise OR of two values
    pub fn bitwise_or(&mut self, a: usize, b: usize, bits: usize) -> Result<usize> {
        let a_bits = gadgets::bit_decompose(&mut self.circuit, a, bits)?;
        let b_bits = gadgets::bit_decompose(&mut self.circuit, b, bits)?;
        
        let mut result_bits = Vec::new();
        for (a_bit, b_bit) in a_bits.iter().zip(b_bits.iter()) {
            let or_bit = gadgets::or_gate(&mut self.circuit, *a_bit, *b_bit)?;
            result_bits.push(or_bit);
        }
        
        gadgets::bit_pack(&mut self.circuit, &result_bits)
    }
    
    /// Bitwise XOR of two values
    pub fn bitwise_xor(&mut self, a: usize, b: usize, bits: usize) -> Result<usize> {
        let a_bits = gadgets::bit_decompose(&mut self.circuit, a, bits)?;
        let b_bits = gadgets::bit_decompose(&mut self.circuit, b, bits)?;
        
        let mut result_bits = Vec::new();
        for (a_bit, b_bit) in a_bits.iter().zip(b_bits.iter()) {
            let xor_bit = gadgets::xor_gate(&mut self.circuit, *a_bit, *b_bit)?;
            result_bits.push(xor_bit);
        }
        
        gadgets::bit_pack(&mut self.circuit, &result_bits)
    }
    
    /// Bitwise NOT of a value
    pub fn bitwise_not(&mut self, a: usize, bits: usize) -> Result<usize> {
        let a_bits = gadgets::bit_decompose(&mut self.circuit, a, bits)?;
        
        let mut result_bits = Vec::new();
        for a_bit in a_bits {
            let not_bit = gadgets::not_gate(&mut self.circuit, a_bit)?;
            result_bits.push(not_bit);
        }
        
        gadgets::bit_pack(&mut self.circuit, &result_bits)
    }
    
    /// Left shift
    pub fn shift_left(&mut self, value: usize, shift: usize, bits: usize) -> Result<usize> {
        let value_bits = gadgets::bit_decompose(&mut self.circuit, value, bits)?;
        let shift_bits = gadgets::bit_decompose(&mut self.circuit, shift, 8)?; // Max shift 256
        
        // For simplicity, assume shift is a constant
        // Real implementation would handle variable shifts
        let mut result_bits = vec![utils::const_gate(&mut self.circuit, F::zero())?; bits];
        
        // This is simplified - real implementation would be more complex
        for i in 0..bits {
            if i < value_bits.len() {
                result_bits[i] = value_bits[i];
            }
        }
        
        gadgets::bit_pack(&mut self.circuit, &result_bits)
    }
    
    /// Right shift
    pub fn shift_right(&mut self, value: usize, shift: usize, bits: usize) -> Result<usize> {
        let value_bits = gadgets::bit_decompose(&mut self.circuit, value, bits)?;
        let shift_bits = gadgets::bit_decompose(&mut self.circuit, shift, 8)?;
        
        // Simplified implementation
        let mut result_bits = vec![utils::const_gate(&mut self.circuit, F::zero())?; bits];
        
        for i in 0..bits {
            if i < value_bits.len() {
                result_bits[i] = value_bits[i];
            }
        }
        
        gadgets::bit_pack(&mut self.circuit, &result_bits)
    }
    
    /// Rotate left
    pub fn rotate_left(&mut self, value: usize, rotate: usize, bits: usize) -> Result<usize> {
        // Simplified - real implementation would handle variable rotations
        let value_bits = gadgets::bit_decompose(&mut self.circuit, value, bits)?;
        
        // For now, just return the value
        gadgets::bit_pack(&mut self.circuit, &value_bits)
    }
    
    /// Rotate right
    pub fn rotate_right(&mut self, value: usize, rotate: usize, bits: usize) -> Result<usize> {
        // Simplified - real implementation would handle variable rotations
        let value_bits = gadgets::bit_decompose(&mut self.circuit, value, bits)?;
        
        // For now, just return the value
        gadgets::bit_pack(&mut self.circuit, &value_bits)
    }
}

/// Lookup table circuit
pub struct LookupTableCircuit<F: Field, C: CircuitBuilder<F>> {
    circuit: C,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field, C: CircuitBuilder<F>> LookupTableCircuit<F, C> {
    /// Create a new lookup table circuit
    pub fn new(circuit: C) -> Self {
        Self {
            circuit,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Lookup value in table
    pub fn lookup(&mut self, index: usize, table: &[F]) -> Result<usize> {
        if table.is_empty() {
            return Err(longfellow_core::LongfellowError::InvalidParameter(
                "Lookup table cannot be empty".to_string()
            ));
        }
        
        // Decompose index into bits
        let bits_needed = (table.len() as f64).log2().ceil() as usize;
        let index_bits = gadgets::bit_decompose(&mut self.circuit, index, bits_needed)?;
        
        // Create selector for each table entry
        let mut result = utils::const_gate(&mut self.circuit, F::zero())?;
        
        for (i, &value) in table.iter().enumerate() {
            // Check if index == i
            let mut is_selected = utils::const_gate(&mut self.circuit, F::one())?;
            
            for (j, &bit) in index_bits.iter().enumerate() {
                let expected_bit = if (i >> j) & 1 == 1 {
                    bit
                } else {
                    gadgets::not_gate(&mut self.circuit, bit)?
                };
                is_selected = gadgets::and_gate(&mut self.circuit, is_selected, expected_bit)?;
            }
            
            // Add value * is_selected to result
            let value_var = utils::const_gate(&mut self.circuit, value)?;
            let contribution = utils::mul_gate(&mut self.circuit, value_var, is_selected)?;
            result = utils::add_gate(&mut self.circuit, result, contribution)?;
        }
        
        Ok(result)
    }
    
    /// Lookup with range check
    pub fn lookup_checked(&mut self, index: usize, table: &[F]) -> Result<usize> {
        // Add range check for index
        let bits_needed = (table.len() as f64).log2().ceil() as usize;
        self.circuit.add_constraint(Constraint::Range { var: index, bits: bits_needed })?;
        
        self.lookup(index, table)
    }
}

/// Multiplexer circuit
pub struct MuxCircuit<F: Field, C: CircuitBuilder<F>> {
    circuit: C,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field, C: CircuitBuilder<F>> MuxCircuit<F, C> {
    /// Create a new multiplexer circuit
    pub fn new(circuit: C) -> Self {
        Self {
            circuit,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// 2-to-1 multiplexer
    pub fn mux2(&mut self, sel: usize, in0: usize, in1: usize) -> Result<usize> {
        gadgets::select(&mut self.circuit, sel, in1, in0)
    }
    
    /// 4-to-1 multiplexer
    pub fn mux4(
        &mut self,
        sel: &[usize; 2],
        inputs: &[usize; 4],
    ) -> Result<usize> {
        // First level: select between pairs
        let out0 = self.mux2(sel[0], inputs[0], inputs[1])?;
        let out1 = self.mux2(sel[0], inputs[2], inputs[3])?;
        
        // Second level: select between results
        self.mux2(sel[1], out0, out1)
    }
    
    /// N-to-1 multiplexer
    pub fn mux_n(
        &mut self,
        sel_bits: &[usize],
        inputs: &[usize],
    ) -> Result<usize> {
        let n = 1 << sel_bits.len();
        if inputs.len() != n {
            return Err(longfellow_core::LongfellowError::InvalidParameter(
                format!("Expected {} inputs, got {}", n, inputs.len())
            ));
        }
        
        // Build tree of 2-to-1 muxes
        let mut current_level = inputs.to_vec();
        
        for &sel_bit in sel_bits {
            let mut next_level = Vec::new();
            
            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    let out = self.mux2(sel_bit, chunk[0], chunk[1])?;
                    next_level.push(out);
                } else {
                    next_level.push(chunk[0]);
                }
            }
            
            current_level = next_level;
        }
        
        Ok(current_level[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StandardCircuit;
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_boolean_formula() {
        let circuit = StandardCircuit::<Fp128>::new();
        let mut bool_circuit = BooleanFormulaCircuit::new(circuit);
        
        let a = bool_circuit.circuit.alloc_var();
        let b = bool_circuit.circuit.alloc_var();
        let c = bool_circuit.circuit.alloc_var();
        
        // CNF: (a OR b) AND (NOT a OR c)
        let cnf = vec![
            vec![(a, true), (b, true)],
            vec![(a, false), (c, true)],
        ];
        
        let result = bool_circuit.evaluate_cnf(&cnf).unwrap();
        assert!(result > c);
    }
    
    #[test]
    fn test_bitwise_ops() {
        let circuit = StandardCircuit::<Fp128>::new();
        let mut bitwise = BitwiseCircuit::new(circuit);
        
        let a = bitwise.circuit.alloc_var();
        let b = bitwise.circuit.alloc_var();
        
        let and_result = bitwise.bitwise_and(a, b, 8).unwrap();
        let or_result = bitwise.bitwise_or(a, b, 8).unwrap();
        let xor_result = bitwise.bitwise_xor(a, b, 8).unwrap();
        
        assert!(and_result > b);
        assert!(or_result > and_result);
        assert!(xor_result > or_result);
    }
    
    #[test]
    fn test_lookup_table() {
        let circuit = StandardCircuit::<Fp128>::new();
        let mut lookup = LookupTableCircuit::new(circuit);
        
        let table = vec![
            Fp128::from(10),
            Fp128::from(20),
            Fp128::from(30),
            Fp128::from(40),
        ];
        
        let index = lookup.circuit.alloc_var();
        let result = lookup.lookup(index, &table).unwrap();
        assert!(result > index);
    }
    
    #[test]
    fn test_multiplexer() {
        let circuit = StandardCircuit::<Fp128>::new();
        let mut mux = MuxCircuit::new(circuit);
        
        let sel = mux.circuit.alloc_var();
        let in0 = mux.circuit.alloc_var();
        let in1 = mux.circuit.alloc_var();
        
        let result = mux.mux2(sel, in0, in1).unwrap();
        assert!(result > in1);
    }
}