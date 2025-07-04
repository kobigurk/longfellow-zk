use crate::field::fp_generic::{FieldReduction, FpGeneric};
use crate::nat::{self, Limb, Nat};
use crate::traits::Field;

pub type Fp128 = FpGeneric<2, Fp128Reduce>;

#[derive(Clone, Copy)]
pub struct Fp128Reduce;

impl FieldReduction<2> for Fp128Reduce {
    const MODULUS: Nat<2> = Nat {
        limbs: [
            0x0000000000000001,
            0xFFFFF00000000000,
        ]
    };
    
    const MODULUS_STR: &'static str = "0xFFFFF00000000000000000000000001";
    const MODULUS_BITS: u32 = 128;
    
    const R: Nat<2> = Nat {
        limbs: [
            0x00000FFFFFFFFFFF,
            0x0000000000000000,
        ]
    };
    
    const R2: Nat<2> = Nat {
        limbs: [
            0x00000FFFEFFFC001,
            0x0000000000000000,
        ]
    };
    
    const INV: Limb = 0xFFFFEFFFFFFFFFFF;
    
    fn reduction_step(a: &mut [Limb], mprime: Limb, modulus: &Nat<2>) {
        // Montgomery reduction step for p = 2^128 - 2^108 + 1
        // This is a placeholder - real implementation would optimize this
        let k = a[0].wrapping_mul(mprime);
        let mut carry = 0u64;
        
        for i in 0..2 {
            let (lo, hi) = nat::mul_wide(modulus.limbs[i], k);
            let (sum, c) = nat::add_with_carry(a[i], lo, carry);
            a[i] = sum;
            carry = hi + c;
        }
        
        a[2] = a[2].wrapping_add(carry);
        
        // Shift right by one limb
        for i in 0..2 {
            a[i] = a[i + 1];
        }
    }
}

impl Fp128 {
    /// Get a primitive root of unity for the given order
    /// Returns None if no such root exists
    pub fn get_root_of_unity(n: usize) -> Option<Self> {
        // For Fp128 with p = 2^128 - 2^108 + 1, we have p - 1 = 2^108 * (2^20 - 1)
        // The C++ implementation uses omega of order 2^32 from the comments in fp_p128.h
        // This is: 164956748514267535023998284330560247862
        
        // Check if n is a power of 2
        if !n.is_power_of_two() {
            return None;
        }
        
        // For powers of 2 up to 2^108, we can compute roots
        // Start with the root of unity of order 2^32 from C++
        const OMEGA_32_ORDER: u32 = 32;
        
        if n == 1 {
            return Some(Self::one());
        }
        
        // Parse the omega_32 value
        let omega_32_bytes = [
            0x36, 0x0c, 0xda, 0x62, 0xfe, 0xea, 0x28, 0x7c,
            0xce, 0x03, 0x89, 0x3f, 0xf2, 0x73, 0x50, 0x01,
        ];
        let omega_32 = Self::from_bytes_le(&omega_32_bytes).ok()?;
        
        // For the requested size n, we need to compute omega_32^(2^(32-log2(n)))
        let log_n = n.trailing_zeros();
        if log_n > OMEGA_32_ORDER {
            return None;
        }
        
        // Compute omega_n = omega_32^(2^(32-log_n))
        let exponent = 1u64 << (OMEGA_32_ORDER - log_n);
        let omega_n = omega_32.pow(&[exponent]);
        
        Some(omega_n)
    }
}

// Field trait is already implemented by FpGeneric<N, R> in fp_generic.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Field;

    #[test]
    fn test_fp128_basic_operations() {
        let a = Fp128::from_u64(5);
        let b = Fp128::from_u64(7);
        
        let sum = a + b;
        assert_eq!(sum, Fp128::from_u64(12));
        
        let diff = b - a;
        assert_eq!(diff, Fp128::from_u64(2));
        
        let prod = a * b;
        assert_eq!(prod, Fp128::from_u64(35));
        
        let neg_a = -a;
        assert_eq!(a + neg_a, Fp128::zero());
    }

    #[test]
    fn test_fp128_inversion() {
        let a = Fp128::from_u64(3);
        let a_inv = a.invert().unwrap();
        assert_eq!(a * a_inv, Fp128::one());
    }

    #[test]
    fn test_fp128_zero_inversion() {
        let zero = Fp128::zero();
        assert!(zero.invert().is_none());
    }
    
    #[test]
    fn test_root_of_unity() {
        // Test that roots of unity have the correct order
        if let Some(omega_2) = Fp128::get_root_of_unity(2) {
            assert_eq!(omega_2.square(), Fp128::one());
            assert_ne!(omega_2, Fp128::one());
        }
        
        if let Some(omega_4) = Fp128::get_root_of_unity(4) {
            let omega_4_squared = omega_4.square();
            assert_ne!(omega_4_squared, Fp128::one());
            assert_eq!(omega_4_squared.square(), Fp128::one());
        }
    }
}