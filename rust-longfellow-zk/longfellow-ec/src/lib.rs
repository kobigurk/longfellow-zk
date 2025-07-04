/// Elliptic curve operations for P-256 (secp256r1)
/// 
/// This module provides elliptic curve operations needed for ECDSA verification
/// and other cryptographic operations in zero-knowledge circuits.

use longfellow_core::{LongfellowError, Result};
use longfellow_algebra::traits::Field;
use p256::{
    AffinePoint, ProjectivePoint, Scalar,
    elliptic_curve::{
        group::Group,
        sec1::{FromEncodedPoint, ToEncodedPoint},
    },
};
use serde::{Deserialize, Serialize};

pub mod ecdsa;
pub mod circuit;

/// P-256 field element (for coordinates)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldElement(p256::FieldElement);

impl FieldElement {
    /// Create from bytes (big-endian)
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let ct_option = p256::FieldElement::from_bytes(bytes.into());
        if ct_option.is_some().into() {
            Ok(FieldElement(ct_option.unwrap()))
        } else {
            Err(LongfellowError::InvalidParameter(
                "Invalid field element bytes".to_string()
            ))
        }
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
        use p256::elliptic_curve::ff::PrimeField;
        
        // p256 uses little-endian internally, so we need to reverse
        let mut le_bytes = *bytes;
        le_bytes.reverse();
        
        Option::from(Scalar::from_repr(le_bytes.into()))
            .map(ScalarElement)
            .ok_or_else(|| LongfellowError::InvalidParameter(
                "Invalid scalar bytes".to_string()
            ))
    }
    
    /// Create from bytes with reduction
    pub fn from_bytes_reduced(bytes: &[u8; 32]) -> Self {
        use p256::elliptic_curve::ops::Reduce;
        use p256::U256;
        
        // Create U256 from big-endian bytes
        let mut le_bytes = [0u8; 32];
        for i in 0..32 {
            le_bytes[i] = bytes[31 - i];
        }
        
        let uint = U256::from_le_slice(&le_bytes);
        ScalarElement(Scalar::reduce(uint))
    }
    
    /// Convert to bytes (big-endian)
    pub fn to_bytes(&self) -> [u8; 32] {
        use p256::elliptic_curve::ff::PrimeField;
        
        let le_bytes: [u8; 32] = self.0.to_repr().into();
        let mut be_bytes = le_bytes;
        be_bytes.reverse();
        be_bytes
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

impl Serialize for ScalarElement {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = self.to_bytes();
        bytes.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ScalarElement {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: [u8; 32] = Deserialize::deserialize(deserializer)?;
        ScalarElement::from_bytes(&bytes)
            .map_err(|_| serde::de::Error::custom("Invalid scalar bytes"))
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
        let b_bytes = [
            0x5a, 0xc6, 0x35, 0xd8, 0xaa, 0x3a, 0x93, 0xe7,
            0xb3, 0xeb, 0xbd, 0x55, 0x76, 0x98, 0x86, 0xbc,
            0x65, 0x1d, 0x06, 0xb0, 0xcc, 0x53, 0xb0, 0xf6,
            0x3b, 0xce, 0x3c, 0x3e, 0x27, 0xd2, 0x60, 0x4b,
        ];
        let b = FieldElement::from_bytes(&b_bytes).unwrap();
        
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
                .map(|a: AffinePoint| a.into())
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
    // Convert big-endian to little-endian
    let mut le_bytes = bytes;
    le_bytes.reverse();
    F::from_bytes_le(&le_bytes)
}

/// Convert algebra field to field element
pub fn algebra_to_field_elem<F: Field>(elem: &F) -> Result<FieldElement> {
    let le_bytes = elem.to_bytes_le();
    if le_bytes.len() > 32 {
        return Err(LongfellowError::InvalidParameter(
            "Field element too large".to_string()
        ));
    }
    
    // Convert little-endian to big-endian and pad
    let mut be_bytes = [0u8; 32];
    for (i, &byte) in le_bytes.iter().enumerate() {
        be_bytes[31 - i] = byte;
    }
    
    FieldElement::from_bytes(&be_bytes)
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