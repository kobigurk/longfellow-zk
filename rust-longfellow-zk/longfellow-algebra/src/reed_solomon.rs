use crate::fft::FFT;
use crate::polynomial::Polynomial;
use crate::traits::Field;
use longfellow_core::{LongfellowError, Result};

pub struct ReedSolomon<F: Field> {
    n: usize,
    k: usize,
    omega: F,
    fft: FFT<F>,
}

impl<F: Field> ReedSolomon<F> {
    pub fn new(n: usize, k: usize, omega: F) -> Result<Self> {
        if !n.is_power_of_two() {
            return Err(LongfellowError::InvalidParameter(
                "n must be a power of two".to_string(),
            ));
        }
        
        if k >= n {
            return Err(LongfellowError::InvalidParameter(
                "k must be less than n".to_string(),
            ));
        }

        let fft = FFT::new(n, omega)?;

        Ok(Self { n, k, omega, fft })
    }

    pub fn encode(&self, data: &[F]) -> Result<Vec<F>> {
        if data.len() != self.k {
            return Err(LongfellowError::InvalidParameter(format!(
                "Data length {} does not match k={}",
                data.len(),
                self.k
            )));
        }

        let mut coeffs = vec![F::zero(); self.n];
        coeffs[..self.k].copy_from_slice(data);

        self.fft.forward(&mut coeffs)?;

        Ok(coeffs)
    }

    pub fn encode_systematic(&self, data: &[F]) -> Result<Vec<F>> {
        let encoded = self.encode(data)?;
        
        let mut systematic = vec![F::zero(); self.n];
        systematic[..self.k].copy_from_slice(data);

        let omega_inv = self.omega.invert().ok_or_else(|| {
            LongfellowError::ArithmeticError("Cannot invert omega".to_string())
        })?;
        
        let mut omega_power = F::one();
        for i in 0..self.n - self.k {
            let idx = self.k + i;
            systematic[idx] = encoded[idx];
            
            for j in 0..self.k {
                systematic[idx] -= data[j] * omega_power.pow(&[j as u64]);
            }
            
            omega_power *= omega_inv;
        }

        Ok(systematic)
    }

    pub fn decode(&self, codeword: &[F], erasures: &[usize]) -> Result<Vec<F>> {
        if codeword.len() != self.n {
            return Err(LongfellowError::InvalidParameter(format!(
                "Codeword length {} does not match n={}",
                codeword.len(),
                self.n
            )));
        }

        if erasures.len() > self.n - self.k {
            return Err(LongfellowError::InvalidParameter(
                "Too many erasures for recovery".to_string(),
            ));
        }

        let mut received = codeword.to_vec();
        for &i in erasures {
            received[i] = F::zero();
        }

        let mut values = received.clone();
        self.fft.inverse(&mut values)?;

        Ok(values[..self.k].to_vec())
    }

