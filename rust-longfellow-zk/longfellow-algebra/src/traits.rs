use longfellow_core::Result;
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};
use zeroize::Zeroize;

pub trait Field:
    Sized
    + Clone
    + Copy
    + Debug
    + Default
    + PartialEq
    + Eq
    + Send
    + Sync
    + 'static
    + ConditionallySelectable
    + ConstantTimeEq
    + Zeroize
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Neg<Output = Self>
    + for<'a> Add<&'a Self, Output = Self>
    + for<'a> Sub<&'a Self, Output = Self>
    + for<'a> Mul<&'a Self, Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + for<'a> AddAssign<&'a Self>
    + for<'a> SubAssign<&'a Self>
    + for<'a> MulAssign<&'a Self>
{
    const ZERO: Self;
    const ONE: Self;
    const MODULUS: &'static str;
    const MODULUS_BITS: u32;
    const CHAR_IS_TWO: bool = false;

    fn zero() -> Self {
        Self::ZERO
    }

    fn one() -> Self {
        Self::ONE
    }

    fn from_u64(val: u64) -> Self;
    
    fn from_bytes_le(bytes: &[u8]) -> Result<Self>;
    
    fn to_bytes_le(&self) -> Vec<u8>;

    fn invert(&self) -> Option<Self>;

    fn square(&self) -> Self {
        *self * self
    }

    fn double(&self) -> Self {
        *self + self
    }

    fn pow(&self, exp: &[u64]) -> Self {
        if exp.is_empty() {
            return Self::one();
        }
        
        let mut result = Self::one();
        let mut base = *self;
        
        for &limb in exp {
            let mut remaining = limb;
            while remaining > 0 {
                if remaining & 1 == 1 {
                    result *= &base;
                }
                base = base.square();
                remaining >>= 1;
            }
        }
        
        result
    }

    fn batch_invert(elements: &mut [Self]) {
        let mut products = Vec::with_capacity(elements.len());
        let mut acc = Self::one();

        for elem in elements.iter() {
            products.push(acc);
            acc *= elem;
        }

        if let Some(inv) = acc.invert() {
            let mut inv_acc = inv;
            for (elem, prod) in elements.iter_mut().zip(products.iter()).rev() {
                let tmp = *elem * inv_acc;
                *elem = *prod * inv_acc;
                inv_acc = tmp;
            }
        }
    }
    
    fn characteristic() -> u64 {
        // For prime fields, this would be the prime
        // For extension fields, this would be the characteristic of the base field
        // Default implementation assumes prime field
        0 // This should be overridden
    }
}

pub trait PrimeField: Field {
    type Repr: AsRef<[u8]> + AsMut<[u8]> + Default + Copy;

    fn from_repr(repr: Self::Repr) -> Option<Self>;
    
    fn to_repr(&self) -> Self::Repr;
    
    fn is_odd(&self) -> Choice;
}

pub trait FieldExtension: Field {
    type BaseField: Field;
    
    const DEGREE: usize;
    
    fn from_base_elements(elements: &[Self::BaseField]) -> Self;
    
    fn to_base_elements(&self) -> Vec<Self::BaseField>;
}