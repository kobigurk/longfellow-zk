use crate::traits::Field;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Polynomial<F: Field> {
    pub coefficients: Vec<F>,
}

impl<F: Field> Polynomial<F> {
    pub fn new(coefficients: Vec<F>) -> Self {
        let mut poly = Self { coefficients };
        poly.trim();
        poly
    }

    pub fn zero() -> Self {
        Self {
            coefficients: vec![],
        }
    }

    pub fn one() -> Self {
        Self {
            coefficients: vec![F::one()],
        }
    }

    pub fn constant(c: F) -> Self {
        if c == F::zero() {
            Self::zero()
        } else {
            Self {
                coefficients: vec![c],
            }
        }
    }

    pub fn degree(&self) -> Option<usize> {
        if self.coefficients.is_empty() {
            None
        } else {
            Some(self.coefficients.len() - 1)
        }
    }

    pub fn is_zero(&self) -> bool {
        self.coefficients.is_empty()
    }

    pub fn trim(&mut self) {
        while self.coefficients.last() == Some(&F::zero()) {
            self.coefficients.pop();
        }
    }

    pub fn evaluate(&self, x: &F) -> F {
        if self.coefficients.is_empty() {
            return F::zero();
        }

        let mut result = F::zero();
        for coeff in self.coefficients.iter().rev() {
            result = result * x + coeff;
        }
        result
    }

    pub fn scale(&mut self, scalar: F) {
        for coeff in &mut self.coefficients {
            *coeff *= scalar;
        }
        self.trim();
    }

    pub fn add_assign_scaled(&mut self, other: &Self, scalar: F) {
        let max_len = self.coefficients.len().max(other.coefficients.len());
        self.coefficients.resize(max_len, F::zero());

        for (i, coeff) in other.coefficients.iter().enumerate() {
            self.coefficients[i] += *coeff * scalar;
        }
        self.trim();
    }
}

impl<F: Field> Add for Polynomial<F> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl<F: Field> AddAssign for Polynomial<F> {
    fn add_assign(&mut self, rhs: Self) {
        self.add_assign(&rhs);
    }
}

impl<F: Field> AddAssign<&Self> for Polynomial<F> {
    fn add_assign(&mut self, rhs: &Self) {
        let max_len = self.coefficients.len().max(rhs.coefficients.len());
        self.coefficients.resize(max_len, F::zero());

        for (i, coeff) in rhs.coefficients.iter().enumerate() {
            self.coefficients[i] += coeff;
        }
        self.trim();
    }
}

impl<F: Field> Sub for Polynomial<F> {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self {
        self -= rhs;
        self
    }
}

impl<F: Field> SubAssign for Polynomial<F> {
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_assign(&rhs);
    }
}

impl<F: Field> SubAssign<&Self> for Polynomial<F> {
    fn sub_assign(&mut self, rhs: &Self) {
        let max_len = self.coefficients.len().max(rhs.coefficients.len());
        self.coefficients.resize(max_len, F::zero());

        for (i, coeff) in rhs.coefficients.iter().enumerate() {
            self.coefficients[i] -= coeff;
        }
        self.trim();
    }
}

impl<F: Field> Mul for Polynomial<F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        self.mul(&rhs)
    }
}

impl<F: Field> Mul<&Self> for Polynomial<F> {
    type Output = Self;

    fn mul(self, rhs: &Self) -> Self {
        if self.is_zero() || rhs.is_zero() {
            return Self::zero();
        }

        let mut result = vec![F::zero(); self.coefficients.len() + rhs.coefficients.len() - 1];

        for (i, a) in self.coefficients.iter().enumerate() {
            for (j, b) in rhs.coefficients.iter().enumerate() {
                result[i + j] += *a * b;
            }
        }

        Self::new(result)
    }
}

