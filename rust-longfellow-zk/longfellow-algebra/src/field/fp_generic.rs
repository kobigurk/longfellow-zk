use crate::nat::{self, Limb, Nat};
use crate::traits::Field;
use longfellow_core::{LongfellowError, Result};
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};
use zeroize::Zeroize;

pub trait FieldReduction<const N: usize>: Copy + Send + Sync + 'static {
    const MODULUS: Nat<N>;
    const MODULUS_STR: &'static str;
    const MODULUS_BITS: u32;
    const R: Nat<N>;
    const R2: Nat<N>;
    const INV: Limb;
    
    fn reduction_step(a: &mut [Limb], mprime: Limb, modulus: &Nat<N>);
}

#[derive(Clone, Copy, Zeroize)]
pub struct FpGeneric<const N: usize, R: FieldReduction<N>> {
    value: Nat<N>,
    _phantom: PhantomData<R>,
}

impl<const N: usize, R: FieldReduction<N>> FpGeneric<N, R> {
    pub const MODULUS: Nat<N> = R::MODULUS;
    pub const MODULUS_STR: &'static str = R::MODULUS_STR;
    pub const MODULUS_BITS: u32 = R::MODULUS_BITS;
    pub const R: Nat<N> = R::R;
    pub const R2: Nat<N> = R::R2;
    pub const INV: Limb = R::INV;

    pub const ZERO: Self = Self {
        value: Nat::ZERO,
        _phantom: PhantomData,
    };

    pub const ONE: Self = Self {
        value: Self::R,
        _phantom: PhantomData,
    };

    #[inline]
    pub fn to_montgomery(value: Nat<N>) -> Self {
        let mut result = Self {
            value,
            _phantom: PhantomData,
        };
        result.mul_assign(&Self {
            value: Self::R2,
            _phantom: PhantomData,
        });
        result
    }

    #[inline]
    pub fn from_montgomery(&self) -> Nat<N> {
        let mut result = self.value;
        Self::montgomery_reduce::<N, R>(&mut result);
        result
    }

    #[inline]
    fn montgomery_reduce<const M: usize, Red: FieldReduction<M>>(a: &mut Nat<M>) {
        let mut extended = vec![0 as Limb; M * 2];
        extended[..M].copy_from_slice(&a.limbs);
        
        for i in 0..M {
            let k = extended[i].wrapping_mul(Red::INV);
            let mut carry = 0;
            
            for j in 0..M {
                let (lo, hi) = nat::mul_wide(k, Red::MODULUS.limbs[j]);
                let (sum1, c1) = nat::add_with_carry(extended[i + j], lo, carry);
                extended[i + j] = sum1;
                carry = hi.wrapping_add(c1);
            }
            
            extended[i + M] = extended[i + M].wrapping_add(carry);
        }
        
        a.limbs.copy_from_slice(&extended[M..]);
        
        let borrow = a.sub_with_borrow(&Red::MODULUS);
        a.conditional_add(&Red::MODULUS, Choice::from(borrow as u8));
    }

    #[inline]
    fn add_reduce(&mut self, other: &Self) {
        let carry = self.value.add_with_carry(&other.value);
        let borrow = self.value.sub_with_borrow(&Self::MODULUS);
        self.value
            .conditional_add(&Self::MODULUS, Choice::from((carry == 0 && borrow != 0) as u8));
    }

    #[inline]
    fn sub_reduce(&mut self, other: &Self) {
        let borrow = self.value.sub_with_borrow(&other.value);
        self.value
            .conditional_add(&Self::MODULUS, Choice::from(borrow as u8));
    }

    #[inline]
    fn mul_montgomery(&mut self, other: &Self) {
        let mut result = vec![0 as Limb; N * 2];
        
        for i in 0..N {
            let mut carry = 0;
            for j in 0..N {
                let (lo, hi) = nat::mul_wide(self.value.limbs[i], other.value.limbs[j]);
                let (sum, c) = nat::add_with_carry(result[i + j], lo, carry);
                result[i + j] = sum;
                carry = hi.wrapping_add(c);
            }
            result[i + N] = carry;
        }
        
        let _reduced: Nat<N> = Nat::new(result[..N].try_into().unwrap());
        for i in 0..N {
            R::reduction_step(&mut result[i..], Self::INV, &Self::MODULUS);
        }
        
        self.value.limbs.copy_from_slice(&result[N..]);
        let borrow = self.value.sub_with_borrow(&Self::MODULUS);
        self.value
            .conditional_add(&Self::MODULUS, Choice::from(borrow as u8));
    }

    pub fn square(&self) -> Self {
        let mut result = *self;
        result.mul_assign(self);
        result
    }

    pub fn invert(&self) -> Option<Self> {
        let mut a = self.from_montgomery();
        let mut b = Self::MODULUS;
        let mut u = Self::ONE.value;
        let mut v = Nat::ZERO;

        while !bool::from(a.is_zero()) {
            if bool::from(a.is_even()) {
                a.shr1();
                if bool::from(u.is_odd()) {
                    let carry = u.add_with_carry(&Self::MODULUS);
                    u.shr1();
                    if carry != 0 {
                        u.limbs[N - 1] |= 1 << (Limb::BITS - 1);
                    }
                } else {
                    u.shr1();
                }
            } else {
                if a < b {
                    std::mem::swap(&mut a, &mut b);
                    std::mem::swap(&mut u, &mut v);
                }
                a.sub_with_borrow(&b);
                let borrow = u.sub_with_borrow(&v);
                u.conditional_add(&Self::MODULUS, Choice::from(borrow as u8));
            }
        }

        if b != Nat::ONE {
            None
        } else {
            Some(Self::to_montgomery(v))
        }
    }

