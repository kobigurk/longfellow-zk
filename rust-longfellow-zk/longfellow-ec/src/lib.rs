/// Elliptic curve operations for P-256 (secp256r1)
/// 
/// This module provides elliptic curve operations needed for ECDSA verification
/// and other cryptographic operations in zero-knowledge circuits.

use longfellow_core::{LongfellowError, Result};
use longfellow_algebra::traits::Field;
use p256::{
    AffinePoint, ProjectivePoint, Scalar,
    elliptic_curve::{
        group::{GroupEncoding, Group},
        sec1::{FromEncodedPoint, ToEncodedPoint},
        Field as ECField,
    },
};
use serde::{Deserialize, Serialize};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

pub mod ecdsa;
pub mod circuit;

/// P-256 field element (for coordinates)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldElement(p256::FieldElement);

impl FieldElement {
    /// Create from bytes (big-endian)
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        p256::FieldElement::from_bytes(bytes.into())
            .map(FieldElement)
            .ok_or_else(|| LongfellowError::InvalidParameter(
                "Invalid field element bytes".to_string()
            ))
    }
    
    /// Convert to bytes (big-endian)
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes().into()
    }
    
    /// Get zero element
    pub fn zero() -> Self {
        FieldElement(p256::FieldElement::ZERO)
    }
    
    /// Get one element
    pub fn one() -> Self {
        FieldElement(p256::FieldElement::ONE)
    }
    
    /// Add two field elements
    pub fn add(&self, other: &Self) -> Self {
        FieldElement(self.0 + other.0)
    }
    
    /// Subtract two field elements
    pub fn sub(&self, other: &Self) -> Self {
        FieldElement(self.0 - other.0)
    }
    
    /// Multiply two field elements
    pub fn mul(&self, other: &Self) -> Self {
        FieldElement(self.0 * other.0)
    }
    
    /// Square a field element
    pub fn square(&self) -> Self {
        FieldElement(self.0.square())
    }
    
    /// Invert a field element
    pub fn invert(&self) -> Option<Self> {
        self.0.invert().map(FieldElement).into()
    }
    
    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.0.is_zero().into()
    }
}

/// P-256 scalar (for private keys and nonces)
#[derive(Clone, Copy, Debug)]
pub struct ScalarElement(Scalar);

impl ScalarElement {
    /// Create from bytes (big-endian)
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        Option::from(Scalar::from_bytes(bytes.into()))
            .map(ScalarElement)
            .ok_or_else(|| LongfellowError::InvalidParameter(
                "Invalid scalar bytes".to_string()
            ))
    }
    
    /// Create from bytes with reduction
    pub fn from_bytes_reduced(bytes: &[u8; 32]) -> Self {
        ScalarElement(Scalar::from_bytes_reduced(bytes.into()))
    }
    
    /// Convert to bytes (big-endian)
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes().into()
    }
    
    /// Get zero scalar
    pub fn zero() -> Self {
        ScalarElement(Scalar::ZERO)
    }
    
    /// Get one scalar
    pub fn one() -> Self {
        ScalarElement(Scalar::ONE)
    }
    
    /// Add two scalars
    pub fn add(&self, other: &Self) -> Self {
        ScalarElement(self.0 + other.0)
    }
    
    /// Subtract two scalars
    pub fn sub(&self, other: &Self) -> Self {
        ScalarElement(self.0 - other.0)
    }
    
    /// Multiply two scalars
    pub fn mul(&self, other: &Self) -> Self {
        ScalarElement(self.0 * other.0)
    }
    
    /// Invert a scalar
    pub fn invert(&self) -> Option<Self> {
        self.0.invert().map(ScalarElement).into()
    }
}

/// P-256 point in affine coordinates
#[derive(Clone, Copy, Debug)]
pub struct Point {
    /// X coordinate
    pub x: FieldElement,
    /// Y coordinate
    pub y: FieldElement,
    /// Is this the point at infinity?
    pub is_infinity: bool,
}

impl Point {
    /// Create a new point
    pub fn new(x: FieldElement, y: FieldElement) -> Result<Self> {
        let point = Self {
            x,
            y,
            is_infinity: false,
        };
        
        // Verify point is on curve
        if !point.is_on_curve() {
            return Err(LongfellowError::InvalidParameter(
                "Point is not on curve".to_string()
            ));
        }
        
        Ok(point)
    }
    
    /// Get the point at infinity
    pub fn infinity() -> Self {
        Self {
            x: FieldElement::zero(),
            y: FieldElement::zero(),
            is_infinity: true,
        }
    }
    
    /// Get the generator point
    pub fn generator() -> Self {
        let gen = AffinePoint::GENERATOR;
        let coords = gen.to_encoded_point(false);
        
        Self {
            x: FieldElement(p256::FieldElement::from_bytes(
                coords.x().unwrap().as_slice().try_into().unwrap()
            ).unwrap()),
            y: FieldElement(p256::FieldElement::from_bytes(
                coords.y().unwrap().as_slice().try_into().unwrap()
            ).unwrap()),
            is_infinity: false,
        }
    }
    
    /// Check if point is on curve: y² = x³ - 3x + b
    pub fn is_on_curve(&self) -> bool {
        if self.is_infinity {
            return true;
        }
        
        let y2 = self.y.square();
        let x3 = self.x.square().mul(&self.x);
        let ax = self.x.mul(&FieldElement(p256::FieldElement::from(3u64).neg()));
        let b = FieldElement(p256::FieldElement::from_bytes(
            &hex::decode("5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b")
                .unwrap().try_into().unwrap()
        ).unwrap());
        
        let rhs = x3.add(&ax).add(&b);
        y2 == rhs
    }
    
