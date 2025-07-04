use crate::affine::affine_interpolation_conditional;
use crate::CornerIndex;
use longfellow_algebra::traits::Field;
use longfellow_core::{LongfellowError, Result};
use std::cmp::Ordering;
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Corner<F: Field> {
    p0: CornerIndex,
    p1: CornerIndex,
    p2: CornerIndex,
    v: F,
}

impl<F: Field> Corner<F> {
    fn new(p0: CornerIndex, p1: CornerIndex, p2: CornerIndex, v: F) -> Self {
        Self { p0, p1, p2, v }
    }
}

impl<F: Field> PartialOrd for Corner<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<F: Field> Ord for Corner<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.p0
            .cmp(&other.p0)
            .then(self.p1.cmp(&other.p1))
            .then(self.p2.cmp(&other.p2))
    }
}

#[derive(Clone, Debug)]
pub struct Sparse<F: Field> {
    n: CornerIndex,
    corners: Vec<Corner<F>>,
}

impl<F: Field> Sparse<F> {
    pub fn new(n: CornerIndex) -> Self {
        assert!(n > 0, "Dimension must be positive");
        Self {
            n,
            corners: Vec::new(),
        }
    }

    pub fn from_corners(n: CornerIndex, corners: Vec<(CornerIndex, CornerIndex, CornerIndex, F)>) -> Result<Self> {
        let corners: Vec<Corner<F>> = corners
            .into_iter()
            .map(|(p0, p1, p2, v)| {
                if p0 >= n || p1 >= n || p2 >= n {
                    Err(LongfellowError::InvalidParameter(format!(
                        "Corner ({}, {}, {}) out of bounds for dimension {}",
                        p0, p1, p2, n
                    )))
                } else {
                    Ok(Corner::new(p0, p1, p2, v))
                }
            })
            .collect::<Result<Vec<_>>>()?;

        let mut sparse = Self { n, corners };
        sparse.canonicalize();
        Ok(sparse)
    }

    pub fn n(&self) -> CornerIndex {
        self.n
    }

    pub fn len(&self) -> usize {
        self.corners.len()
    }

    pub fn is_empty(&self) -> bool {
        self.corners.is_empty()
    }

    pub fn insert(&mut self, p0: CornerIndex, p1: CornerIndex, p2: CornerIndex, v: F) -> Result<()> {
        if p0 >= self.n || p1 >= self.n || p2 >= self.n {
            return Err(LongfellowError::InvalidParameter(format!(
                "Corner ({}, {}, {}) out of bounds for dimension {}",
                p0, p1, p2, self.n
            )));
        }
        self.corners.push(Corner::new(p0, p1, p2, v));
        Ok(())
    }

    pub fn canonicalize(&mut self) {
        if self.corners.is_empty() {
            return;
        }

        self.corners.sort();

        let mut write_idx = 0;
        let mut current = self.corners[0].clone();

        for i in 1..self.corners.len() {
            let corner = &self.corners[i];
            if corner.p0 == current.p0 && corner.p1 == current.p1 && corner.p2 == current.p2 {
                current.v += corner.v;
            } else {
                if current.v != F::zero() {
                    self.corners[write_idx] = current;
                    write_idx += 1;
                }
                current = corner.clone();
            }
        }

        if current.v != F::zero() {
            self.corners[write_idx] = current;
            write_idx += 1;
        }

        self.corners.truncate(write_idx);
    }

