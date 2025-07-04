/// Sparse quadratic form representation for gate constraints

use longfellow_algebra::traits::Field;
use longfellow_core::{LongfellowError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A corner in the sparse quadratic form
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuadCorner {
    /// Gate index
    pub g: u32,
    /// Left hand/wire index (0 means constant 1)
    pub h0: u32,
    /// Right hand/wire index (0 means constant 1)
    pub h1: u32,
}

impl QuadCorner {
    /// Create a new corner
    pub fn new(g: usize, h0: usize, h1: usize) -> Self {
        Self {
            g: g as u32,
            h0: h0 as u32,
            h1: h1 as u32,
        }
    }
    
    /// Convert to canonical form (h0 <= h1)
    pub fn canonicalize(self) -> Self {
        if self.h0 > self.h1 {
            Self {
                g: self.g,
                h0: self.h1,
                h1: self.h0,
            }
        } else {
            self
        }
    }
    
    /// Morton order for efficient storage and access
    pub fn morton_order(&self) -> u64 {
        // Interleave bits of g, h0, h1 for spatial locality
        let mut result = 0u64;
        for i in 0..21 {
            if i < 11 {
                result |= ((self.g >> i) & 1) as u64 * (1u64 << (3 * i));
            }
            if i < 11 {
                result |= ((self.h0 >> i) & 1) as u64 * (1u64 << (3 * i + 1));
            }
            if i < 11 {
                result |= ((self.h1 >> i) & 1) as u64 * (1u64 << (3 * i + 2));
            }
        }
        result
    }
}

impl Ord for QuadCorner {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.morton_order().cmp(&other.morton_order())
    }
}

impl PartialOrd for QuadCorner {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Sparse quadratic form Q(g,h0,h1) = sum of coefficients * gate * hand0 * hand1
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quad<F: Field> {
    /// Sparse corners with their coefficients
    corners: Vec<(QuadCorner, F)>,
    /// Whether corners are sorted
    sorted: bool,
}

impl<F: Field> Quad<F> {
    /// Create an empty quadratic form
    pub fn new() -> Self {
        Self {
            corners: Vec::new(),
            sorted: true,
        }
    }
    
    /// Add a corner with coefficient
    pub fn add_corner(&mut self, g: usize, h0: usize, h1: usize, coeff: F) -> Result<()> {
        if coeff == F::zero() {
            return Ok(()); // Skip zero coefficients
        }
        
        let corner = QuadCorner::new(g, h0, h1).canonicalize();
        self.corners.push((corner, coeff));
        self.sorted = false;
        
        Ok(())
    }
    
    /// Sort and coalesce duplicate corners
    pub fn coalesce(&mut self) {
        if self.sorted && !self.has_duplicates() {
            return;
        }
        
        // Sort by Morton order
        self.corners.sort_by_key(|(c, _)| *c);
        
        // Coalesce duplicates
        let mut result = Vec::new();
        let mut current: Option<(QuadCorner, F)> = None;
        
        for (corner, coeff) in self.corners.drain(..) {
            match current {
                None => current = Some((corner, coeff)),
                Some((c, v)) if c == corner => {
                    current = Some((c, v + coeff));
                }
                Some((c, v)) => {
                    if v != F::zero() {
                        result.push((c, v));
                    }
                    current = Some((corner, coeff));
                }
            }
        }
        
        if let Some((c, v)) = current {
            if v != F::zero() {
                result.push((c, v));
            }
        }
        
        self.corners = result;
        self.sorted = true;
    }
    
    /// Check if there might be duplicates
    fn has_duplicates(&self) -> bool {
        if self.corners.len() < 2 {
            return false;
        }
        
        for i in 1..self.corners.len() {
            if self.corners[i].0 == self.corners[i-1].0 {
                return true;
            }
        }
        false
    }
    
