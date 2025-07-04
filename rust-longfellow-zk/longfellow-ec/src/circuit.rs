/// Circuit-friendly elliptic curve operations

use crate::{FieldElement, Point};
use longfellow_algebra::traits::Field;
use longfellow_core::Result;

/// Circuit representation of a field element
#[derive(Clone, Debug)]
pub struct CircuitFieldElement<F: Field> {
    /// Limbs representing the field element
    pub limbs: Vec<F>,
    /// Number of bits per limb
    pub limb_bits: usize,
}

impl<F: Field> CircuitFieldElement<F> {
    /// Create from native field element
    pub fn from_native(elem: &FieldElement, limb_bits: usize) -> Result<Self> {
        let bytes = elem.to_bytes();
        let num_limbs = (256 + limb_bits - 1) / limb_bits;
        let mut limbs = Vec::with_capacity(num_limbs);
        
        // Convert to limbs (little-endian)
        for i in 0..num_limbs {
            let mut limb_value = 0u64;
            for j in 0..limb_bits {
                let bit_idx = i * limb_bits + j;
                if bit_idx < 256 {
                    let byte_idx = bit_idx / 8;
                    let bit_in_byte = bit_idx % 8;
                    if (bytes[31 - byte_idx] >> bit_in_byte) & 1 == 1 {
                        limb_value |= 1u64 << j;
                    }
                }
            }
            limbs.push(F::from_u64(limb_value));
        }
        
        Ok(Self { limbs, limb_bits })
    }
    
    /// Convert back to native field element
    pub fn to_native(&self) -> Result<FieldElement> {
        let mut bytes = [0u8; 32];
        
        // Reconstruct bytes from limbs
        for (i, limb) in self.limbs.iter().enumerate() {
            let limb_bytes = limb.to_bytes_le();
            // Assume each limb is at most 8 bytes
            let limb_value = if limb_bytes.len() >= 8 {
                u64::from_le_bytes(limb_bytes[0..8].try_into().unwrap())
            } else {
                let mut buf = [0u8; 8];
                buf[..limb_bytes.len()].copy_from_slice(&limb_bytes);
                u64::from_le_bytes(buf)
            };
            
            for j in 0..self.limb_bits {
                let bit_idx = i * self.limb_bits + j;
                if bit_idx < 256 {
                    let byte_idx = bit_idx / 8;
                    let bit_in_byte = bit_idx % 8;
                    if (limb_value >> j) & 1 == 1 {
                        bytes[31 - byte_idx] |= 1u8 << bit_in_byte;
                    }
                }
            }
        }
        
        FieldElement::from_bytes(&bytes)
    }
}

/// Circuit representation of an elliptic curve point
#[derive(Clone, Debug)]
pub struct CircuitPoint<F: Field> {
    /// X coordinate
    pub x: CircuitFieldElement<F>,
    /// Y coordinate
    pub y: CircuitFieldElement<F>,
    /// Is infinity flag
    pub is_infinity: F,
}

impl<F: Field> CircuitPoint<F> {
    /// Create from native point
    pub fn from_native(point: &Point, limb_bits: usize) -> Result<Self> {
        Ok(Self {
            x: CircuitFieldElement::from_native(&point.x, limb_bits)?,
            y: CircuitFieldElement::from_native(&point.y, limb_bits)?,
            is_infinity: if point.is_infinity { F::one() } else { F::zero() },
        })
    }
    
    /// Convert back to native point
    pub fn to_native(&self) -> Result<Point> {
        if self.is_infinity == F::one() {
            Ok(Point::infinity())
        } else {
            let x = self.x.to_native()?;
            let y = self.y.to_native()?;
            Point::new(x, y)
        }
    }
}

/// Circuit operations for elliptic curves
pub struct CircuitEllipticCurve<F: Field> {
    /// Number of bits per limb
    limb_bits: usize,
    /// P-256 field modulus limbs
    p_limbs: Vec<F>,
    /// P-256 curve parameter a = -3
    a_limbs: Vec<F>,
    /// P-256 curve parameter b
    b_limbs: Vec<F>,
}