    /// Convert from little-endian bytes
    pub fn from_bytes_le(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > N * 8 {
            return Err(LongfellowError::InvalidParameter(
                "Byte array too long for field element".to_string()
            ));
        }
        
        let mut limbs = [0u64; N];
        for (i, chunk) in bytes.chunks(8).enumerate() {
            if i >= N {
                break;
            }
            let mut bytes_array = [0u8; 8];
            bytes_array[..chunk.len()].copy_from_slice(chunk);
            limbs[i] = u64::from_le_bytes(bytes_array);
        }
        
        let value = Nat::new(limbs);
        if value >= Self::MODULUS {
            return Err(LongfellowError::InvalidParameter(
                "Value exceeds field modulus".to_string()
            ));
        }
        
        Ok(Self::to_montgomery(value))
    }
    
    /// Convert to little-endian bytes
    pub fn to_bytes_le(&self) -> Vec<u8> {
        let value = self.from_montgomery();
        let mut bytes = Vec::with_capacity(N * 8);
        
        for i in 0..N {
            bytes.extend_from_slice(&value.limbs[i].to_le_bytes());
        }
        
        // Remove trailing zeros
        while bytes.last() == Some(&0) && bytes.len() > 1 {
            bytes.pop();
        }
        
        bytes
    }
}

impl<const N: usize, R: FieldReduction<N>> Field for FpGeneric<N, R> {
    const ZERO: Self = Self::ZERO;
    const ONE: Self = Self::ONE;
    const MODULUS: &'static str = R::MODULUS_STR;
    const MODULUS_BITS: u32 = R::MODULUS_BITS;

    fn zero() -> Self {
        Self::ZERO
    }

    fn one() -> Self {
        Self::ONE
    }

    fn from_u64(val: u64) -> Self {
        Self::to_montgomery(Nat::from_u64(val))
    }

    fn from_bytes_le(bytes: &[u8]) -> Result<Self> {
        let nat = Nat::from_bytes_le(bytes)
            .ok_or_else(|| LongfellowError::InvalidParameter("Invalid bytes".to_string()))?;
        if nat >= Self::MODULUS {
            return Err(LongfellowError::InvalidParameter(
                "Value exceeds modulus".to_string(),
            ));
        }
        Ok(Self::to_montgomery(nat))
    }

    fn to_bytes_le(&self) -> Vec<u8> {
        self.from_montgomery().to_bytes_le()
    }

    fn invert(&self) -> Option<Self> {
        self.invert()
    }
}

impl<const N: usize, R: FieldReduction<N>> ConstantTimeEq for FpGeneric<N, R> {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.value.ct_eq(&other.value)
    }
}

impl<const N: usize, R: FieldReduction<N>> ConditionallySelectable for FpGeneric<N, R> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self {
            value: Nat::conditional_select(&a.value, &b.value, choice),
            _phantom: PhantomData,
        }
    }
}

impl<const N: usize, R: FieldReduction<N>> Default for FpGeneric<N, R> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<const N: usize, R: FieldReduction<N>> PartialEq for FpGeneric<N, R> {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

impl<const N: usize, R: FieldReduction<N>> Eq for FpGeneric<N, R> {}

impl<const N: usize, R: FieldReduction<N>> Add for FpGeneric<N, R> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self.add_reduce(&rhs);
        self
    }
}

impl<const N: usize, R: FieldReduction<N>> AddAssign for FpGeneric<N, R> {
    fn add_assign(&mut self, rhs: Self) {
        self.add_reduce(&rhs);
    }
}

impl<const N: usize, R: FieldReduction<N>> Sub for FpGeneric<N, R> {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self {
        self.sub_reduce(&rhs);
        self
    }
}

impl<const N: usize, R: FieldReduction<N>> SubAssign for FpGeneric<N, R> {
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_reduce(&rhs);
    }
}

impl<const N: usize, R: FieldReduction<N>> Mul for FpGeneric<N, R> {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self {
        self.mul_montgomery(&rhs);
        self
    }
}

impl<const N: usize, R: FieldReduction<N>> MulAssign for FpGeneric<N, R> {
    fn mul_assign(&mut self, rhs: Self) {
        self.mul_montgomery(&rhs);
    }
}

impl<const N: usize, R: FieldReduction<N>> Neg for FpGeneric<N, R> {
    type Output = Self;

    fn neg(self) -> Self {
        Self::ZERO - self
    }
}

impl<const N: usize, R: FieldReduction<N>> Add<&Self> for FpGeneric<N, R> {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self {
        self + *rhs
    }
}

impl<const N: usize, R: FieldReduction<N>> AddAssign<&Self> for FpGeneric<N, R> {
    fn add_assign(&mut self, rhs: &Self) {
        *self += *rhs;
    }
}

impl<const N: usize, R: FieldReduction<N>> Sub<&Self> for FpGeneric<N, R> {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self {
        self - *rhs
    }
}

impl<const N: usize, R: FieldReduction<N>> SubAssign<&Self> for FpGeneric<N, R> {
    fn sub_assign(&mut self, rhs: &Self) {
        *self -= *rhs;
    }
}

impl<const N: usize, R: FieldReduction<N>> Mul<&Self> for FpGeneric<N, R> {
    type Output = Self;

    fn mul(self, rhs: &Self) -> Self {
        self * *rhs
    }
}

impl<const N: usize, R: FieldReduction<N>> MulAssign<&Self> for FpGeneric<N, R> {
    fn mul_assign(&mut self, rhs: &Self) {
        *self *= *rhs;
    }
}

impl<const N: usize, R: FieldReduction<N>> std::fmt::Debug for FpGeneric<N, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FpGeneric({:?})", self.from_montgomery())
    }
}