    /// Bind a gate variable to a value
    pub fn bind_gate(&mut self, var: usize, value: F) -> Result<Self> {
        self.coalesce();
        
        let mut result = Quad::new();
        let mask = 1u32 << var;
        
        for &(corner, coeff) in &self.corners {
            let bit = (corner.g & mask) != 0;
            
            // Keep corners where the bit matches the binding
            if bit == (value == F::one()) {
                let new_corner = QuadCorner {
                    g: corner.g & !mask, // Clear the bound bit
                    h0: corner.h0,
                    h1: corner.h1,
                };
                result.corners.push((new_corner, coeff));
            } else if value != F::zero() && value != F::one() {
                // Interpolate for non-boolean values
                let new_corner = QuadCorner {
                    g: corner.g & !mask,
                    h0: corner.h0,
                    h1: corner.h1,
                };
                
                let weight = if bit { value } else { F::one() - value };
                result.corners.push((new_corner, coeff * weight));
            }
        }
        
        result.sorted = false;
        result.coalesce();
        Ok(result)
    }
    
    /// Bind a hand/wire variable to a value
    pub fn bind_hand(&mut self, var: usize, value: F, is_left: bool) -> Result<Self> {
        self.coalesce();
        
        let mut result = Quad::new();
        let mask = 1u32 << var;
        
        for &(corner, coeff) in &self.corners {
            let (h_check, h_other) = if is_left {
                (corner.h0, corner.h1)
            } else {
                (corner.h1, corner.h0)
            };
            
            if h_check == 0 {
                // Constant 1, no binding needed
                result.corners.push((corner, coeff));
                continue;
            }
            
            let bit = (h_check & mask) != 0;
            
            if bit == (value == F::one()) {
                let new_h = h_check & !mask;
                let new_corner = if is_left {
                    QuadCorner::new(corner.g as usize, new_h as usize, h_other as usize)
                } else {
                    QuadCorner::new(corner.g as usize, h_other as usize, new_h as usize)
                };
                result.corners.push((new_corner.canonicalize(), coeff));
            } else if value != F::zero() && value != F::one() {
                let new_h = h_check & !mask;
                let weight = if bit { value } else { F::one() - value };
                
                let new_corner = if is_left {
                    QuadCorner::new(corner.g as usize, new_h as usize, h_other as usize)
                } else {
                    QuadCorner::new(corner.g as usize, h_other as usize, new_h as usize)
                };
                result.corners.push((new_corner.canonicalize(), coeff * weight));
            }
        }
        
        result.sorted = false;
        result.coalesce();
        Ok(result)
    }
    
    /// Evaluate the quadratic form at given points
    pub fn evaluate(&self, gates: &[F], left_hands: &[F], right_hands: &[F]) -> Result<F> {
        let mut sum = F::zero();
        
        for &(corner, coeff) in &self.corners {
            let g_val = if corner.g < gates.len() as u32 {
                gates[corner.g as usize]
            } else {
                return Err(LongfellowError::InvalidParameter(
                    format!("Gate index {} out of range", corner.g)
                ));
            };
            
            let h0_val = if corner.h0 == 0 {
                F::one()
            } else if (corner.h0 as usize - 1) < left_hands.len() {
                left_hands[corner.h0 as usize - 1]
            } else {
                return Err(LongfellowError::InvalidParameter(
                    format!("Left hand index {} out of range", corner.h0)
                ));
            };
            
            let h1_val = if corner.h1 == 0 {
                F::one()
            } else if (corner.h1 as usize - 1) < right_hands.len() {
                right_hands[corner.h1 as usize - 1]
            } else {
                return Err(LongfellowError::InvalidParameter(
                    format!("Right hand index {} out of range", corner.h1)
                ));
            };
            
            sum += coeff * g_val * h0_val * h1_val;
        }
        
        Ok(sum)
    }
    
    /// Get number of corners
    pub fn num_corners(&self) -> usize {
        self.corners.len()
    }
    
