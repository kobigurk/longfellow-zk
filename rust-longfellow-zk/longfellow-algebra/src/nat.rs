use std::cmp::Ordering;
use std::fmt::{self, Debug, Display};
use std::ops::{Add, AddAssign, BitAnd, Shl, Shr, Sub, SubAssign};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeLess};
use zeroize::Zeroize;

#[cfg(target_pointer_width = "64")]
pub type Limb = u64;
#[cfg(target_pointer_width = "32")]
pub type Limb = u32;

#[derive(Clone, Copy, Zeroize)]
#[repr(transparent)]
pub struct Nat<const N: usize> {
    pub limbs: [Limb; N],
}

impl<const N: usize> Nat<N> {
    pub const LIMB_BITS: usize = Limb::BITS as usize;
    pub const BITS: usize = N * Self::LIMB_BITS;
    pub const BYTES: usize = N * std::mem::size_of::<Limb>();

    pub const ZERO: Self = Self { limbs: [0; N] };
    pub const ONE: Self = {
        let mut limbs = [0; N];
        limbs[0] = 1;
        Self { limbs }
    };

    pub const fn new(limbs: [Limb; N]) -> Self {
        Self { limbs }
    }

    pub fn from_u64(val: u64) -> Self {
        let mut limbs = [0; N];
        if N > 0 {
            #[cfg(target_pointer_width = "64")]
            {
                limbs[0] = val;
            }
            #[cfg(target_pointer_width = "32")]
            {
                limbs[0] = val as u32;
                if N > 1 {
                    limbs[1] = (val >> 32) as u32;
                }
            }
        }
        Self { limbs }
    }

    pub fn from_bytes_le(bytes: &[u8]) -> Option<Self> {
        if bytes.len() > Self::BYTES {
            return None;
        }

        let mut limbs = [0; N];
        let limb_bytes = std::mem::size_of::<Limb>();

        for (i, chunk) in bytes.chunks(limb_bytes).enumerate() {
            if i >= N {
                break;
            }
            let mut limb_bytes = [0u8; 8];
            limb_bytes[..chunk.len()].copy_from_slice(chunk);
            limbs[i] = Limb::from_le_bytes(limb_bytes[..limb_bytes].try_into().ok()?);
        }

        Some(Self { limbs })
    }

    pub fn to_bytes_le(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::BYTES);
        for limb in &self.limbs {
            bytes.extend_from_slice(&limb.to_le_bytes());
        }
        bytes
    }

    pub fn is_zero(&self) -> Choice {
        self.ct_eq(&Self::ZERO)
    }

    pub fn is_odd(&self) -> Choice {
        Choice::from((self.limbs[0] & 1) as u8)
    }

    pub fn is_even(&self) -> Choice {
        !self.is_odd()
    }

    #[inline]
    pub fn add_with_carry(&mut self, rhs: &Self) -> Limb {
        let mut carry = 0;
        for i in 0..N {
            let (sum1, c1) = self.limbs[i].overflowing_add(rhs.limbs[i]);
            let (sum2, c2) = sum1.overflowing_add(carry);
            self.limbs[i] = sum2;
            carry = (c1 as Limb) | (c2 as Limb);
        }
        carry
    }

    #[inline]
    pub fn sub_with_borrow(&mut self, rhs: &Self) -> Limb {
        let mut borrow = 0;
        for i in 0..N {
            let (diff1, b1) = self.limbs[i].overflowing_sub(rhs.limbs[i]);
            let (diff2, b2) = diff1.overflowing_sub(borrow);
            self.limbs[i] = diff2;
            borrow = (b1 as Limb) | (b2 as Limb);
        }
        borrow
    }

    pub fn shr1(&mut self) {
        let mut carry = 0;
        for i in (0..N).rev() {
            let new_carry = self.limbs[i] & 1;
            self.limbs[i] = (self.limbs[i] >> 1) | (carry << (Self::LIMB_BITS - 1));
            carry = new_carry;
        }
    }

    pub fn shl1(&mut self) -> Limb {
        let mut carry = 0;
        for i in 0..N {
            let new_carry = self.limbs[i] >> (Self::LIMB_BITS - 1);
            self.limbs[i] = (self.limbs[i] << 1) | carry;
            carry = new_carry;
        }
        carry
    }

    pub fn conditional_add(&mut self, rhs: &Self, choice: Choice) -> Choice {
        let mut carry = 0;
        for i in 0..N {
            let rhs_limb = Limb::conditional_select(&0, &rhs.limbs[i], choice);
            let (sum1, c1) = self.limbs[i].overflowing_add(rhs_limb);
            let (sum2, c2) = sum1.overflowing_add(carry);
            self.limbs[i] = sum2;
            carry = (c1 as Limb) | (c2 as Limb);
        }
        Choice::from(carry as u8)
    }

    pub fn conditional_sub(&mut self, rhs: &Self, choice: Choice) -> Choice {
        let mut borrow = 0;
        for i in 0..N {
            let rhs_limb = Limb::conditional_select(&0, &rhs.limbs[i], choice);
            let (diff1, b1) = self.limbs[i].overflowing_sub(rhs_limb);
            let (diff2, b2) = diff1.overflowing_sub(borrow);
            self.limbs[i] = diff2;
            borrow = (b1 as Limb) | (b2 as Limb);
        }
        Choice::from(borrow as u8)
    }
}

impl<const N: usize> Default for Nat<N> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<const N: usize> ConstantTimeEq for Nat<N> {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.limbs
            .iter()
            .zip(other.limbs.iter())
            .fold(Choice::from(1), |acc, (a, b)| acc & a.ct_eq(b))
    }
}

impl<const N: usize> ConditionallySelectable for Nat<N> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut limbs = [0; N];
        for i in 0..N {
            limbs[i] = Limb::conditional_select(&a.limbs[i], &b.limbs[i], choice);
        }
        Self { limbs }
    }
}

impl<const N: usize> PartialEq for Nat<N> {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

impl<const N: usize> Eq for Nat<N> {}

impl<const N: usize> PartialOrd for Nat<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> Ord for Nat<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in (0..N).rev() {
            match self.limbs[i].cmp(&other.limbs[i]) {
                Ordering::Equal => continue,
                ord => return ord,
            }
        }
        Ordering::Equal
    }
}

impl<const N: usize> Debug for Nat<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Nat(0x")?;
        for limb in self.limbs.iter().rev() {
            write!(f, "{:016x}", limb)?;
        }
        write!(f, ")")
    }
}

#[inline]
pub fn add_with_carry(a: Limb, b: Limb, carry: Limb) -> (Limb, Limb) {
    let (sum1, c1) = a.overflowing_add(b);
    let (sum2, c2) = sum1.overflowing_add(carry);
    (sum2, (c1 as Limb) | (c2 as Limb))
}

#[inline]
pub fn sub_with_borrow(a: Limb, b: Limb, borrow: Limb) -> (Limb, Limb) {
    let (diff1, b1) = a.overflowing_sub(b);
    let (diff2, b2) = diff1.overflowing_sub(borrow);
    (diff2, (b1 as Limb) | (b2 as Limb))
}

#[inline]
pub fn mul_wide(a: Limb, b: Limb) -> (Limb, Limb) {
    let wide = (a as u128) * (b as u128);
    (wide as Limb, (wide >> Limb::BITS) as Limb)
}

#[inline]
pub fn mac_with_carry(a: Limb, b: Limb, c: Limb, carry: Limb) -> (Limb, Limb) {
    let wide = (a as u128) + (b as u128) * (c as u128) + (carry as u128);
    (wide as Limb, (wide >> Limb::BITS) as Limb)
}