    pub fn bind(&mut self, r: F) {
        assert!(self.n > 1, "Cannot bind dimension of size 1");

        let new_n = self.n / 2;
        let mut new_corners = Vec::with_capacity(self.corners.len());

        for corner in &self.corners {
            let (new_p0, pair_p0) = if corner.p0 < new_n {
                (corner.p0, corner.p0 + new_n)
            } else {
                (corner.p0 - new_n, corner.p0 - new_n)
            };

            let is_lower = corner.p0 < new_n;
            
            let partner = self.corners.iter().find(|c| {
                c.p0 == pair_p0 && c.p1 == corner.p1 && c.p2 == corner.p2
            });

            let new_v = match (is_lower, partner) {
                (true, Some(partner)) => affine_interpolation_conditional(corner.v, partner.v, r),
                (true, None) => affine_interpolation_conditional(corner.v, F::zero(), r),
                (false, Some(partner)) => affine_interpolation_conditional(partner.v, corner.v, r),
                (false, None) => affine_interpolation_conditional(F::zero(), corner.v, r),
            };

            if new_v != F::zero() {
                new_corners.push(Corner::new(new_p0, corner.p1, corner.p2, new_v));
            }
        }

        self.n = new_n;
        self.corners = new_corners;
        self.canonicalize();
    }

    pub fn bind_all(&mut self, log_v: usize, r: &[F]) {
        assert!(
            r.len() == log_v,
            "Number of binding values must equal log_v"
        );
        assert!(
            self.n == 1 << log_v,
            "n must equal 2^log_v for bind_all"
        );

        for &ri in r {
            self.bind(ri);
        }

        assert_eq!(self.n, 1);
    }

    pub fn reshape(&mut self) -> Result<()> {
        if self.n != 1 {
            return Err(LongfellowError::InvalidParameter(
                "Can only reshape when n=1".to_string(),
            ));
        }
        Ok(())
    }

    pub fn scalar(&self) -> Result<F> {
        if self.n != 1 {
            return Err(LongfellowError::InvalidParameter(format!(
                "Cannot get scalar from sparse array with n={}",
                self.n
            )));
        }

        Ok(self
            .corners
            .iter()
            .find(|c| c.p0 == 0 && c.p1 == 0 && c.p2 == 0)
            .map(|c| c.v)
            .unwrap_or(F::zero()))
    }

    pub fn get(&self, p0: CornerIndex, p1: CornerIndex, p2: CornerIndex) -> Option<F> {
        self.corners
            .binary_search_by(|c| {
                c.p0.cmp(&p0)
                    .then(c.p1.cmp(&p1))
                    .then(c.p2.cmp(&p2))
            })
            .ok()
            .map(|idx| self.corners[idx].v)
    }

    pub fn to_vec(&self) -> Vec<(CornerIndex, CornerIndex, CornerIndex, F)> {
        self.corners
            .iter()
            .map(|c| (c.p0, c.p1, c.p2, c.v))
            .collect()
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
    fn test_sparse_creation() {
        let sparse = Sparse::<TestField>::new(8);
        assert_eq!(sparse.n(), 8);
        assert_eq!(sparse.len(), 0);
    }

    #[test]
    fn test_sparse_insert_and_get() {
        let mut sparse = Sparse::new(8);
        sparse.insert(1, 2, 3, TestField(42)).unwrap();
        sparse.canonicalize();
        
        assert_eq!(sparse.get(1, 2, 3), Some(TestField(42)));
        assert_eq!(sparse.get(0, 0, 0), None);
    }

    #[test]
    fn test_sparse_canonicalize() {
        let mut sparse = Sparse::new(8);
        sparse.insert(1, 2, 3, TestField(10)).unwrap();
        sparse.insert(1, 2, 3, TestField(20)).unwrap();
        sparse.insert(2, 3, 4, TestField(30)).unwrap();
        sparse.canonicalize();
        
        assert_eq!(sparse.len(), 2);
        assert_eq!(sparse.get(1, 2, 3), Some(TestField(30)));
        assert_eq!(sparse.get(2, 3, 4), Some(TestField(30)));
    }

    #[test]
    fn test_sparse_bind() {
        let corners = vec![
            (0, 0, 0, TestField(1)),
            (1, 0, 0, TestField(2)),
            (2, 0, 0, TestField(3)),
            (3, 0, 0, TestField(4)),
        ];
        
        let mut sparse = Sparse::from_corners(4, corners).unwrap();
        sparse.bind(TestField(50));
        
        assert_eq!(sparse.n(), 2);
        assert_eq!(sparse.len(), 2);
    }
}