    /// Iterate over corners
    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, usize, F)> + '_ {
        self.corners.iter().map(|(c, v)| {
            (c.g as usize, c.h0 as usize, c.h1 as usize, *v)
        })
    }
    
    /// Validate the quadratic form
    pub fn validate(&self, max_gates: usize, max_hands: usize) -> Result<()> {
        let max_g = (1u32 << max_gates) - 1;
        let max_h = (1u32 << max_hands) - 1;
        
        for &(corner, _) in &self.corners {
            if corner.g > max_g {
                return Err(LongfellowError::InvalidParameter(
                    format!("Gate index {} exceeds maximum {}", corner.g, max_g)
                ));
            }
            
            if corner.h0 > max_h + 1 || corner.h1 > max_h + 1 {
                return Err(LongfellowError::InvalidParameter(
                    format!("Hand indices {},{} exceed maximum {}", 
                        corner.h0, corner.h1, max_h)
                ));
            }
        }
        
        Ok(())
    }
}

impl<F: Field> Default for Quad<F> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_quad_corner_ordering() {
        let c1 = QuadCorner::new(1, 2, 3);
        let c2 = QuadCorner::new(1, 3, 2);
        
        // Should canonicalize to same form
        assert_eq!(c1.canonicalize(), c2.canonicalize());
        
        // Test Morton ordering
        let c3 = QuadCorner::new(0, 0, 0);
        let c4 = QuadCorner::new(1, 0, 0);
        assert!(c3 < c4);
    }
    
    #[test]
    fn test_quad_coalesce() {
        let mut quad = Quad::<Fp128>::new();
        
        // Add duplicate corners
        quad.add_corner(1, 2, 3, Fp128::from(5)).unwrap();
        quad.add_corner(1, 3, 2, Fp128::from(3)).unwrap(); // Same corner, different order
        quad.add_corner(2, 1, 1, Fp128::from(7)).unwrap();
        
        quad.coalesce();
        
        assert_eq!(quad.num_corners(), 2);
        
        // Check coalesced values
        let corners: Vec<_> = quad.corners.clone();
        assert_eq!(corners[0].1, Fp128::from(8)); // 5 + 3
        assert_eq!(corners[1].1, Fp128::from(7));
    }
    
    #[test]
    fn test_quad_binding() {
        let mut quad = Quad::<Fp128>::new();
        
        // Add some corners: g_0 * h_0 * h_1 + g_1 * h_0
        quad.add_corner(0, 1, 2, Fp128::one()).unwrap(); // g_0 * h_0 * h_1
        quad.add_corner(1, 1, 0, Fp128::one()).unwrap(); // g_1 * h_0
        
        // Bind g_0 = 1 (should keep first corner, remove second)
        let bound = quad.bind_gate(0, Fp128::one()).unwrap();
        assert_eq!(bound.num_corners(), 1);
        
        // Bind h_0 = 1 (both corners should remain)
        let bound2 = quad.bind_hand(0, Fp128::one(), true).unwrap();
        assert_eq!(bound2.num_corners(), 2);
    }
    
    #[test]
    fn test_quad_evaluate() {
        let mut quad = Quad::<Fp128>::new();
        
        // Q(g,h) = g_0 * h_0 * h_1 + 2 * g_1 * h_0
        quad.add_corner(0, 1, 2, Fp128::one()).unwrap();
        quad.add_corner(1, 1, 0, Fp128::from(2)).unwrap();
        
        let gates = vec![Fp128::from(3), Fp128::from(4)];
        let left = vec![Fp128::from(5), Fp128::from(6)];
        let right = vec![Fp128::from(7), Fp128::from(8)];
        
        let result = quad.evaluate(&gates, &left, &right).unwrap();
        
        // 3 * 5 * 7 + 2 * 4 * 5 = 105 + 40 = 145
        assert_eq!(result, Fp128::from(145));
    }
}