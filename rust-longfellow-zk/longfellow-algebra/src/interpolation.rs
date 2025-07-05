use crate::polynomial::Polynomial;
use crate::traits::Field;
use longfellow_core::{LongfellowError, Result};

pub fn lagrange_interpolate<F: Field>(
    points: &[(F, F)],
) -> Result<Polynomial<F>> {
    if points.is_empty() {
        return Ok(Polynomial::zero());
    }

    let n = points.len();
    let mut result = Polynomial::zero();

    for i in 0..n {
        let mut term = Polynomial::constant(points[i].1);
        
        for j in 0..n {
            if i != j {
                let num = Polynomial::new(vec![-points[j].0, F::one()]);
                let denom = points[i].0 - points[j].0;
                let denom_inv = denom.invert().ok_or_else(|| {
                    LongfellowError::ArithmeticError("Division by zero in interpolation".to_string())
                })?;
                
                term = term * num;
                term.scale(denom_inv);
            }
        }
        
        result += term;
    }

    Ok(result)
}

pub fn newton_interpolate<F: Field>(
    points: &[(F, F)],
) -> Result<(Vec<F>, Vec<F>)> {
    if points.is_empty() {
        return Ok((vec![], vec![]));
    }

    let n = points.len();
    let mut divided_differences = vec![vec![F::zero(); n]; n];
    let evaluation_points: Vec<F> = points.iter().map(|(x, _)| *x).collect();

    for i in 0..n {
        divided_differences[i][0] = points[i].1;
    }

    for j in 1..n {
        for i in 0..n - j {
            let num = divided_differences[i + 1][j - 1] - divided_differences[i][j - 1];
            let denom = evaluation_points[i + j] - evaluation_points[i];
            let denom_inv = denom.invert().ok_or_else(|| {
                LongfellowError::ArithmeticError("Division by zero in Newton interpolation".to_string())
            })?;
            divided_differences[i][j] = num * denom_inv;
        }
    }

    let coefficients: Vec<F> = (0..n).map(|i| divided_differences[0][i]).collect();

    Ok((coefficients, evaluation_points))
}

pub fn barycentric_weights<F: Field>(evaluation_points: &[F]) -> Result<Vec<F>> {
    let n = evaluation_points.len();
    let mut weights = vec![F::one(); n];

    for i in 0..n {
        for j in 0..n {
            if i != j {
                let diff = evaluation_points[i] - evaluation_points[j];
                let diff_inv = diff.invert().ok_or_else(|| {
                    LongfellowError::ArithmeticError("Duplicate evaluation points".to_string())
                })?;
                weights[i] *= diff_inv;
            }
        }
    }

    Ok(weights)
}

pub fn barycentric_interpolate<F: Field>(
    evaluation_points: &[F],
    values: &[F],
    weights: &[F],
    x: F,
) -> Result<F> {
    if evaluation_points.len() != values.len() || evaluation_points.len() != weights.len() {
        return Err(LongfellowError::InvalidParameter(
            "Mismatched array lengths".to_string(),
        ));
    }

    for (i, &xi) in evaluation_points.iter().enumerate() {
        if x == xi {
            return Ok(values[i]);
        }
    }

    let mut numerator = F::zero();
    let mut denominator = F::zero();

    for i in 0..evaluation_points.len() {
        let diff = x - evaluation_points[i];
        let diff_inv = diff.invert().ok_or_else(|| {
            LongfellowError::ArithmeticError("Division by zero".to_string())
        })?;
        let term = weights[i] * diff_inv;
        numerator += term * values[i];
        denominator += term;
    }

    let denom_inv = denominator.invert().ok_or_else(|| {
        LongfellowError::ArithmeticError("Division by zero".to_string())
    })?;

    Ok(numerator * denom_inv)
}

pub fn multipoint_evaluate<F: Field>(
    poly: &Polynomial<F>,
    points: &[F],
) -> Vec<F> {
    points.iter().map(|x| poly.evaluate(x)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polynomial::PolynomialInBasis;
    use longfellow_core::Result;
    use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

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
                return None;
            }
            
            let mut a = self.0;
            let mut b = 101;
            let mut x = 1i64;
            let mut y = 0i64;
            
            while a > 1 {
                let q = a / b;
                let tmp = b;
                b = a % b;
                a = tmp;
                
                let tmp = y;
                y = x - (q as i64) * y;
                x = tmp;
            }
            
            if x < 0 {
                x += 101;
            }
            
            Some(TestField(x as u64))
        }
    }

    impl Add for TestField {
        type Output = Self;
        fn add(self, rhs: Self) -> Self {
            TestField((self.0 + rhs.0) % 101)
        }
    }

    impl Sub for TestField {
        type Output = Self;
        fn sub(self, rhs: Self) -> Self {
            TestField((self.0 + 101 - rhs.0) % 101)
        }
    }

    impl Mul for TestField {
        type Output = Self;
        fn mul(self, rhs: Self) -> Self {
            TestField((self.0 * rhs.0) % 101)
        }
    }

    impl Neg for TestField {
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

    impl AddAssign for TestField {
        fn add_assign(&mut self, rhs: Self) {
            *self = *self + rhs;
        }
    }

    impl SubAssign for TestField {
        fn sub_assign(&mut self, rhs: Self) {
            *self = *self - rhs;
        }
    }

    impl MulAssign for TestField {
        fn mul_assign(&mut self, rhs: Self) {
            *self = *self * rhs;
        }
    }

    impl<'a> Add<&'a Self> for TestField {
        type Output = Self;
        fn add(self, rhs: &'a Self) -> Self {
            self + *rhs
        }
    }

    impl<'a> Sub<&'a Self> for TestField {
        type Output = Self;
        fn sub(self, rhs: &'a Self) -> Self {
            self - *rhs
        }
    }

    impl<'a> Mul<&'a Self> for TestField {
        type Output = Self;
        fn mul(self, rhs: &'a Self) -> Self {
            self * *rhs
        }
    }

    impl<'a> AddAssign<&'a Self> for TestField {
        fn add_assign(&mut self, rhs: &'a Self) {
            *self += *rhs;
        }
    }

    impl<'a> SubAssign<&'a Self> for TestField {
        fn sub_assign(&mut self, rhs: &'a Self) {
            *self -= *rhs;
        }
    }

    impl<'a> MulAssign<&'a Self> for TestField {
        fn mul_assign(&mut self, rhs: &'a Self) {
            *self *= *rhs;
        }
    }

    #[test]
    fn test_lagrange_interpolation() {
        let points = vec![
            (TestField(0), TestField(1)),
            (TestField(1), TestField(2)),
            (TestField(2), TestField(5)),
        ];

        let poly = lagrange_interpolate(&points).unwrap();
        
        assert_eq!(poly.evaluate(&TestField(0)), TestField(1));
        assert_eq!(poly.evaluate(&TestField(1)), TestField(2));
        assert_eq!(poly.evaluate(&TestField(2)), TestField(5));
    }

    #[test]
    fn test_newton_interpolation() {
        let points = vec![
            (TestField(0), TestField(1)),
            (TestField(1), TestField(2)),
            (TestField(2), TestField(5)),
        ];

        let (coeffs, eval_points) = newton_interpolate(&points).unwrap();
        let poly = PolynomialInBasis::new_newton(coeffs, eval_points);
        
        assert_eq!(poly.evaluate(&TestField(0)), TestField(1));
        assert_eq!(poly.evaluate(&TestField(1)), TestField(2));
        assert_eq!(poly.evaluate(&TestField(2)), TestField(5));
    }
}