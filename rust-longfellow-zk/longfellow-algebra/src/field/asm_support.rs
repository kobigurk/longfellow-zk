/// Assembly-optimized operations for field arithmetic
/// 
/// These functions provide low-level optimized implementations
/// for critical field operations using inline assembly.

use crate::nat::Limb;

/// Multiply two 64-bit values and return (low, high) parts
#[inline(always)]
pub fn mul_wide_asm(a: u64, b: u64) -> (u64, u64) {
    #[cfg(target_arch = "x86_64")]
    {
        let mut lo: u64;
        let mut hi: u64;
        unsafe {
            std::arch::asm!(
                "mul {}",
                in(reg) b,
                inlateout("rax") a => lo,
                lateout("rdx") hi,
                options(pure, nomem, nostack)
            );
        }
        (lo, hi)
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let wide = (a as u128) * (b as u128);
        (wide as u64, (wide >> 64) as u64)
    }
}

/// Add with carry using assembly
#[inline(always)]
pub fn add_with_carry_asm(a: u64, b: u64, carry_in: u8) -> (u64, u8) {
    #[cfg(target_arch = "x86_64")]
    {
        let mut sum: u64;
        let mut carry_out: u8;
        unsafe {
            std::arch::asm!(
                "add {}, {}",
                "adc {}, 0",
                "setc {}",
                inlateout(reg) a => sum,
                in(reg) b,
                inlateout(reg) carry_in as u64 => _,
                lateout(reg_byte) carry_out,
                options(pure, nomem, nostack)
            );
        }
        (sum, carry_out)
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let wide = (a as u128) + (b as u128) + (carry_in as u128);
        (wide as u64, (wide >> 64) as u8)
    }
}

/// Subtract with borrow using assembly
#[inline(always)]
pub fn sub_with_borrow_asm(a: u64, b: u64, borrow_in: u8) -> (u64, u8) {
    #[cfg(target_arch = "x86_64")]
    {
        let mut diff: u64;
        let mut borrow_out: u8;
        unsafe {
            std::arch::asm!(
                "sub {}, {}",
                "sbb {}, 0",
                "setc {}",
                inlateout(reg) a => diff,
                in(reg) b,
                inlateout(reg) borrow_in as u64 => _,
                lateout(reg_byte) borrow_out,
                options(pure, nomem, nostack)
            );
        }
        (diff, borrow_out)
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let wide = (a as u128).wrapping_sub((b as u128) + (borrow_in as u128));
        (wide as u64, if wide > a as u128 { 1 } else { 0 })
    }
}

/// Multiply-accumulate with carry: a + b * c + carry
#[inline(always)]
pub fn mac_with_carry_asm(a: u64, b: u64, c: u64, carry: u64) -> (u64, u64) {
    #[cfg(target_arch = "x86_64")]
    {
        let mut lo: u64;
        let mut hi: u64;
        unsafe {
            std::arch::asm!(
                "mul {}",
                "add {}, {}",
                "adc {}, {}",
                in(reg) c,
                inlateout("rax") b => lo,
                lateout("rdx") hi,
                in(reg) a,
                in(reg) carry,
                options(pure, nomem, nostack)
            );
        }
        (lo, hi)
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let wide = (a as u128) + (b as u128) * (c as u128) + (carry as u128);
        (wide as u64, (wide >> 64) as u64)
    }
}