impl<F: Field> CircuitEllipticCurve<F> {
    /// Create a new circuit EC instance
    pub fn new(limb_bits: usize) -> Result<Self> {
        // P-256 prime: 2^256 - 2^224 + 2^192 + 2^96 - 1
        let p_bytes = [
            0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];
        let p = FieldElement::from_bytes(&p_bytes)?;
        let p_circuit = CircuitFieldElement::from_native(&p, limb_bits)?;
        
        // a = -3 mod p
        let a = FieldElement::from_bytes(&[
            0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfc,
        ])?;
        let a_circuit = CircuitFieldElement::from_native(&a, limb_bits)?;
        
        // b parameter
        let b_bytes = [
            0x5a, 0xc6, 0x35, 0xd8, 0xaa, 0x3a, 0x93, 0xe7,
            0xb3, 0xeb, 0xbd, 0x55, 0x76, 0x98, 0x86, 0xbc,
            0x65, 0x1d, 0x06, 0xb0, 0xcc, 0x53, 0xb0, 0xf6,
            0x3b, 0xce, 0x3c, 0x3e, 0x27, 0xd2, 0x60, 0x4b,
        ];
        let b = FieldElement::from_bytes(&b_bytes)?;
        let b_circuit = CircuitFieldElement::from_native(&b, limb_bits)?;
        
        Ok(Self {
            limb_bits,
            p_limbs: p_circuit.limbs,
            a_limbs: a_circuit.limbs,
            b_limbs: b_circuit.limbs,
        })
    }
    
    /// Check if a point is on the curve (circuit version)
    pub fn is_on_curve(&self, point: &CircuitPoint<F>) -> Result<F> {
        // Skip check if point is infinity
        if point.is_infinity == F::one() {
            return Ok(F::one());
        }
        
        // Compute y^2
        let y_squared = self.field_square(&point.y)?;
        
        // Compute x^3
        let x_squared = self.field_square(&point.x)?;
        let x_cubed = self.field_mul(&x_squared, &point.x)?;
        
        // Compute a*x
        let ax = self.field_mul(&self.a_circuit(), &point.x)?;
        
        // Compute x^3 + a*x + b
        let rhs = self.field_add(&x_cubed, &ax)?;
        let rhs = self.field_add(&rhs, &self.b_circuit())?;
        
        // Check y^2 == x^3 + a*x + b
        self.field_equal(&y_squared, &rhs)
    }
    
    /// Add two field elements in circuit
    fn field_add(&self, a: &CircuitFieldElement<F>, b: &CircuitFieldElement<F>) -> Result<CircuitFieldElement<F>> {
        let mut result_limbs = Vec::with_capacity(a.limbs.len());
        
        for i in 0..a.limbs.len() {
            result_limbs.push(a.limbs[i] + b.limbs[i]);
        }
        
        // TODO: Implement proper reduction
        Ok(CircuitFieldElement {
            limbs: result_limbs,
            limb_bits: self.limb_bits,
        })
    }
    
    /// Multiply two field elements in circuit
    fn field_mul(&self, a: &CircuitFieldElement<F>, b: &CircuitFieldElement<F>) -> Result<CircuitFieldElement<F>> {
        // Schoolbook multiplication followed by reduction
        let num_limbs = a.limbs.len();
        let mut product_limbs = vec![F::zero(); 2 * num_limbs];
        
        for i in 0..num_limbs {
            for j in 0..num_limbs {
                product_limbs[i + j] += a.limbs[i] * b.limbs[j];
            }
        }
        
        // TODO: Implement proper Barrett reduction
        Ok(CircuitFieldElement {
            limbs: product_limbs[..num_limbs].to_vec(),
            limb_bits: self.limb_bits,
        })
    }
    
    /// Square a field element in circuit
    fn field_square(&self, a: &CircuitFieldElement<F>) -> Result<CircuitFieldElement<F>> {
        self.field_mul(a, a)
    }
    
