use crate::CornerIndex;
use longfellow_algebra::traits::Field;
use longfellow_core::{LongfellowError, Result};

#[derive(Clone, Debug)]
pub struct Eq<F: Field> {
    log_n: usize,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Field> Eq<F> {
    pub fn new(log_n: usize) -> Self {
        Self {
            log_n,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn log_n(&self) -> usize {
        self.log_n
    }

    pub fn n(&self) -> usize {
        1 << self.log_n
    }

    pub fn evaluate(&self, i: CornerIndex, j: CornerIndex) -> Result<F> {
        if i >= self.n() || j >= self.n() {
            return Err(LongfellowError::InvalidParameter(format!(
                "Indices ({}, {}) out of bounds for EQ of size {}",
                i,
                j,
                self.n()
            )));
        }

        Ok(if i == j { F::one() } else { F::zero() })
    }

    pub fn evaluate_at_points(&self, i: CornerIndex, points: &[F]) -> Result<F> {
        if i >= self.n() {
            return Err(LongfellowError::InvalidParameter(format!(
                "Index {} out of bounds for EQ of size {}",
                i,
                self.n()
            )));
        }

        if points.len() != self.log_n {
            return Err(LongfellowError::InvalidParameter(format!(
                "Expected {} points, got {}",
                self.log_n,
                points.len()
            )));
        }

        let mut result = F::one();
        let mut i_bits = i;

        for &point in points {
            let bit = (i_bits & 1) as u64;
            i_bits >>= 1;

            if bit == 1 {
                result *= point;
            } else {
                result *= F::one() - point;
            }
        }

        Ok(result)
    }

    pub fn bind(&mut self, _r: F) -> Result<()> {
        if self.log_n == 0 {
            return Err(LongfellowError::InvalidParameter(
                "Cannot bind EQ with log_n = 0".to_string(),
            ));
        }
        self.log_n -= 1;
        Ok(())
    }

    pub fn bind_all(&mut self, r: &[F]) -> Result<()> {
        if r.len() > self.log_n {
            return Err(LongfellowError::InvalidParameter(format!(
                "Cannot bind {} values to EQ with log_n = {}",
                r.len(),
                self.log_n
            )));
        }

        for _ in r {
            self.bind(F::zero())?;
        }

        Ok(())
    }

    pub fn scalar(&self) -> Result<F> {
        if self.log_n != 0 {
            return Err(LongfellowError::InvalidParameter(format!(
                "Cannot get scalar from EQ with log_n = {}",
                self.log_n
            )));
        }
        Ok(F::one())
    }
}

pub fn compute_eq_polynomial<F: Field>(log_n: usize, r: &[F]) -> Result<Vec<F>> {
    if r.len() != log_n {
        return Err(LongfellowError::InvalidParameter(format!(
            "Expected {} random values, got {}",
            log_n,
            r.len()
        )));
    }

    let n = 1 << log_n;
    let mut eq_values = vec![F::one(); n];

    for (k, &r_k) in r.iter().enumerate() {
        let stride = 1 << k;
        let one_minus_r = F::one() - r_k;

        for i in (0..n).step_by(2 * stride) {
            for j in 0..stride {
                let idx0 = i + j;
                let idx1 = i + j + stride;
                eq_values[idx0] *= one_minus_r;
                eq_values[idx1] *= r_k;
            }
        }
    }

    Ok(eq_values)
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
    fn test_eq_evaluate() {
        let eq = Eq::<TestField>::new(3);
        
        assert_eq!(eq.evaluate(0, 0).unwrap(), TestField(1));
        assert_eq!(eq.evaluate(1, 1).unwrap(), TestField(1));
        assert_eq!(eq.evaluate(0, 1).unwrap(), TestField(0));
        assert_eq!(eq.evaluate(3, 3).unwrap(), TestField(1));
        assert_eq!(eq.evaluate(2, 5).unwrap(), TestField(0));
    }

    #[test]
    fn test_eq_evaluate_at_points() {
        let eq = Eq::<TestField>::new(2);
        let points = vec![TestField(30), TestField(40)];
        
        let result0 = eq.evaluate_at_points(0, &points).unwrap();
        let result1 = eq.evaluate_at_points(1, &points).unwrap();
        let result2 = eq.evaluate_at_points(2, &points).unwrap();
        let result3 = eq.evaluate_at_points(3, &points).unwrap();
        
        let sum = result0 + result1 + result2 + result3;
        assert_eq!(sum, TestField(1));
    }

    #[test]
    fn test_compute_eq_polynomial() {
        let r = vec![TestField(20), TestField(30)];
        let eq_poly = compute_eq_polynomial(2, &r).unwrap();
        
        assert_eq!(eq_poly.len(), 4);
        
        let sum = eq_poly.iter().fold(TestField(0), |acc, &x| acc + x);
        assert_eq!(sum, TestField(1));
    }
}