/// Montgomery reduction step with assembly optimization
#[inline(always)]
pub fn montgomery_reduce_step_asm(a: &mut [u64], k: u64, m: &[u64], offset: usize) {
    #[cfg(target_arch = "x86_64")]
    {
        let n = m.len();
        let mut carry = 0u64;
        
        unsafe {
            for j in 0..n {
                let (lo, hi) = mac_with_carry_asm(a[offset + j], k, m[j], carry);
                a[offset + j] = lo;
                carry = hi;
            }
            
            // Propagate final carry
            let mut carry_flag: u8;
            std::arch::asm!(
                "add {}, {}",
                "setc {}",
                inlateout(reg) a[offset + n] => a[offset + n],
                in(reg) carry,
                lateout(reg_byte) carry_flag,
                options(pure, nomem, nostack)
            );
            
            if carry_flag != 0 && offset + n + 1 < a.len() {
                a[offset + n + 1] = a[offset + n + 1].wrapping_add(1);
            }
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let n = m.len();
        let mut carry = 0u64;
        
        for j in 0..n {
            let (lo, hi) = mac_with_carry_asm(a[offset + j], k, m[j], carry);
            a[offset + j] = lo;
            carry = hi;
        }
        
        a[offset + n] = a[offset + n].wrapping_add(carry);
    }
}

/// Optimized field addition for 128-bit fields
#[inline(always)]
pub fn add_fp128_asm(a: &mut [u64; 2], b: &[u64; 2], modulus: &[u64; 2]) {
    #[cfg(target_arch = "x86_64")]
    {
        unsafe {
            let mut carry: u8;
            
            // Add a + b
            std::arch::asm!(
                "add {0}, {2}",
                "adc {1}, {3}",
                "setc {4}",
                inlateout(reg) a[0],
                inlateout(reg) a[1],
                in(reg) b[0],
                in(reg) b[1],
                lateout(reg_byte) carry,
                options(pure, nomem, nostack)
            );
            
            // Conditional subtraction of modulus
            let mut tmp0 = a[0];
            let mut tmp1 = a[1];
            let mut borrow: u8;
            
            std::arch::asm!(
                "sub {0}, {2}",
                "sbb {1}, {3}",
                "setc {4}",
                inlateout(reg) tmp0,
                inlateout(reg) tmp1,
                in(reg) modulus[0],
                in(reg) modulus[1],
                lateout(reg_byte) borrow,
                options(pure, nomem, nostack)
            );
            
            // Select result based on carry/borrow
            if carry == 0 && borrow == 0 {
                a[0] = tmp0;
                a[1] = tmp1;
            }
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let (sum0, carry0) = a[0].overflowing_add(b[0]);
        let (sum1, carry1) = a[1].overflowing_add(b[1]);
        let (sum1, carry2) = sum1.overflowing_add(carry0 as u64);
        
        let carry = (carry1 | carry2) as u8;
        
        a[0] = sum0;
        a[1] = sum1;
        
        if carry != 0 {
            let (diff0, borrow0) = a[0].overflowing_sub(modulus[0]);
            let (diff1, _) = a[1].overflowing_sub(modulus[1]);
            let (diff1, _) = diff1.overflowing_sub(borrow0 as u64);
            
            a[0] = diff0;
            a[1] = diff1;
        }
    }
}

/// Optimized field multiplication for 128-bit fields using MULX instruction
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "bmi2")]
#[inline(always)]
pub unsafe fn mul_fp128_mulx(a: &[u64; 2], b: &[u64; 2], result: &mut [u64; 4]) {
    use std::arch::x86_64::{_mulx_u64, _addcarryx_u64};
    
    let mut carry: u8;
    let mut hi: u64;
    
    // a[0] * b[0]
    result[0] = _mulx_u64(a[0], b[0], &mut result[1]);
    
    // a[0] * b[1]
    let t0 = _mulx_u64(a[0], b[1], &mut hi);
    carry = _addcarryx_u64(0, result[1], t0, &mut result[1]);
    carry = _addcarryx_u64(carry, 0, hi, &mut result[2]);
    
    // a[1] * b[0]
    let t0 = _mulx_u64(a[1], b[0], &mut hi);
    carry = _addcarryx_u64(0, result[1], t0, &mut result[1]);
    carry = _addcarryx_u64(carry, result[2], hi, &mut result[2]);
    result[3] = carry as u64;
    
    // a[1] * b[1]
    let t0 = _mulx_u64(a[1], b[1], &mut hi);
    carry = _addcarryx_u64(0, result[2], t0, &mut result[2]);
    carry = _addcarryx_u64(carry, result[3], hi, &mut result[3]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul_wide_asm() {
        let a = 0x123456789ABCDEF0u64;
        let b = 0xFEDCBA9876543210u64;
        
        let (lo1, hi1) = mul_wide_asm(a, b);
        let (lo2, hi2) = crate::nat::mul_wide(a, b);
        
        assert_eq!((lo1, hi1), (lo2, hi2));
    }

    #[test]
    fn test_add_with_carry_asm() {
        let test_cases = [
            (u64::MAX, 1, 0),
            (u64::MAX, u64::MAX, 1),
            (1234567890, 9876543210, 0),
        ];
        
        for (a, b, carry_in) in test_cases {
            let (sum1, carry1) = add_with_carry_asm(a, b, carry_in);
            let (sum2, carry2) = crate::nat::add_with_carry(a, b, carry_in as u64);
            
            assert_eq!(sum1, sum2);
            assert_eq!(carry1 as u64, carry2);
        }
    }
}