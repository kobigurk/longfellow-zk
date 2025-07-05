/// Circuit implementations for cryptographic operations
/// 
/// This module provides circuit representations of common cryptographic
/// operations like hashing, signatures, and comparisons.

pub mod gadgets;
pub mod hash;
pub mod comparison;
pub mod arithmetic;
pub mod boolean;

use longfellow_algebra::traits::Field;
use longfellow_core::{LongfellowError, Result};
use longfellow_sumcheck::circuit::{Circuit as SumcheckCircuit, Layer};
use longfellow_ligero::ConstraintSystem;

/// Circuit builder trait
pub trait CircuitBuilder<F: Field> {
    /// Add a constraint
    fn add_constraint(&mut self, constraint: Constraint<F>) -> Result<()>;
    
    /// Get number of variables
    fn num_vars(&self) -> usize;
    
    /// Allocate a new variable
    fn alloc_var(&mut self) -> usize;
    
    /// Allocate multiple variables
    fn alloc_vars(&mut self, count: usize) -> Vec<usize>;
}

/// Constraint types
#[derive(Clone, Debug)]
pub enum Constraint<F: Field> {
    /// Linear constraint: sum of coeffs[i] * vars[i] = constant
    Linear {
        coeffs: Vec<(usize, F)>,
        constant: F,
    },
    
    /// Quadratic constraint: var[x] * var[y] = var[z]
    Quadratic {
        x: usize,
        y: usize,
        z: usize,
    },
    
    /// Boolean constraint: var is 0 or 1
    Boolean {
        var: usize,
    },
    
    /// Range constraint: var is in [0, 2^bits)
    Range {
        var: usize,
        bits: usize,
    },
}

/// Standard circuit implementation
pub struct StandardCircuit<F: Field> {
    /// Constraint system for Ligero
    pub constraints: ConstraintSystem<F>,
    /// Current number of variables
    num_vars: usize,
}

impl<F: Field> StandardCircuit<F> {
    /// Create a new circuit
    pub fn new() -> Self {
        Self {
            constraints: ConstraintSystem::new(F::zero()),
            num_vars: 0,
        }
    }
    
    /// Set the witness values
    pub fn set_witness(&mut self, witness: Vec<F>) -> Result<()> {
        if witness.len() != self.num_vars {
            return Err(LongfellowError::InvalidParameter(
                format!("Witness size mismatch: {} vs {}", witness.len(), self.num_vars)
            ));
        }
        
        // Update constraint system
        self.constraints = ConstraintSystem::new(witness.len());
        
        Ok(())
    }
}

impl<F: Field> CircuitBuilder<F> for StandardCircuit<F> {
    fn add_constraint(&mut self, constraint: Constraint<F>) -> Result<()> {
        match constraint {
            Constraint::Linear { coeffs, constant } => {
                self.constraints.add_linear_constraint(coeffs, constant);
            }
            Constraint::Quadratic { x, y, z } => {
                self.constraints.add_quadratic_constraint(x, y, z);
            }
            Constraint::Boolean { var } => {
                // var * (var - 1) = 0
                let temp = self.alloc_var();
                self.constraints.add_linear_constraint(
                    vec![(var, F::one()), (temp, -F::one())],
                    F::one(),
                );
                let result_var = self.alloc_var();
                self.constraints.add_quadratic_constraint(var, temp, result_var);
            }
            Constraint::Range { var, bits } => {
                // Decompose into bits
                let bit_vars = self.alloc_vars(bits);
                
                // Each bit is boolean
                for &bit in &bit_vars {
                    self.add_constraint(Constraint::Boolean { var: bit })?;
                }
                
                // Sum of bits * 2^i = var
                let mut coeffs = vec![];
                for (i, &bit) in bit_vars.iter().enumerate() {
                    coeffs.push((bit, F::from_u64(1u64 << i)));
                }
                coeffs.push((var, -F::one()));
                
                self.constraints.add_linear_constraint(coeffs, F::zero());
            }
        }
        Ok(())
    }
    
    fn num_vars(&self) -> usize {
        self.num_vars
    }
    
    fn alloc_var(&mut self) -> usize {
        let var = self.num_vars;
        self.num_vars += 1;
        var
    }
    
    fn alloc_vars(&mut self, count: usize) -> Vec<usize> {
        let start = self.num_vars;
        self.num_vars += count;
        (start..self.num_vars).collect()
    }
}

