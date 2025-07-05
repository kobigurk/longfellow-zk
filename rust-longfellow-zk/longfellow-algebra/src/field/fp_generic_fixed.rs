// Fixed Montgomery reduction implementation

use crate::nat::{Nat, Limb, mul_wide, add_with_carry};

pub fn montgomery_reduce_fixed<const N: usize>(
    extended: &mut Vec<Limb>,
    inv: Limb,
    modulus: &Nat<N>
) {
    // Montgomery reduction for a 2N-limb value
    
    // Ensure we have at least 2N limbs
    while extended.len() < 2 * N {
        extended.push(0);
    }
    
    for i in 0..N {
        // k = t[i] * INV mod 2^64
        let k = extended[i].wrapping_mul(inv);
        
        // t = t + k * modulus * 2^(64*i)
        let mut carry = 0u64;
        
        for j in 0..N {
            let (lo, hi) = mul_wide(k, modulus.limbs[j]);
            let (sum, c) = add_with_carry(extended[i + j], lo, carry);
            extended[i + j] = sum;
            carry = hi + c;
        }
        
        // Propagate carry to higher limbs
        let mut idx = i + N;
        while idx < extended.len() && carry > 0 {
            let (sum, c) = add_with_carry(extended[idx], 0, carry);
            extended[idx] = sum;
            carry = c;
            idx += 1;
        }
    }
    
    // Shift right by N limbs (this is the division by R)
    for i in 0..N {
        extended[i] = if i + N < extended.len() {
            extended[i + N]
        } else {
            0
        };
    }
    
    // Truncate to N limbs
    extended.truncate(N);
    
    // Final reduction if result >= modulus
    let mut result = Nat::<N>::default();
    result.limbs.copy_from_slice(&extended[..N]);
    
    // Use the built-in comparison
    if result >= modulus {
        result.sub_with_borrow(modulus);
        extended.copy_from_slice(&result.limbs);
    }
}

// Test the fixed implementation
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_montgomery_reduce_r() {
        // Test that Montgomery reduction of R gives 1
        // For Fp128: R = 2^108 - 1
        
        let r = Nat::<2>::new([0xFFFFFFFFFFFFFFFF, 0x00000FFFFFFFFFFF]);
        let modulus = Nat::<2>::new([0x0000000000000001, 0xFFFFF00000000000]);
        let inv = 0xFFFFFFFFFFFFFFFF;
        
        // Create input: R in lower limbs, 0 in upper limbs
        let mut t = vec![0u64; 4];
        t[0] = r.limbs[0];
        t[1] = r.limbs[1];
        
        montgomery_reduce_fixed(&mut t, inv, &modulus);
        
        // Result should be 1
        assert_eq!(t.len(), 2);
        assert_eq!(t[0], 1);
        assert_eq!(t[1], 0);
    }
}