use crate::affine::affine_interpolation_conditional;
use crate::CornerIndex;
use longfellow_algebra::traits::Field;
use longfellow_core::{LongfellowError, Result};
use rayon::prelude::*;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Dense<F: Field> {
    n0: CornerIndex,
    n1: CornerIndex,
    v: Vec<F>,
}

impl<F: Field> Dense<F> {
    pub fn new(n0: CornerIndex, n1: CornerIndex) -> Self {
        assert!(n0 > 0 && n1 > 0, "Dimensions must be positive");
        Self {
            n0,
            n1,
            v: vec![F::zero(); n0 * n1],
        }
    }

    pub fn from_vec(n0: CornerIndex, n1: CornerIndex, v: Vec<F>) -> Result<Self> {
        if v.len() != n0 * n1 {
            return Err(LongfellowError::InvalidParameter(format!(
                "Vector length {} does not match dimensions {}x{}",
                v.len(),
                n0,
                n1
            )));
        }
        Ok(Self { n0, n1, v })
    }

    pub fn n0(&self) -> CornerIndex {
        self.n0
    }

    pub fn n1(&self) -> CornerIndex {
        self.n1
    }

    pub fn len(&self) -> usize {
        self.v.len()
    }

    pub fn is_empty(&self) -> bool {
        self.v.is_empty()
    }

    pub fn as_slice(&self) -> &[F] {
        &self.v
    }

    pub fn as_mut_slice(&mut self) -> &mut [F] {
        &mut self.v
    }

    pub fn get(&self, i0: CornerIndex, i1: CornerIndex) -> Option<&F> {
        if i0 < self.n0 && i1 < self.n1 {
            Some(&self.v[i0 * self.n1 + i1])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, i0: CornerIndex, i1: CornerIndex) -> Option<&mut F> {
        if i0 < self.n0 && i1 < self.n1 {
            Some(&mut self.v[i0 * self.n1 + i1])
        } else {
            None
        }
    }

    pub fn set(&mut self, i0: CornerIndex, i1: CornerIndex, value: F) -> Result<()> {
        if i0 >= self.n0 || i1 >= self.n1 {
            return Err(LongfellowError::InvalidParameter(format!(
                "Index ({}, {}) out of bounds for dimensions {}x{}",
                i0, i1, self.n0, self.n1
            )));
        }
        self.v[i0 * self.n1 + i1] = value;
        Ok(())
    }

    pub fn bind(&mut self, r: F) {
        assert!(self.n0 > 1, "Cannot bind dimension of size 1");
        
        let new_n0 = self.n0 / 2;
        let n1 = self.n1;

        if new_n0 * n1 >= 1024 {
            // Process rows in parallel, but we need to be careful about aliasing
            // Since we're reading from the second half and writing to the first half,
            // we can split the array
            let (first_half, second_half) = self.v.split_at_mut(new_n0 * n1);
            
            first_half.par_chunks_mut(n1)
                .zip(second_half.par_chunks(n1))
                .for_each(|(row, other_row)| {
                    for j in 0..n1 {
                        let f0 = row[j];
                        let f1 = other_row[j];
                        row[j] = affine_interpolation_conditional(f0, f1, r);
                    }
                });
        } else {
            for i in 0..new_n0 {
                for j in 0..n1 {
                    let f0 = self.v[i * n1 + j];
                    let f1 = self.v[(i + new_n0) * n1 + j];
                    self.v[i * n1 + j] = affine_interpolation_conditional(f0, f1, r);
                }
            }
        }

        self.n0 = new_n0;
        self.v.truncate(new_n0 * n1);
    }

    pub fn bind_all(&mut self, log_v: usize, r: &[F]) {
        assert!(
            r.len() == log_v,
            "Number of binding values must equal log_v"
        );
        assert!(
            self.n0 == 1 << log_v,
            "n0 must equal 2^log_v for bind_all"
        );

        for &ri in r {
            self.bind(ri);
        }

        assert_eq!(self.n0, 1);
    }

    pub fn scale(&mut self, x: F, x_last: F) {
        let n = self.v.len();
        if n == 0 {
            return;
        }

        if n >= 1024 {
            self.v.par_iter_mut().enumerate().for_each(|(i, v)| {
                if i < n - 1 {
                    *v *= x;
                } else {
                    *v *= x_last;
                }
            });
        } else {
            for i in 0..n - 1 {
                self.v[i] *= x;
            }
            self.v[n - 1] *= x_last;
        }
    }

    pub fn reshape(&mut self, new_n0: CornerIndex) -> Result<()> {
        let total = self.n0 * self.n1;
        if total % new_n0 != 0 {
            return Err(LongfellowError::InvalidParameter(format!(
                "Cannot reshape {}x{} to have n0={}",
                self.n0, self.n1, new_n0
            )));
        }
        self.n0 = new_n0;
        self.n1 = total / new_n0;
        Ok(())
    }

    pub fn at_corners(&self, p0: CornerIndex, p1: CornerIndex) -> Result<&F> {
        self.get(p0, p1).ok_or_else(|| {
            LongfellowError::InvalidParameter(format!(
                "Corner ({}, {}) out of bounds",
                p0, p1
            ))
        })
    }

    pub fn scalar(&self) -> Result<F> {
        if self.n0 != 1 || self.n1 != 1 {
            return Err(LongfellowError::InvalidParameter(format!(
                "Cannot get scalar from {}x{} array",
                self.n0, self.n1
            )));
        }
        Ok(self.v[0])
    }

    pub fn add_scaled(&mut self, other: &Self, scalar: F) -> Result<()> {
        if self.n0 != other.n0 || self.n1 != other.n1 {
            return Err(LongfellowError::InvalidParameter(
                "Dimension mismatch for add_scaled".to_string(),
            ));
        }

        if self.v.len() >= 1024 {
            self.v
                .par_iter_mut()
                .zip(other.v.par_iter())
                .for_each(|(a, b)| *a += *b * scalar);
        } else {
            for (a, b) in self.v.iter_mut().zip(other.v.iter()) {
                *a += *b * scalar;
            }
        }

        Ok(())
    }
}

pub struct DenseFiller<F: Field> {
    dense: Dense<F>,
    index: usize,
}

impl<F: Field> DenseFiller<F> {
    pub fn new(n0: CornerIndex, n1: CornerIndex) -> Self {
        Self {
            dense: Dense::new(n0, n1),
            index: 0,
        }
    }