/// Layered circuit for Sumcheck
pub struct LayeredCircuit<F: Field> {
    /// Sumcheck circuit
    pub circuit: SumcheckCircuit<F>,
    /// Variable allocation counter
    _next_var: usize,
}

impl<F: Field> LayeredCircuit<F> {
    /// Create a new layered circuit
    pub fn new() -> Self {
        Self {
            circuit: SumcheckCircuit::new(),
            _next_var: 0,
        }
    }
    
    /// Add a layer
    pub fn add_layer(&mut self, layer: Layer<F>) -> Result<()> {
        self.circuit.add_layer(layer)
    }
}

/// Circuit compiler trait
pub trait CircuitCompiler<F: Field> {
    /// Compile to constraint system
    fn compile_to_constraints(&self) -> Result<ConstraintSystem<F>>;
    
    /// Compile to layered circuit
    fn compile_to_layers(&self) -> Result<SumcheckCircuit<F>>;
    
    /// Get public inputs
    fn public_inputs(&self) -> Vec<F>;
    
    /// Get private inputs
    fn private_inputs(&self) -> Vec<F>;
}

/// Utilities for circuit construction
pub mod utils {
    use super::*;
    
    /// Create an addition gate
    pub fn add_gate<F: Field, C: CircuitBuilder<F>>(
        circuit: &mut C,
        a: usize,
        b: usize,
    ) -> Result<usize> {
        let c = circuit.alloc_var();
        circuit.add_constraint(Constraint::Linear {
            coeffs: vec![(a, F::one()), (b, F::one()), (c, -F::one())],
            constant: F::zero(),
        })?;
        Ok(c)
    }
    
    /// Create a multiplication gate
    pub fn mul_gate<F: Field, C: CircuitBuilder<F>>(
        circuit: &mut C,
        a: usize,
        b: usize,
    ) -> Result<usize> {
        let c = circuit.alloc_var();
        circuit.add_constraint(Constraint::Quadratic { x: a, y: b, z: c })?;
        Ok(c)
    }
    
    /// Create a constant gate
    pub fn const_gate<F: Field, C: CircuitBuilder<F>>(
        circuit: &mut C,
        value: F,
    ) -> Result<usize> {
        let c = circuit.alloc_var();
        circuit.add_constraint(Constraint::Linear {
            coeffs: vec![(c, F::one())],
            constant: value,
        })?;
        Ok(c)
    }
    
    /// Assert equality
    pub fn assert_equal<F: Field, C: CircuitBuilder<F>>(
        circuit: &mut C,
        a: usize,
        b: usize,
    ) -> Result<()> {
        circuit.add_constraint(Constraint::Linear {
            coeffs: vec![(a, F::one()), (b, -F::one())],
            constant: F::zero(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_standard_circuit() {
        let mut circuit = StandardCircuit::<Fp128>::new();
        
        // Allocate variables
        let a = circuit.alloc_var();
        let b = circuit.alloc_var();
        let c = circuit.alloc_var();
        
        // Add constraint: a + b = c
        circuit.add_constraint(Constraint::Linear {
            coeffs: vec![(a, Fp128::one()), (b, Fp128::one()), (c, -Fp128::one())],
            constant: Fp128::zero(),
        }).unwrap();
        
        // Add constraint: a * b = c
        let d = circuit.alloc_var();
        circuit.add_constraint(Constraint::Quadratic { x: a, y: b, z: d }).unwrap();
        
        assert_eq!(circuit.num_vars(), 4);
    }
    
    #[test]
    fn test_circuit_utils() {
        let mut circuit = StandardCircuit::<Fp128>::new();
        
        let a = circuit.alloc_var();
        let b = circuit.alloc_var();
        
        // Test addition
        let c = utils::add_gate(&mut circuit, a, b).unwrap();
        assert_eq!(c, 2);
        
        // Test multiplication
        let d = utils::mul_gate(&mut circuit, a, b).unwrap();
        assert_eq!(d, 3);
        
        // Test constant
        let e = utils::const_gate(&mut circuit, Fp128::from(42)).unwrap();
        assert_eq!(e, 4);
        
        // Test equality assertion
        utils::assert_equal(&mut circuit, c, d).unwrap();
    }
}