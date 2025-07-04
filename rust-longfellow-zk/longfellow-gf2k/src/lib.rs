/// GF(2^128) - Galois Field implementation
/// 
/// This module implements arithmetic in GF(2^128) using the irreducible polynomial
/// x^128 + x^7 + x^2 + x + 1

use longfellow_core::{LongfellowError, Result};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};
use zeroize::Zeroize;

/// GF(2^128) element represented as two 64-bit words
#[derive(Clone, Copy, Zeroize)]
#[repr(C, align(16))]
pub struct Gf2_128 {
    pub lo: u64,
    pub hi: u64,
}

impl Gf2_128 {
    pub const ZERO: Self = Self { lo: 0, hi: 0 };
    pub const ONE: Self = Self { lo: 1, hi: 0 };
    
    /// The irreducible polynomial: x^128 + x^7 + x^2 + x + 1
    /// In bit representation: 0x87 (lower 8 bits)
    const REDUCTION_POLY: u64 = 0x87;

    pub fn new(lo: u64, hi: u64) -> Self {
        Self { lo, hi }
    }

    pub fn from_bytes_le(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 16 {
            return Err(LongfellowError::InvalidParameter(
                "GF2^128 requires exactly 16 bytes".to_string(),
            ));
        }
        
        let lo = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let hi = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        
        Ok(Self { lo, hi })
    }

