use longfellow_algebra::traits::Field;

#[inline]
pub fn affine_interpolation<F: Field>(f0: F, f1: F, r: F) -> F {
    f0 + r * (f1 - f0)
}

#[inline]
pub fn affine_interpolation_z_nz<F: Field>(f1: F, r: F) -> F {
    r * f1
}

#[inline]
pub fn affine_interpolation_nz_z<F: Field>(f0: F, r: F) -> F {
    f0 - r * f0
}

#[inline]
pub fn affine_interpolation_conditional<F: Field>(f0: F, f1: F, r: F) -> F {
    if f0 == F::zero() {
        affine_interpolation_z_nz(f1, r)
    } else if f1 == F::zero() {
        affine_interpolation_nz_z(f0, r)
    } else {
        affine_interpolation(f0, f1, r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestField(u64);

    impl Field for TestField {
        const ZERO: Self = TestField(0);
        const ONE: Self = TestField(1);
        const MODULUS: &'static str = "101";
        const MODULUS_BITS: u32 = 7;

        fn zero() -> Self {
            Self::ZERO
        }

        fn one() -> Self {
            Self::ONE
        }

        fn from_u64(val: u64) -> Self {
            TestField(val % 101)
        }

        fn from_bytes_le(_bytes: &[u8]) -> longfellow_core::Result<Self> {
            unimplemented!()
        }

        fn to_bytes_le(&self) -> Vec<u8> {
            unimplemented!()
        }

        fn invert(&self) -> Option<Self> {
            if self.0 == 0 {
                None
            } else {
                Some(TestField(1))
            }
        }
    }

    impl std::ops::Add for TestField {
        type Output = Self;
        fn add(self, rhs: Self) -> Self {
            TestField((self.0 + rhs.0) % 101)
        }
    }

    impl std::ops::Sub for TestField {
        type Output = Self;
        fn sub(self, rhs: Self) -> Self {
            TestField((self.0 + 101 - rhs.0) % 101)
        }
    }

    impl std::ops::Mul for TestField {
        type Output = Self;
        fn mul(self, rhs: Self) -> Self {
            TestField((self.0 * rhs.0) % 101)
        }
    }

    impl std::ops::Neg for TestField {
        type Output = Self;
        fn neg(self) -> Self {
            TestField((101 - self.0) % 101)
        }
    }

    impl Default for TestField {
        fn default() -> Self {
            Self::ZERO
        }
    }

    impl subtle::ConditionallySelectable for TestField {
        fn conditional_select(a: &Self, b: &Self, choice: subtle::Choice) -> Self {
            if choice.into() {
                *b
            } else {
                *a
            }
        }
    }

    impl subtle::ConstantTimeEq for TestField {
        fn ct_eq(&self, other: &Self) -> subtle::Choice {
            subtle::Choice::from((self.0 == other.0) as u8)
        }
    }

    impl zeroize::Zeroize for TestField {
        fn zeroize(&mut self) {
            self.0 = 0;
        }
    }

    impl std::ops::AddAssign for TestField {
        fn add_assign(&mut self, rhs: Self) {
            *self = *self + rhs;
        }
    }

    impl std::ops::SubAssign for TestField {
        fn sub_assign(&mut self, rhs: Self) {
            *self = *self - rhs;
        }
    }

    impl std::ops::MulAssign for TestField {
        fn mul_assign(&mut self, rhs: Self) {
            *self = *self * rhs;
        }
    }

    impl<'a> std::ops::Add<&'a Self> for TestField {
        type Output = Self;
        fn add(self, rhs: &'a Self) -> Self {
            self + *rhs
        }
    }

    impl<'a> std::ops::Sub<&'a Self> for TestField {
        type Output = Self;
        fn sub(self, rhs: &'a Self) -> Self {
            self - *rhs
        }
    }

    impl<'a> std::ops::Mul<&'a Self> for TestField {
        type Output = Self;
        fn mul(self, rhs: &'a Self) -> Self {
            self * *rhs
        }
    }

    impl<'a> std::ops::AddAssign<&'a Self> for TestField {
        fn add_assign(&mut self, rhs: &'a Self) {
            *self += *rhs;
        }
    }

    impl<'a> std::ops::SubAssign<&'a Self> for TestField {
        fn sub_assign(&mut self, rhs: &'a Self) {
            *self -= *rhs;
        }
    }

    impl<'a> std::ops::MulAssign<&'a Self> for TestField {
        fn mul_assign(&mut self, rhs: &'a Self) {
            *self *= *rhs;
        }
    }

    #[test]
    fn test_affine_interpolation() {
        let f0 = TestField(10);
        let f1 = TestField(20);
        let r = TestField(50);
        
        let result = affine_interpolation(f0, f1, r);
        assert_eq!(result, TestField(15));
    }

    #[test]
    fn test_affine_interpolation_zero_cases() {
        let f1 = TestField(20);
        let r = TestField(50);
        
        let result = affine_interpolation_z_nz(f1, r);
        assert_eq!(result, TestField(10));
        
        let f0 = TestField(20);
        let result = affine_interpolation_nz_z(f0, r);
        assert_eq!(result, TestField(10));
    }
}