    pub fn push(&mut self, value: F) -> Result<()> {
        if self.index >= self.dense.v.len() {
            return Err(LongfellowError::InvalidParameter(
                "DenseFiller is full".to_string(),
            ));
        }
        self.dense.v[self.index] = value;
        self.index += 1;
        Ok(())
    }

    pub fn finish(self) -> Result<Dense<F>> {
        if self.index != self.dense.v.len() {
            return Err(LongfellowError::InvalidParameter(format!(
                "DenseFiller incomplete: {} of {} elements filled",
                self.index,
                self.dense.v.len()
            )));
        }
        Ok(self.dense)
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

        fn from_bytes_le(_bytes: &[u8]) -> Result<Self> {
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
    fn test_dense_creation() {
        let dense = Dense::<TestField>::new(4, 3);
        assert_eq!(dense.n0(), 4);
        assert_eq!(dense.n1(), 3);
        assert_eq!(dense.len(), 12);
    }

    #[test]
    fn test_dense_get_set() {
        let mut dense = Dense::new(2, 3);
        dense.set(1, 2, TestField(42)).unwrap();
        assert_eq!(dense.get(1, 2), Some(&TestField(42)));
    }

    #[test]
    fn test_dense_bind() {
        let mut dense = Dense::from_vec(
            4,
            2,
            vec![
                TestField(1),
                TestField(2),
                TestField(3),
                TestField(4),
                TestField(5),
                TestField(6),
                TestField(7),
                TestField(8),
            ],
        )
        .unwrap();

        dense.bind(TestField(50));
        assert_eq!(dense.n0(), 2);
        assert_eq!(dense.len(), 4);
    }

    #[test]
    fn test_dense_filler() {
        let mut filler = DenseFiller::new(2, 2);
        filler.push(TestField(1)).unwrap();
        filler.push(TestField(2)).unwrap();
        filler.push(TestField(3)).unwrap();
        filler.push(TestField(4)).unwrap();
        
        let dense = filler.finish().unwrap();
        assert_eq!(dense.get(0, 0), Some(&TestField(1)));
        assert_eq!(dense.get(0, 1), Some(&TestField(2)));
        assert_eq!(dense.get(1, 0), Some(&TestField(3)));
        assert_eq!(dense.get(1, 1), Some(&TestField(4)));
    }
}