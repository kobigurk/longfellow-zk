use std::ops::{Add, Sub, Mul, Neg};
use longfellow_algebra::Field;
use longfellow_core::Error as LongfellowError;

/// A simple field implementation for testing that uses regular modular arithmetic
/// This is NOT efficient but it works correctly for small values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimpleField {
    value: u128,
}

// Use a small prime for testing: 2^31 - 1 = 2147483647
const MODULUS: u128 = 2147483647;

impl SimpleField {
    pub fn new(value: u128) -> Self {
        SimpleField { value: value % MODULUS }
    }
    
    pub fn from_u64(value: u64) -> Self {
        SimpleField::new(value as u128)
    }
}

impl Field for SimpleField {
    fn zero() -> Self {
        SimpleField { value: 0 }
    }
    
    fn one() -> Self {
        SimpleField { value: 1 }
    }
    
    fn is_zero(&self) -> bool {
        self.value == 0
    }
    
    fn double(&self) -> Self {
        SimpleField::new((self.value * 2) % MODULUS)
    }
    
    fn square(&self) -> Self {
        SimpleField::new((self.value * self.value) % MODULUS)
    }
    
    fn inverse(&self) -> Result<Self, LongfellowError> {
        if self.is_zero() {
            return Err(LongfellowError::InvalidParameter("Cannot invert zero".into()));
        }
        
        // Extended Euclidean algorithm for modular inverse
        let mut a = self.value as i128;
        let mut b = MODULUS as i128;
        let mut x0 = 1i128;
        let mut x1 = 0i128;
        
        while b != 0 {
            let q = a / b;
            let temp = b;
            b = a % b;
            a = temp;
            
            let temp = x1;
            x1 = x0 - q * x1;
            x0 = temp;
        }
        
        if x0 < 0 {
            x0 += MODULUS as i128;
        }
        
        Ok(SimpleField::new(x0 as u128))
    }
    
    fn from_bytes_le(bytes: &[u8]) -> Result<Self, LongfellowError> {
        if bytes.len() > 16 {
            return Err(LongfellowError::InvalidParameter("Too many bytes".into()));
        }
        
        let mut value = 0u128;
        for (i, &byte) in bytes.iter().enumerate() {
            value |= (byte as u128) << (i * 8);
        }
        
        Ok(SimpleField::new(value))
    }
    
    fn to_bytes_le(&self) -> Vec<u8> {
        self.value.to_le_bytes().to_vec()
    }
}

impl Add for SimpleField {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        SimpleField::new((self.value + other.value) % MODULUS)
    }
}

impl Sub for SimpleField {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        let result = if self.value >= other.value {
            self.value - other.value
        } else {
            MODULUS - (other.value - self.value)
        };
        SimpleField::new(result)
    }
}

impl Mul for SimpleField {
    type Output = Self;
    
    fn mul(self, other: Self) -> Self {
        SimpleField::new((self.value * other.value) % MODULUS)
    }
}

impl Neg for SimpleField {
    type Output = Self;
    
    fn neg(self) -> Self {
        if self.value == 0 {
            self
        } else {
            SimpleField::new(MODULUS - self.value)
        }
    }
}

impl std::ops::AddAssign for SimpleField {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl std::ops::SubAssign for SimpleField {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl std::ops::MulAssign for SimpleField {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_arithmetic() {
        let a = SimpleField::from_u64(5);
        let b = SimpleField::from_u64(7);
        let c = SimpleField::from_u64(12);
        
        assert_eq!(a + b, c);
        assert_eq!(c - b, a);
        assert_eq!(c - a, b);
    }
    
    #[test]
    fn test_constraint() {
        let w0 = SimpleField::from_u64(5);
        let w1 = SimpleField::from_u64(7);
        let w2 = SimpleField::from_u64(12);
        
        let result = w0 + w1 - w2;
        assert_eq!(result, SimpleField::zero());
    }
}