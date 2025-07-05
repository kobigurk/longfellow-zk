/// Layered arithmetic circuit representation for sumcheck

use longfellow_algebra::traits::Field;
use longfellow_core::{LongfellowError, Result};
use serde::{Deserialize, Serialize};
use crate::quad::Quad;

/// A layer in an arithmetic circuit
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer<F: Field> {
    /// Number of output wires (log2)
    pub nout: usize,
    
    /// Number of input wires (log2)
    pub nin: usize,
    
    /// Number of gates (log2)
    pub ngate: usize,
    
    /// Quadratic form representing gate constraints
    pub quad: Quad<F>,
    
    /// Number of public inputs at this layer
    pub npub_in: Option<usize>,
}

impl<F: Field> Layer<F> {
    /// Create a new layer
    pub fn new(nout: usize, nin: usize, ngate: usize) -> Self {
        Self {
            nout,
            nin,
            ngate,
            quad: Quad::new(),
            npub_in: None,
        }
    }
    
    /// Get total number of output wires
    pub fn num_outputs(&self) -> usize {
        1 << self.nout
    }
    
    /// Get total number of input wires
    pub fn num_inputs(&self) -> usize {
        1 << self.nin
    }
    
    /// Get total number of gates
    pub fn num_gates(&self) -> usize {
        1 << self.ngate
    }
    
    /// Add a gate constraint
    pub fn add_gate(
        &mut self,
        output: usize,
        left: usize,
        right: usize,
        gate_type: GateType<F>,
    ) -> Result<()> {
        if output >= self.num_outputs() {
            return Err(LongfellowError::InvalidParameter(
                format!("Output index {} out of range", output)
            ));
        }
        
        if left >= self.num_inputs() || right >= self.num_inputs() {
            return Err(LongfellowError::InvalidParameter(
                "Input indices out of range".to_string()
            ));
        }
        
        match gate_type {
            GateType::Add(coeff) => {
                // For add gate: output = coeff * (left + right)
                // In quad form: coeff * gate[output] * (hand[left] + hand[right])
                // Since hand indices are 1-based (0 means constant 1), we add 1
                self.quad.add_corner(output, left + 1, 0, coeff)?;
                self.quad.add_corner(output, 0, right + 1, coeff)?;
            }
            GateType::Mul(coeff) => {
                // For mul gate: output = coeff * left * right
                self.quad.add_corner(output, left + 1, right + 1, coeff)?;
            }
            GateType::Const(value) => {
                // For const gate: output = value
                self.quad.add_corner(output, 0, 0, value)?;
            }
        }
        
        Ok(())
    }
    
    /// Validate the layer
    pub fn validate(&self) -> Result<()> {
        if self.nout > crate::MAX_BINDINGS || 
           self.nin > crate::MAX_BINDINGS || 
           self.ngate > crate::MAX_BINDINGS {
            return Err(LongfellowError::InvalidParameter(
                format!("Layer dimensions exceed maximum: {} > {}", 
                    self.nout.max(self.nin).max(self.ngate), 
                    crate::MAX_BINDINGS)
            ));
        }
        
        self.quad.validate(self.ngate, self.nin)?;
        
        Ok(())
    }
}

/// Type of arithmetic gate
#[derive(Clone, Debug)]
pub enum GateType<F: Field> {
    /// Addition gate with coefficient
    Add(F),
    /// Multiplication gate with coefficient
    Mul(F),
    /// Constant gate
    Const(F),
}

/// A layered arithmetic circuit
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Circuit<F: Field> {
    /// Layers from output (index 0) to input
    pub layers: Vec<Layer<F>>,
    
    /// Number of public inputs
    pub num_public_inputs: usize,
}

impl<F: Field> Circuit<F> {
    /// Create a new empty circuit
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            num_public_inputs: 0,
        }
    }
    
    /// Add a layer to the circuit
    pub fn add_layer(&mut self, layer: Layer<F>) -> Result<()> {
        // Validate layer dimensions match
        if !self.layers.is_empty() {
            let prev_layer = &self.layers[self.layers.len() - 1];
            if layer.nout != prev_layer.nin {
                return Err(LongfellowError::InvalidParameter(
                    format!("Layer dimension mismatch: {} != {}", 
                        layer.nout, prev_layer.nin)
                ));
            }
        }
        
        layer.validate()?;
        self.layers.push(layer);
        Ok(())
    }
    
    /// Get the number of output variables
    pub fn num_output_vars(&self) -> usize {
        self.layers.first().map(|l| l.nout).unwrap_or(0)
    }
    
    /// Get the number of input variables
    pub fn num_input_vars(&self) -> usize {
        self.layers.last().map(|l| l.nin).unwrap_or(0)
    }
    
    /// Get total number of variables across all layers
    pub fn num_vars(&self) -> usize {
        self.layers.iter().map(|l| l.nout + l.nin + l.ngate).sum()
    }
    
    /// Evaluate the circuit on given inputs
    pub fn evaluate(&self, inputs: &[F], num_copies: usize) -> Result<Vec<F>> {
        if inputs.len() != self.num_inputs() * num_copies {
            return Err(LongfellowError::InvalidParameter(
                format!("Input size mismatch: {} != {}", 
                    inputs.len(), self.num_inputs() * num_copies)
            ));
        }
        
        let mut current = inputs.to_vec();
        
        // Process layers from input to output (reverse order)
        for layer in self.layers.iter().rev() {
            let mut next = vec![F::zero(); layer.num_outputs() * num_copies];
            
            // Evaluate each copy
            for copy in 0..num_copies {
                let input_offset = copy * layer.num_inputs();
                let output_offset = copy * layer.num_outputs();
                
                // Evaluate layer
                for (g, left, right, coeff) in layer.quad.iter() {
                    let out_idx = output_offset + g;
                    let left_val = if left > 0 {
                        current[input_offset + left - 1]
                    } else {
                        F::one()
                    };
                    let right_val = if right > 0 {
                        current[input_offset + right - 1]
                    } else {
                        F::one()
                    };
                    
                    next[out_idx] += coeff * left_val * right_val;
                }
            }
            
            current = next;
        }
        
        Ok(current)
    }
    
    /// Get the total number of inputs
    pub fn num_inputs(&self) -> usize {
        1 << self.num_input_vars()
    }
    
    /// Get the total number of outputs
    pub fn num_outputs(&self) -> usize {
        1 << self.num_output_vars()
    }
    
    /// Validate the circuit
    pub fn validate(&self) -> Result<()> {
        if self.layers.is_empty() {
            return Err(LongfellowError::InvalidParameter(
                "Circuit has no layers".to_string()
            ));
        }
        
        // Check layer compatibility
        for i in 1..self.layers.len() {
            if self.layers[i].nout != self.layers[i-1].nin {
                return Err(LongfellowError::InvalidParameter(
                    format!("Layer {} output doesn't match layer {} input", i, i-1)
                ));
            }
        }
        
        // Validate each layer
        for (i, layer) in self.layers.iter().enumerate() {
            layer.validate().map_err(|e| 
                LongfellowError::InvalidParameter(
                    format!("Layer {} validation failed: {}", i, e)
                )
            )?;
        }
        
        Ok(())
    }
}