impl<F: Field> MulAssign<F> for Polynomial<F> {
    fn mul_assign(&mut self, rhs: F) {
        self.scale(rhs);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolynomialBasis {
    Monomial,
    Lagrange,
    Newton,
}

#[derive(Clone, Debug)]
pub struct PolynomialInBasis<F: Field> {
    pub coefficients: Vec<F>,
    pub basis: PolynomialBasis,
    pub evaluation_points: Vec<F>,
}

impl<F: Field> PolynomialInBasis<F> {
    pub fn new_monomial(coefficients: Vec<F>) -> Self {
        Self {
            coefficients,
            basis: PolynomialBasis::Monomial,
            evaluation_points: vec![],
        }
    }

    pub fn new_lagrange(values: Vec<F>, evaluation_points: Vec<F>) -> Self {
        assert_eq!(values.len(), evaluation_points.len());
        Self {
            coefficients: values,
            basis: PolynomialBasis::Lagrange,
            evaluation_points,
        }
    }

    pub fn new_newton(coefficients: Vec<F>, evaluation_points: Vec<F>) -> Self {
        assert_eq!(coefficients.len(), evaluation_points.len());
        Self {
            coefficients,
            basis: PolynomialBasis::Newton,
            evaluation_points,
        }
    }

    pub fn evaluate(&self, x: &F) -> F {
        match self.basis {
            PolynomialBasis::Monomial => {
                let poly = Polynomial::new(self.coefficients.clone());
                poly.evaluate(x)
            }
            PolynomialBasis::Lagrange => {
                let mut result = F::zero();
                for (i, &yi) in self.coefficients.iter().enumerate() {
                    let mut term = yi;
                    for (j, &xj) in self.evaluation_points.iter().enumerate() {
                        if i != j {
                            let num = *x - xj;
                            let denom = self.evaluation_points[i] - xj;
                            term *= num * denom.invert().unwrap();
                        }
                    }
                    result += term;
                }
                result
            }
            PolynomialBasis::Newton => {
                let mut result = F::zero();
                let mut product = F::one();
                for (i, &coeff) in self.coefficients.iter().enumerate() {
                    result += coeff * product;
                    if i < self.evaluation_points.len() {
                        product *= *x - self.evaluation_points[i];
                    }
                }
                result
            }
        }
    }

    pub fn to_monomial(&self) -> Polynomial<F> {
        match self.basis {
            PolynomialBasis::Monomial => Polynomial::new(self.coefficients.clone()),
            _ => {
                let degree = self.coefficients.len();
                let mut monomial_coeffs = vec![F::zero(); degree];
                
                for i in 0..degree {
                    let mut basis_poly = Polynomial::one();
                    for j in 0..i {
                        let linear = Polynomial::new(vec![-self.evaluation_points[j], F::one()]);
                        basis_poly = basis_poly * linear;
                    }
                    
                    let scale = match self.basis {
                        PolynomialBasis::Newton => self.coefficients[i],
                        PolynomialBasis::Lagrange => {
                            let mut denom = F::one();
                            for j in 0..degree {
                                if i != j {
                                    denom *= self.evaluation_points[i] - self.evaluation_points[j];
                                }
                            }
                            self.coefficients[i] * denom.invert().unwrap()
                        }
                        _ => unreachable!(),
                    };
                    
                    for (k, &coeff) in basis_poly.coefficients.iter().enumerate() {
                        monomial_coeffs[k] += coeff * scale;
                    }
                }
                
                Polynomial::new(monomial_coeffs)
            }
        }
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
        fn conditional_select(_a: &Self, _b: &Self, _choice: subtle::Choice) -> Self {
            unimplemented!()
        }
    }

    impl subtle::ConstantTimeEq for TestField {
        fn ct_eq(&self, _other: &Self) -> subtle::Choice {
            unimplemented!()
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
    fn test_polynomial_arithmetic() {
        let p1 = Polynomial::new(vec![TestField(1), TestField(2), TestField(3)]);
        let p2 = Polynomial::new(vec![TestField(4), TestField(5)]);

        let sum = p1.clone() + p2.clone();
        assert_eq!(
            sum.coefficients,
            vec![TestField(5), TestField(7), TestField(3)]
        );

        let product = p1 * p2;
        assert_eq!(
            product.coefficients,
            vec![TestField(4), TestField(13), TestField(22), TestField(15)]
        );
    }

    #[test]
    fn test_polynomial_evaluation() {
        let poly = Polynomial::new(vec![TestField(1), TestField(2), TestField(3)]);
        let x = TestField(5);
        let result = poly.evaluate(&x);
        assert_eq!(result, TestField(86));
    }
}