    /// Convert to projective coordinates
    pub fn to_projective(&self) -> ProjectivePoint {
        if self.is_infinity {
            ProjectivePoint::IDENTITY
        } else {
            let encoded = p256::EncodedPoint::from_affine_coordinates(
                &self.x.0.to_bytes(),
                &self.y.0.to_bytes(),
                false,
            );
            
            Option::from(AffinePoint::from_encoded_point(&encoded))
                .map(|a| a.into())
                .unwrap_or(ProjectivePoint::IDENTITY)
        }
    }
    
    /// Double a point
    pub fn double(&self) -> Self {
        if self.is_infinity {
            return *self;
        }
        
        let proj = self.to_projective().double();
        Self::from_projective(&proj)
    }
    
    /// Add two points
    pub fn add(&self, other: &Self) -> Self {
        if self.is_infinity {
            return *other;
        }
        if other.is_infinity {
            return *self;
        }
        
        let p1 = self.to_projective();
        let p2 = other.to_projective();
        let sum = p1 + p2;
        
        Self::from_projective(&sum)
    }
    
    /// Scalar multiplication
    pub fn scalar_mul(&self, scalar: &ScalarElement) -> Self {
        if self.is_infinity {
            return *self;
        }
        
        let proj = self.to_projective();
        let result = proj * scalar.0;
        
        Self::from_projective(&result)
    }
    
    /// Convert from projective point
    fn from_projective(proj: &ProjectivePoint) -> Self {
        if bool::from(proj.is_identity()) {
            Self::infinity()
        } else {
            let affine = proj.to_affine();
            let encoded = affine.to_encoded_point(false);
            
            Self {
                x: FieldElement(p256::FieldElement::from_bytes(
                    encoded.x().unwrap().as_slice().try_into().unwrap()
                ).unwrap()),
                y: FieldElement(p256::FieldElement::from_bytes(
                    encoded.y().unwrap().as_slice().try_into().unwrap()
                ).unwrap()),
                is_infinity: false,
            }
        }
    }
    
    /// Encode point to bytes (uncompressed)
    pub fn to_bytes_uncompressed(&self) -> [u8; 65] {
        if self.is_infinity {
            let mut bytes = [0u8; 65];
            bytes[0] = 0x00; // Infinity encoding
            bytes
        } else {
            let mut bytes = [0u8; 65];
            bytes[0] = 0x04; // Uncompressed
            bytes[1..33].copy_from_slice(&self.x.to_bytes());
            bytes[33..65].copy_from_slice(&self.y.to_bytes());
            bytes
        }
    }
    
    /// Decode point from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(LongfellowError::InvalidParameter(
                "Empty point bytes".to_string()
            ));
        }
        
        match bytes[0] {
            0x00 => Ok(Self::infinity()),
            0x04 if bytes.len() == 65 => {
                let x = FieldElement::from_bytes(bytes[1..33].try_into().unwrap())?;
                let y = FieldElement::from_bytes(bytes[33..65].try_into().unwrap())?;
                Self::new(x, y)
            }
            _ => Err(LongfellowError::InvalidParameter(
                "Invalid point encoding".to_string()
            )),
        }
    }
}

/// Convert field element to algebra field trait
pub fn field_elem_to_algebra<F: Field>(elem: &FieldElement) -> Result<F> {
    let bytes = elem.to_bytes();
    F::from_bytes_be(&bytes)
}

/// Convert algebra field to field element
pub fn algebra_to_field_elem<F: Field>(elem: &F) -> Result<FieldElement> {
    let bytes = elem.to_bytes_be();
    if bytes.len() < 32 {
        let mut padded = [0u8; 32];
        padded[32 - bytes.len()..].copy_from_slice(&bytes);
        FieldElement::from_bytes(&padded)
    } else {
        FieldElement::from_bytes(&bytes[..32].try_into().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generator_on_curve() {
        let gen = Point::generator();
        assert!(gen.is_on_curve());
        assert!(!gen.is_infinity);
    }
    
    #[test]
    fn test_point_addition() {
        let gen = Point::generator();
        let double = gen.double();
        let sum = gen.add(&gen);
        
        // Double and add should give same result
        assert_eq!(double.x, sum.x);
        assert_eq!(double.y, sum.y);
        assert_eq!(double.is_infinity, sum.is_infinity);
    }
    
    #[test]
    fn test_scalar_multiplication() {
        let gen = Point::generator();
        let scalar = ScalarElement::from_bytes(&[
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 5,
        ]).unwrap();
        
        let result = gen.scalar_mul(&scalar);
        
        // 5*G should equal G + G + G + G + G
        let expected = gen.add(&gen).add(&gen).add(&gen).add(&gen);
        assert_eq!(result.x, expected.x);
        assert_eq!(result.y, expected.y);
    }
    
    #[test]
    fn test_point_encoding() {
        let gen = Point::generator();
        let bytes = gen.to_bytes_uncompressed();
        let decoded = Point::from_bytes(&bytes).unwrap();
        
        assert_eq!(gen.x, decoded.x);
        assert_eq!(gen.y, decoded.y);
        assert_eq!(gen.is_infinity, decoded.is_infinity);
    }
}