    /// Check field element equality
    fn field_equal(&self, a: &CircuitFieldElement<F>, b: &CircuitFieldElement<F>) -> Result<F> {
        let mut all_equal = F::one();
        
        for i in 0..a.limbs.len() {
            let diff = a.limbs[i] - b.limbs[i];
            // In a real circuit, we'd use an equality gadget
            if diff != F::zero() {
                all_equal = F::zero();
            }
        }
        
        Ok(all_equal)
    }
    
    /// Get circuit representation of curve parameter a
    fn a_circuit(&self) -> CircuitFieldElement<F> {
        CircuitFieldElement {
            limbs: self.a_limbs.clone(),
            limb_bits: self.limb_bits,
        }
    }
    
    /// Get circuit representation of curve parameter b
    fn b_circuit(&self) -> CircuitFieldElement<F> {
        CircuitFieldElement {
            limbs: self.b_limbs.clone(),
            limb_bits: self.limb_bits,
        }
    }
}

/// Circuit for scalar multiplication using double-and-add
pub struct ScalarMulCircuit<F: Field> {
    ec: CircuitEllipticCurve<F>,
}

impl<F: Field> ScalarMulCircuit<F> {
    /// Create a new scalar multiplication circuit
    pub fn new(limb_bits: usize) -> Result<Self> {
        Ok(Self {
            ec: CircuitEllipticCurve::new(limb_bits)?,
        })
    }
    
    /// Compute scalar multiplication k * P
    pub fn scalar_mul(
        &self,
        scalar_bits: &[F],
        point: &CircuitPoint<F>,
    ) -> Result<CircuitPoint<F>> {
        // Initialize result to infinity
        let mut result = self.point_infinity();
        let mut temp = point.clone();
        
        // Double-and-add algorithm
        for &bit in scalar_bits.iter() {
            // Conditionally add temp to result based on bit
            let new_result = self.conditional_add(&result, &temp, bit)?;
            result = new_result;
            
            // Double temp
            temp = self.point_double(&temp)?;
        }
        
        Ok(result)
    }
    
    /// Create point at infinity
    fn point_infinity(&self) -> CircuitPoint<F> {
        CircuitPoint {
            x: CircuitFieldElement {
                limbs: vec![F::zero(); self.ec.p_limbs.len()],
                limb_bits: self.ec.limb_bits,
            },
            y: CircuitFieldElement {
                limbs: vec![F::zero(); self.ec.p_limbs.len()],
                limb_bits: self.ec.limb_bits,
            },
            is_infinity: F::one(),
        }
    }
    
    /// Double a point (placeholder)
    fn point_double(&self, _point: &CircuitPoint<F>) -> Result<CircuitPoint<F>> {
        // TODO: Implement point doubling in circuit
        Ok(self.point_infinity())
    }
    
    /// Conditionally add two points based on a bit
    fn conditional_add(
        &self,
        _a: &CircuitPoint<F>,
        _b: &CircuitPoint<F>,
        _condition: F,
    ) -> Result<CircuitPoint<F>> {
        // TODO: Implement conditional point addition
        Ok(self.point_infinity())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use longfellow_algebra::Fp128;
    
    #[test]
    fn test_field_element_conversion() {
        let elem = FieldElement::one();
        let circuit_elem = CircuitFieldElement::<Fp128>::from_native(&elem, 16).unwrap();
        let back = circuit_elem.to_native().unwrap();
        
        assert_eq!(elem.to_bytes(), back.to_bytes());
    }
    
    #[test]
    fn test_point_conversion() {
        let point = Point::generator();
        let circuit_point = CircuitPoint::<Fp128>::from_native(&point, 16).unwrap();
        let back = circuit_point.to_native().unwrap();
        
        assert_eq!(point.x.to_bytes(), back.x.to_bytes());
        assert_eq!(point.y.to_bytes(), back.y.to_bytes());
        assert_eq!(point.is_infinity, back.is_infinity);
    }
}