use crate::traits::Field;
use rayon::prelude::*;

pub fn dot_product<F: Field>(a: &[F], b: &[F]) -> F {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");
    
    a.iter()
        .zip(b.iter())
        .map(|(ai, bi)| *ai * bi)
        .fold(F::zero(), |acc, x| acc + x)
}

pub fn scale<F: Field>(v: &mut [F], scalar: F) {
    v.par_iter_mut().for_each(|x| *x *= scalar);
}

pub fn add_scaled<F: Field>(dst: &mut [F], src: &[F], scalar: F) {
    assert_eq!(dst.len(), src.len(), "Vectors must have the same length");
    
    dst.par_iter_mut()
        .zip(src.par_iter())
        .for_each(|(d, s)| *d += *s * scalar);
}

pub fn hadamard_product<F: Field>(a: &mut [F], b: &[F]) {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");
    
    a.par_iter_mut()
        .zip(b.par_iter())
        .for_each(|(ai, bi)| *ai *= bi);
}

pub fn matrix_vector_multiply<F: Field>(matrix: &[Vec<F>], vector: &[F]) -> Vec<F> {
    matrix
        .par_iter()
        .map(|row| dot_product(row, vector))
        .collect()
}

pub fn transpose<F: Field>(matrix: &[Vec<F>]) -> Vec<Vec<F>> {
    if matrix.is_empty() || matrix[0].is_empty() {
        return vec![];
    }
    
    let rows = matrix.len();
    let cols = matrix[0].len();
    
    (0..cols)
        .into_par_iter()
        .map(|j| (0..rows).map(|i| matrix[i][j]).collect())
        .collect()
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

        fn from_bytes_le(_bytes: &[u8]) -> crate::longfellow_core::Result<Self> {
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
    fn test_dot_product() {
        let a = vec![TestField(1), TestField(2), TestField(3)];
        let b = vec![TestField(4), TestField(5), TestField(6)];
        let result = dot_product(&a, &b);
        assert_eq!(result, TestField(32));
    }

    #[test]
    fn test_scale() {
        let mut v = vec![TestField(1), TestField(2), TestField(3)];
        scale(&mut v, TestField(2));
        assert_eq!(v, vec![TestField(2), TestField(4), TestField(6)]);
    }

    #[test]
    fn test_transpose() {
        let matrix = vec![
            vec![TestField(1), TestField(2), TestField(3)],
            vec![TestField(4), TestField(5), TestField(6)],
        ];
        let transposed = transpose(&matrix);
        assert_eq!(
            transposed,
            vec![
                vec![TestField(1), TestField(4)],
                vec![TestField(2), TestField(5)],
                vec![TestField(3), TestField(6)],
            ]
        );
    }
}