    pub fn to_bytes_le(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&self.lo.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.hi.to_le_bytes());
        bytes
    }

    pub fn is_zero(&self) -> Choice {
        self.ct_eq(&Self::ZERO)
    }

    /// Multiplication in GF(2^128) using CLMUL instruction when available
    #[inline]
    pub fn mul_clmul(&self, other: &Self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "pclmulqdq"))]
        {
            unsafe { self.mul_clmul_impl(other) }
        }
        #[cfg(not(all(target_arch = "x86_64", target_feature = "pclmulqdq")))]
        {
            self.mul_portable(other)
        }
    }

    /// Portable multiplication implementation
    pub fn mul_portable(&self, other: &Self) -> Self {
        // Karatsuba multiplication for 128-bit values
        let a0 = self.lo;
        let a1 = self.hi;
        let b0 = other.lo;
        let b1 = other.hi;

        // Products
        let p0 = gf2_mul_64(a0, b0);
        let p1 = gf2_mul_64(a1, b1);
        let p2 = gf2_mul_64(a0 ^ a1, b0 ^ b1);

        // Combine products
        let mut r0 = p0.0;
        let mut r1 = p0.1 ^ p1.0 ^ p2.0 ^ p0.0;
        let mut r2 = p1.0 ^ p2.1 ^ p0.1 ^ p1.1;
        let mut r3 = p1.1;

        // Reduce modulo the irreducible polynomial
        reduce_gf2_128(&mut r0, &mut r1, &mut r2, &mut r3);

        Self { lo: r0, hi: r1 }
    }

    /// CLMUL-based multiplication (x86_64 with PCLMULQDQ)
    #[cfg(all(target_arch = "x86_64", target_feature = "pclmulqdq"))]
    #[target_feature(enable = "pclmulqdq")]
    unsafe fn mul_clmul_impl(&self, other: &Self) -> Self {
        use std::arch::x86_64::*;

        let a = _mm_set_epi64x(self.hi as i64, self.lo as i64);
        let b = _mm_set_epi64x(other.hi as i64, other.lo as i64);

        // Karatsuba multiplication using CLMUL
        let p0 = _mm_clmulepi64_si128(a, b, 0x00); // a0 * b0
        let p1 = _mm_clmulepi64_si128(a, b, 0x11); // a1 * b1
        let p2 = _mm_clmulepi64_si128(a, b, 0x10); // a0 * b1
        let p3 = _mm_clmulepi64_si128(a, b, 0x01); // a1 * b0

        // Combine middle terms
        let middle = _mm_xor_si128(p2, p3);

        // Reduction
        let mut result = [0u64; 4];
        _mm_storeu_si128(result.as_mut_ptr() as *mut __m128i, p0);
        let t = _mm_extract_epi64(middle, 0) as u64;
        result[1] ^= t;
        let t = _mm_extract_epi64(middle, 1) as u64;
        result[2] ^= t;
        _mm_storeu_si128((result.as_mut_ptr() as *mut __m128i).add(1), p1);

        reduce_gf2_128(&mut result[0], &mut result[1], &mut result[2], &mut result[3]);

        Self {
            lo: result[0],
            hi: result[1],
        }
    }

    /// Square operation (more efficient than general multiplication)
    pub fn square(&self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "pclmulqdq"))]
        {
            unsafe {
                use std::arch::x86_64::*;
                
                let a = _mm_set_epi64x(self.hi as i64, self.lo as i64);
                let sq_lo = _mm_clmulepi64_si128(a, a, 0x00);
                let sq_hi = _mm_clmulepi64_si128(a, a, 0x11);
                
                let mut result = [0u64; 4];
                _mm_storeu_si128(result.as_mut_ptr() as *mut __m128i, sq_lo);
                _mm_storeu_si128((result.as_mut_ptr() as *mut __m128i).add(1), sq_hi);
                
                reduce_gf2_128(&mut result[0], &mut result[1], &mut result[2], &mut result[3]);
                
                Self {
                    lo: result[0],
                    hi: result[1],
                }
            }
        }
        #[cfg(not(all(target_arch = "x86_64", target_feature = "pclmulqdq")))]
        {
            self.mul_portable(self)
        }
    }

    /// Inversion using extended binary GCD
    pub fn invert(&self) -> Option<Self> {
        if self.is_zero().into() {
            return None;
        }

        // Extended binary GCD for GF(2^128)
        let mut u = *self;
        let mut v = Self { lo: Self::REDUCTION_POLY, hi: 1u64 << 63 }; // x^128 + reduction poly
        let mut g1 = Self::ONE;
        let mut g2 = Self::ZERO;

        while !u.is_zero().into() {
            // Find the degree (position of highest bit)
            let deg_u = u.degree();
            let deg_v = v.degree();

            if deg_u < deg_v {
                std::mem::swap(&mut u, &mut v);
                std::mem::swap(&mut g1, &mut g2);
            } else {
                let shift = deg_u - deg_v;
                u = u + v.shift_left(shift);
                g1 = g1 + g2.shift_left(shift);
            }
        }

        Some(g2)
    }

    /// Get the degree (position of highest set bit)
    fn degree(&self) -> usize {
        if self.hi != 0 {
            64 + (63 - self.hi.leading_zeros() as usize)
        } else if self.lo != 0 {
            63 - self.lo.leading_zeros() as usize
        } else {
            0
        }
    }

    /// Left shift by n bits (multiplication by x^n)
    fn shift_left(&self, n: usize) -> Self {
        if n >= 128 {
            Self::ZERO
        } else if n >= 64 {
            Self {
                lo: 0,
                hi: self.lo << (n - 64),
            }
        } else if n > 0 {
            Self {
                lo: self.lo << n,
                hi: (self.hi << n) | (self.lo >> (64 - n)),
            }
        } else {
            *self
        }
    }
}

/// Portable 64-bit carryless multiplication
fn gf2_mul_64(a: u64, b: u64) -> (u64, u64) {
    let mut lo = 0u64;
    let mut hi = 0u64;

    for i in 0..64 {
        if (b >> i) & 1 == 1 {
            lo ^= a << i;
            if i > 0 {
                hi ^= a >> (64 - i);
            }
        }
    }

    (lo, hi)
}

/// Reduction modulo x^128 + x^7 + x^2 + x + 1
fn reduce_gf2_128(r0: &mut u64, r1: &mut u64, r2: &mut u64, r3: &mut u64) {
    // Reduce 256-bit to 128-bit
    let t3 = *r3;
    let t2 = *r2;

    *r2 = 0;
    *r3 = 0;

    // Multiply high 128 bits by reduction polynomial and XOR into low 128 bits
    *r1 ^= t3 ^ (t3 >> 57) ^ (t3 >> 62) ^ (t3 >> 63);
    *r0 ^= (t3 << 7) ^ (t3 << 2) ^ (t3 << 1) ^ t3;
    
    *r1 ^= t2 ^ (t2 >> 57) ^ (t2 >> 62) ^ (t2 >> 63);
    *r0 ^= (t2 << 7) ^ (t2 << 2) ^ (t2 << 1) ^ t2;
}

