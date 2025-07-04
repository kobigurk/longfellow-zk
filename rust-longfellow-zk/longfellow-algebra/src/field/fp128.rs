use crate::field::fp_generic::{FieldReduction, FpGeneric};
use crate::nat::{Limb, Nat};
use crate::traits::PrimeField;
use subtle::Choice;

pub struct Fp128Reduce;

impl Fp128Reduce {
    pub const MODULUS: Nat<2> = Nat::new([0x0000000000000001u64, 0xFFFFF00000000000u64]);
    
    pub const MODULUS_STR: &'static str = "340282042402384805036647824275747635201";
    pub const MODULUS_BITS: u32 = 128;
    
    pub const R: Nat<2> = Nat::new([0x00000ffffffffffe, 0x00000fffffffffff]);
    
    pub const R2: Nat<2> = Nat::new([0xe000000000001, 0xfffffff]);
    
    pub const INV: u64 = 0xfffffffffffffffd;
    
    pub const TWO_ADIC_ROOT_OF_UNITY: Nat<2> = 
        Nat::new([0xc0d80fe4033fea5a, 0xf33cfff73f3cffc]);
}

impl FieldReduction<2> for Fp128Reduce {
    fn reduction_step(a: &mut [Limb], _mprime: Limb, _modulus: &Nat<2>) {
        let r = (!a[0]).wrapping_add(1);
        
        let sub_lo = r << 44;
        let sub_hi = r >> 20;
        
        let (sum1, carry1) = a[0].overflowing_add(r);
        a[0] = sum1;
        
        let (sum2, carry2) = a[2].overflowing_add(r);
        let (sum3, carry3) = sum2.overflowing_add(carry1 as u64);
        a[2] = sum3;
        
        if a.len() > 3 {
            a[3] = a[3].wrapping_add((carry2 | carry3) as u64);
        }
        
        let (diff1, borrow1) = a[1].overflowing_sub(sub_lo);
        a[1] = diff1;
        
        let (diff2, _) = a[2].overflowing_sub(sub_hi);
        let (diff3, _) = diff2.overflowing_sub(borrow1 as u64);
        a[2] = diff3;
    }
}

pub type Fp128 = FpGeneric<2, Fp128Reduce>;

impl PrimeField for Fp128 {
    type Repr = [u8; 16];

    fn from_repr(repr: Self::Repr) -> Option<Self> {
        Self::from_bytes_le(&repr).ok()
    }

    fn to_repr(&self) -> Self::Repr {
        let bytes = self.to_bytes_le();
        let mut repr = [0u8; 16];
        repr.copy_from_slice(&bytes[..16]);
        repr
    }

    fn is_odd(&self) -> Choice {
        self.from_montgomery().is_odd()
    }
}

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
}