/// ECDSA signature operations for P-256

use crate::{Point, ScalarElement, FieldElement};
use longfellow_core::{LongfellowError, Result};
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// ECDSA signature
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct EcdsaSignature {
    /// R component
    pub r: ScalarElement,
    /// S component
    pub s: ScalarElement,
}

impl EcdsaSignature {
    /// Create a new signature
    pub fn new(r: ScalarElement, s: ScalarElement) -> Self {
        Self { r, s }
    }
    
    /// Parse from DER encoding
    pub fn from_der(bytes: &[u8]) -> Result<Self> {
        let sig = Signature::from_der(bytes)
            .map_err(|e| LongfellowError::ParseError(format!("Invalid DER signature: {}", e)))?;
        
        let (r_bytes, s_bytes) = sig.split_bytes();
        
        Ok(Self {
            r: ScalarElement::from_bytes(&r_bytes)?,
            s: ScalarElement::from_bytes(&s_bytes)?,
        })
    }
    
    /// Encode to DER
    pub fn to_der(&self) -> Vec<u8> {
        let sig = Signature::from_scalars(self.r.to_bytes(), self.s.to_bytes()).unwrap();
        sig.to_der().to_vec()
    }
    
    /// Parse from fixed-size encoding (64 bytes)
    pub fn from_bytes(bytes: &[u8; 64]) -> Result<Self> {
        let r = ScalarElement::from_bytes(&bytes[0..32].try_into().unwrap())?;
        let s = ScalarElement::from_bytes(&bytes[32..64].try_into().unwrap())?;
        Ok(Self { r, s })
    }
    
    /// Encode to fixed-size bytes
    pub fn to_bytes(&self) -> [u8; 64] {
        let mut bytes = [0u8; 64];
        bytes[0..32].copy_from_slice(&self.r.to_bytes());
        bytes[32..64].copy_from_slice(&self.s.to_bytes());
        bytes
    }
}

/// ECDSA public key
#[derive(Clone, Copy, Debug)]
pub struct PublicKey {
    point: Point,
}

impl PublicKey {
    /// Create from a point
    pub fn from_point(point: Point) -> Result<Self> {
        if point.is_infinity {
            return Err(LongfellowError::InvalidParameter(
                "Public key cannot be point at infinity".to_string()
            ));
        }
        
        if !point.is_on_curve() {
            return Err(LongfellowError::InvalidParameter(
                "Public key point must be on curve".to_string()
            ));
        }
        
        Ok(Self { point })
    }
    
    /// Get the underlying point
    pub fn point(&self) -> &Point {
        &self.point
    }
    
    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &EcdsaSignature) -> Result<bool> {
        // Use p256 crate for actual verification
        let verifying_key = VerifyingKey::from_encoded_point(
            &p256::EncodedPoint::from_affine_coordinates(
                &self.point.x.to_bytes().into(),
                &self.point.y.to_bytes().into(),
                false,
            )
        ).map_err(|e| LongfellowError::InvalidParameter(format!("Invalid public key: {}", e)))?;
        
        let sig = Signature::from_scalars(
            signature.r.to_bytes(),
            signature.s.to_bytes(),
        ).map_err(|e| LongfellowError::InvalidParameter(format!("Invalid signature: {}", e)))?;
        
        Ok(verifying_key.verify(message, &sig).is_ok())
    }
    
    /// Verify with explicit hash
    pub fn verify_prehashed(&self, hash: &[u8; 32], signature: &EcdsaSignature) -> Result<bool> {
        // ECDSA verification equation:
        // R = (hash * s^-1) * G + (r * s^-1) * PublicKey
        
        let s_inv = signature.s.invert()
            .ok_or_else(|| LongfellowError::InvalidParameter("Invalid signature: s = 0".to_string()))?;
        
        let z = ScalarElement::from_bytes_reduced(hash);
        let u1 = z.mul(&s_inv);
        let u2 = signature.r.mul(&s_inv);
        
        let g = Point::generator();
        let point1 = g.scalar_mul(&u1);
        let point2 = self.point.scalar_mul(&u2);
        let r_point = point1.add(&point2);
        
        if r_point.is_infinity {
            return Ok(false);
        }
        
        // Check if x-coordinate of R equals r (mod n)
        let r_x_reduced = ScalarElement::from_bytes_reduced(&r_point.x.to_bytes());
        Ok(r_x_reduced.to_bytes() == signature.r.to_bytes())
    }
}