    pub fn verify(&self, codeword: &[F]) -> Result<bool> {
        if codeword.len() != self.n {
            return Err(LongfellowError::InvalidParameter(format!(
                "Codeword length {} does not match n={}",
                codeword.len(),
                self.n
            )));
        }

        let mut check = codeword.to_vec();
        self.fft.inverse(&mut check)?;

        for i in self.k..self.n {
            if check[i] != F::zero() {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

pub struct SystematicReedSolomon<F: Field> {
    n: usize,
    k: usize,
    generator: Polynomial<F>,
    _evaluation_points: Vec<F>,
}

impl<F: Field> SystematicReedSolomon<F> {
    pub fn new(n: usize, k: usize, primitive_root: F) -> Result<Self> {
        if k >= n {
            return Err(LongfellowError::InvalidParameter(
                "k must be less than n".to_string(),
            ));
        }

        let mut evaluation_points = Vec::with_capacity(n);
        let mut current = F::one();
        for _ in 0..n {
            evaluation_points.push(current);
            current *= primitive_root;
        }

        let mut generator = Polynomial::one();
        for i in 0..n - k {
            let root = Polynomial::new(vec![-evaluation_points[i], F::one()]);
            generator = generator * root;
        }

        Ok(Self {
            n,
            k,
            generator,
            _evaluation_points: evaluation_points,
        })
    }

    pub fn encode(&self, data: &[F]) -> Result<Vec<F>> {
        if data.len() != self.k {
            return Err(LongfellowError::InvalidParameter(format!(
                "Data length {} does not match k={}",
                data.len(),
                self.k
            )));
        }

        let _message_poly = Polynomial::new(data.to_vec());
        
        let shift = self.n - self.k;
        let mut shifted_coeffs = vec![F::zero(); data.len() + shift];
        shifted_coeffs[shift..].copy_from_slice(data);
        let shifted_message = Polynomial::new(shifted_coeffs);

        let remainder = polynomial_division(&shifted_message, &self.generator)?;

        let mut codeword = vec![F::zero(); self.n];
        codeword[..self.k].copy_from_slice(data);
        for (i, &coeff) in remainder.coefficients.iter().enumerate() {
            if i < shift {
                codeword[self.k + i] = -coeff;
            }
        }

        Ok(codeword)
    }
}

fn polynomial_division<F: Field>(
    dividend: &Polynomial<F>,
    divisor: &Polynomial<F>,
) -> Result<Polynomial<F>> {
    if divisor.is_zero() {
        return Err(LongfellowError::ArithmeticError(
            "Division by zero polynomial".to_string(),
        ));
    }

    let mut remainder = dividend.clone();
    let divisor_degree = divisor.degree().unwrap();
    let divisor_lead = divisor.coefficients.last().unwrap();
    let divisor_lead_inv = divisor_lead.invert().ok_or_else(|| {
        LongfellowError::ArithmeticError("Cannot invert leading coefficient".to_string())
    })?;

    while let Some(rem_degree) = remainder.degree() {
        if rem_degree < divisor_degree {
            break;
        }

        let coeff = *remainder.coefficients.last().unwrap() * divisor_lead_inv;
        let shift = rem_degree - divisor_degree;

        for i in 0..=divisor_degree {
            remainder.coefficients[shift + i] -= coeff * divisor.coefficients[i];
        }

        remainder.trim();
    }

    Ok(remainder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestField(u64);

    impl Field for TestField {
        const ZERO: Self = TestField(0);
        const ONE: Self = TestField(1);
        const MODULUS: &'static str = "97";
        const MODULUS_BITS: u32 = 7;

        fn zero() -> Self {
            Self::ZERO
        }

        fn one() -> Self {
            Self::ONE
        }

        fn from_u64(val: u64) -> Self {
            TestField(val % 97)
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
            let mut b = 97;
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
                x += 97;
            }
            
            Some(TestField(x as u64))
        }
    }

    impl std::ops::Add for TestField {
        type Output = Self;
        fn add(self, rhs: Self) -> Self {
            TestField((self.0 + rhs.0) % 97)
        }
    }

    impl std::ops::Sub for TestField {
        type Output = Self;
        fn sub(self, rhs: Self) -> Self {
            TestField((self.0 + 97 - rhs.0) % 97)
        }
    }

    impl std::ops::Mul for TestField {
        type Output = Self;
        fn mul(self, rhs: Self) -> Self {
            TestField((self.0 * rhs.0) % 97)
        }
    }

    impl std::ops::Neg for TestField {
        type Output = Self;
        fn neg(self) -> Self {
            TestField((97 - self.0) % 97)
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
    fn test_polynomial_division() {
        let dividend = Polynomial::new(vec![TestField(1), TestField(2), TestField(3), TestField(4)]);
        let divisor = Polynomial::new(vec![TestField(1), TestField(1)]);
        
        let remainder = polynomial_division(&dividend, &divisor).unwrap();
        assert_eq!(remainder.coefficients.len(), 1);
    }
}