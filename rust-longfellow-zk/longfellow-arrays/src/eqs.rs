use crate::{compute_eq_polynomial, Dense};
use longfellow_algebra::traits::Field;
use longfellow_core::{LongfellowError, Result};

#[derive(Clone, Debug)]
pub struct Eqs<F: Field> {
    dense: Dense<F>,
    fixed_point: Vec<F>,
}

impl<F: Field> Eqs<F> {
    pub fn new(fixed_point: Vec<F>) -> Result<Self> {
        let log_n = fixed_point.len();
        let n = 1 << log_n;
        
        let eq_values = compute_eq_polynomial(log_n, &fixed_point)?;
        
        let dense = Dense::from_vec(n, 1, eq_values)?;
        
        Ok(Self {
            dense,
            fixed_point,
        })
    }

    pub fn fixed_point(&self) -> &[F] {
        &self.fixed_point
    }

    pub fn log_n(&self) -> usize {
        self.fixed_point.len()
    }

    pub fn n(&self) -> usize {
        1 << self.log_n()
    }

    pub fn get(&self, j: usize) -> Result<F> {
        self.dense
            .get(j, 0)
            .copied()
            .ok_or_else(|| {
                LongfellowError::InvalidParameter(format!("Index {} out of bounds", j))
            })
    }

    pub fn bind(&mut self, r: F) {
        self.dense.bind(r);
        self.fixed_point.push(r);
    }

    pub fn bind_all(&mut self, r: &[F]) {
        for &ri in r {
            self.bind(ri);
        }
    }

    pub fn scalar(&self) -> Result<F> {
        self.dense.scalar()
    }

    pub fn as_dense(&self) -> &Dense<F> {
        &self.dense
    }

    pub fn as_dense_mut(&mut self) -> &mut Dense<F> {
        &mut self.dense
    }

    pub fn into_dense(self) -> Dense<F> {
        self.dense
    }

    pub fn verify(&self) -> Result<()> {
        let expected_eq = compute_eq_polynomial(self.fixed_point.len(), &self.fixed_point)?;
        
        for (i, &expected) in expected_eq.iter().enumerate() {
            let actual = self.get(i)?;
            if actual != expected {
                return Err(LongfellowError::Other(format!(
                    "EQ verification failed at index {}: expected {:?}, got {:?}",
                    i, expected, actual
                )));
            }
        }
        
        Ok(())
    }
}

pub fn create_eqs_from_points<F: Field>(points: &[Vec<F>]) -> Result<Vec<Eqs<F>>> {
    points
        .iter()
        .map(|point| Eqs::new(point.clone()))
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
    fn test_eqs_creation() {
        let fixed_point = vec![TestField(20), TestField(30)];
        let eqs = Eqs::new(fixed_point.clone()).unwrap();
        
        assert_eq!(eqs.log_n(), 2);
        assert_eq!(eqs.n(), 4);
        assert_eq!(eqs.fixed_point(), &fixed_point);
    }

    #[test]
    fn test_eqs_values() {
        let fixed_point = vec![TestField(20), TestField(30)];
        let eqs = Eqs::new(fixed_point).unwrap();
        
        let sum = (0..4).map(|i| eqs.get(i).unwrap()).fold(TestField(0), |acc, x| acc + x);
        assert_eq!(sum, TestField(1));
    }

    #[test]
    fn test_eqs_bind() {
        let fixed_point = vec![TestField(20), TestField(30)];
        let mut eqs = Eqs::new(fixed_point).unwrap();
        
        eqs.bind(TestField(40));
        assert_eq!(eqs.fixed_point().len(), 3);
        assert_eq!(eqs.as_dense().n0(), 2);
    }

    #[test]
    fn test_eqs_verify() {
        let fixed_point = vec![TestField(20), TestField(30)];
        let eqs = Eqs::new(fixed_point).unwrap();
        
        assert!(eqs.verify().is_ok());
    }
}