/// Hash a message for ECDSA signing
pub fn hash_message(message: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.finalize().into()
}

/// ECDSA signature verification for circuit implementation
pub struct CircuitEcdsaVerifier;

impl CircuitEcdsaVerifier {
    /// Decompose ECDSA verification into circuit-friendly operations
    pub fn verify_components(
        public_key: &Point,
        message_hash: &[u8; 32],
        signature: &EcdsaSignature,
    ) -> Result<EcdsaVerificationComponents> {
        // Compute s^-1
        let s_inv = signature.s.invert()
            .ok_or_else(|| LongfellowError::InvalidParameter("Invalid signature: s = 0".to_string()))?;
        
        // Compute u1 = hash * s^-1
        let z = ScalarElement::from_bytes_reduced(message_hash);
        let u1 = z.mul(&s_inv);
        
        // Compute u2 = r * s^-1
        let u2 = signature.r.mul(&s_inv);
        
        // Points for scalar multiplication
        let g = Point::generator();
        
        Ok(EcdsaVerificationComponents {
            u1,
            u2,
            generator: g,
            public_key: *public_key,
            r: signature.r,
        })
    }
}

/// Components needed for ECDSA verification in a circuit
#[derive(Clone, Debug)]
pub struct EcdsaVerificationComponents {
    /// u1 = hash * s^-1
    pub u1: ScalarElement,
    /// u2 = r * s^-1
    pub u2: ScalarElement,
    /// Generator point
    pub generator: Point,
    /// Public key point
    pub public_key: Point,
    /// r component of signature
    pub r: ScalarElement,
}

impl EcdsaVerificationComponents {
    /// Complete the verification
    pub fn verify(&self) -> bool {
        // R = u1 * G + u2 * PublicKey
        let point1 = self.generator.scalar_mul(&self.u1);
        let point2 = self.public_key.scalar_mul(&self.u2);
        let r_point = point1.add(&point2);
        
        if r_point.is_infinity {
            return false;
        }
        
        // Check if x-coordinate of R equals r (mod n)
        let r_x_reduced = ScalarElement::from_bytes_reduced(&r_point.x.to_bytes());
        r_x_reduced.to_bytes() == self.r.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;
    
    #[test]
    fn test_signature_encoding() {
        let r = ScalarElement::from_bytes(&[0u8; 31].iter().chain(&[1u8]).cloned().collect::<Vec<_>>().try_into().unwrap()).unwrap();
        let s = ScalarElement::from_bytes(&[0u8; 31].iter().chain(&[2u8]).cloned().collect::<Vec<_>>().try_into().unwrap()).unwrap();
        
        let sig = EcdsaSignature::new(r, s);
        
        // Test fixed-size encoding
        let bytes = sig.to_bytes();
        let decoded = EcdsaSignature::from_bytes(&bytes).unwrap();
        
        assert_eq!(sig.r.to_bytes(), decoded.r.to_bytes());
        assert_eq!(sig.s.to_bytes(), decoded.s.to_bytes());
        
        // Test DER encoding
        let der = sig.to_der();
        let decoded_der = EcdsaSignature::from_der(&der).unwrap();
        
        assert_eq!(sig.r.to_bytes(), decoded_der.r.to_bytes());
        assert_eq!(sig.s.to_bytes(), decoded_der.s.to_bytes());
    }
    
    #[test]
    fn test_hash_message() {
        let message = b"test message";
        let hash = hash_message(message);
        
        // Known SHA-256 hash of "test message"
        let expected = hex::decode("3f0a377ba0a4a460ecb616f6507ce0d8cfa3e704025d4fda3ed0c5ca05468728").unwrap();
        assert_eq!(&hash[..], &expected[..]);
    }
}