/// Builder for constructing circuits
pub struct CircuitBuilder<F: Field> {
    circuit: Circuit<F>,
    current_layer: Option<Layer<F>>,
}

impl<F: Field> CircuitBuilder<F> {
    /// Create a new circuit builder
    pub fn new() -> Self {
        Self {
            circuit: Circuit::new(),
            current_layer: None,
        }
    }
    
    /// Start a new layer
    pub fn begin_layer(&mut self, nout: usize, nin: usize, ngate: usize) -> Result<()> {
        if self.current_layer.is_some() {
            return Err(LongfellowError::InvalidParameter(
                "Previous layer not finalized".to_string()
            ));
        }
        
        self.current_layer = Some(Layer::new(nout, nin, ngate));
        Ok(())
    }
    
    /// Add a gate to the current layer
    pub fn add_gate(
        &mut self,
        output: usize,
        left: usize,
        right: usize,
        gate_type: GateType<F>,
    ) -> Result<()> {
        let layer = self.current_layer.as_mut()
            .ok_or_else(|| LongfellowError::InvalidParameter(
                "No layer started".to_string()
            ))?;
        
        layer.add_gate(output, left, right, gate_type)
    }
    
    /// Finalize the current layer
    pub fn finalize_layer(&mut self) -> Result<()> {
        let layer = self.current_layer.take()
            .ok_or_else(|| LongfellowError::InvalidParameter(
                "No layer to finalize".to_string()
            ))?;
        
        self.circuit.add_layer(layer)
    }
    
    /// Set the number of public inputs
    pub fn set_public_inputs(&mut self, num: usize) {
        self.circuit.num_public_inputs = num;
    }
    
    /// Build the final circuit
    pub fn build(self) -> Result<Circuit<F>> {
        if self.current_layer.is_some() {
            return Err(LongfellowError::InvalidParameter(
                "Unfinalized layer exists".to_string()
            ));
        }
        
        self.circuit.validate()?;
        Ok(self.circuit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_simple_circuit() {
        let mut builder = CircuitBuilder::<Fp128>::new();
        
        // Layer 0: 2 outputs, 4 inputs
        builder.begin_layer(1, 2, 1).unwrap(); // 2^1=2 outputs, 2^2=4 inputs
        
        // Output 0 = Input 0 + Input 1
        builder.add_gate(0, 0, 1, GateType::Add(Fp128::one())).unwrap();
        
        // Output 1 = Input 2 * Input 3
        builder.add_gate(1, 2, 3, GateType::Mul(Fp128::one())).unwrap();
        
        builder.finalize_layer().unwrap();
        
        let circuit = builder.build().unwrap();
        
        // Test evaluation
        let inputs = vec![
            Fp128::from(2),
            Fp128::from(3),
            Fp128::from(4),
            Fp128::from(5),
        ];
        
        let outputs = circuit.evaluate(&inputs, 1).unwrap();
        assert_eq!(outputs[0], Fp128::from(5)); // 2 + 3
        assert_eq!(outputs[1], Fp128::from(20)); // 4 * 5
    }
    
    #[test]
    fn test_multi_layer_circuit() {
        let mut builder = CircuitBuilder::<Fp128>::new();
        
        // Layer 0: 1 output, 2 inputs
        builder.begin_layer(0, 1, 0).unwrap();
        builder.add_gate(0, 0, 1, GateType::Mul(Fp128::one())).unwrap();
        builder.finalize_layer().unwrap();
        
        // Layer 1: 2 outputs, 4 inputs
        builder.begin_layer(1, 2, 1).unwrap();
        builder.add_gate(0, 0, 1, GateType::Add(Fp128::one())).unwrap();
        builder.add_gate(1, 2, 3, GateType::Add(Fp128::one())).unwrap();
        builder.finalize_layer().unwrap();
        
        let circuit = builder.build().unwrap();
        
        // Test: (a+b) * (c+d)
        let inputs = vec![
            Fp128::from(1),
            Fp128::from(2),
            Fp128::from(3),
            Fp128::from(4),
        ];
        
        let outputs = circuit.evaluate(&inputs, 1).unwrap();
        assert_eq!(outputs[0], Fp128::from(21)); // (1+2) * (3+4) = 3 * 7 = 21
    }
}