impl ConstantTimeEq for Gf2_128 {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.lo.ct_eq(&other.lo) & self.hi.ct_eq(&other.hi)
    }
}

impl ConditionallySelectable for Gf2_128 {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self {
            lo: u64::conditional_select(&a.lo, &b.lo, choice),
            hi: u64::conditional_select(&a.hi, &b.hi, choice),
        }
    }
}

impl Default for Gf2_128 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl PartialEq for Gf2_128 {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

impl Eq for Gf2_128 {}

impl Add for Gf2_128 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            lo: self.lo ^ rhs.lo,
            hi: self.hi ^ rhs.hi,
        }
    }
}

impl AddAssign for Gf2_128 {
    fn add_assign(&mut self, rhs: Self) {
        self.lo ^= rhs.lo;
        self.hi ^= rhs.hi;
    }
}

impl Sub for Gf2_128 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        // In GF(2^n), addition and subtraction are the same (XOR)
        self + rhs
    }
}

impl SubAssign for Gf2_128 {
    fn sub_assign(&mut self, rhs: Self) {
        self.lo ^= rhs.lo;
        self.hi ^= rhs.hi;
    }
}

impl Mul for Gf2_128 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        self.mul_clmul(&rhs)
    }
}

impl MulAssign for Gf2_128 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.mul_clmul(&rhs);
    }
}

impl Debug for Gf2_128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gf2_128({:016x}{:016x})", self.hi, self.lo)
    }
}

impl Display for Gf2_128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}{:016x}", self.hi, self.lo)
    }
}

/// Batch multiplication optimized for GF(2^128)
pub fn batch_multiply(a: &[Gf2_128], b: &[Gf2_128]) -> Vec<Gf2_128> {
    assert_eq!(a.len(), b.len());
    
    #[cfg(all(target_arch = "x86_64", target_feature = "pclmulqdq"))]
    {
        // Process 4 multiplications at once using AVX if available
        let mut result = Vec::with_capacity(a.len());
        
        for (a_elem, b_elem) in a.iter().zip(b.iter()) {
            result.push(*a_elem * *b_elem);
        }
        
        result
    }
    #[cfg(not(all(target_arch = "x86_64", target_feature = "pclmulqdq")))]
    {
        a.iter()
            .zip(b.iter())
            .map(|(a, b)| *a * *b)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_gf2_128_basic_ops() {
        let a = Gf2_128::new(0x123456789ABCDEF0, 0xFEDCBA9876543210);
        let b = Gf2_128::new(0x0F0F0F0F0F0F0F0F, 0xF0F0F0F0F0F0F0F0);
        
        // Addition (XOR)
        let sum = a + b;
        assert_eq!(sum.lo, a.lo ^ b.lo);
        assert_eq!(sum.hi, a.hi ^ b.hi);
        
        // Addition is self-inverse
        assert_eq!(sum + b, a);
        
        // Multiplication with one
        assert_eq!(a * Gf2_128::ONE, a);
        
        // Multiplication with zero
        assert_eq!(a * Gf2_128::ZERO, Gf2_128::ZERO);
    }

    #[test]
    fn test_gf2_128_inversion() {
        let mut rng = ChaCha20Rng::seed_from_u64(42);
        
        for _ in 0..100 {
            let a = Gf2_128::new(rng.gen(), rng.gen());
            if !a.is_zero().into() {
                let a_inv = a.invert().unwrap();
                assert_eq!(a * a_inv, Gf2_128::ONE);
            }
        }
    }

    #[test]
    fn test_gf2_128_square() {
        let a = Gf2_128::new(0x123456789ABCDEF0, 0xFEDCBA9876543210);
        let sq1 = a.square();
        let sq2 = a * a;
        assert_eq!(sq1, sq2);
    }

    #[test]
    fn test_distributivity() {
        let mut rng = ChaCha20Rng::seed_from_u64(42);
        
        for _ in 0..50 {
            let a = Gf2_128::new(rng.gen(), rng.gen());
            let b = Gf2_128::new(rng.gen(), rng.gen());
            let c = Gf2_128::new(rng.gen(), rng.gen());
            
            // Test distributivity: a * (b + c) = a * b + a * c
            let left = a * (b + c);
            let right = a * b + a * c;
            assert_eq!(left, right);
        }
    }
}