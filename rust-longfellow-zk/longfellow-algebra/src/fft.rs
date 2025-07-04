use crate::permutations::bit_reverse_inplace;
use crate::traits::Field;
use longfellow_core::{LongfellowError, Result};
use rayon::prelude::*;

pub struct FFT<F: Field> {
    size: usize,
    log_size: usize,
    omega: F,
    omega_inv: F,
    twiddle_factors: Vec<F>,
    inv_twiddle_factors: Vec<F>,
}

impl<F: Field> FFT<F> {
    pub fn new(size: usize, omega: F) -> Result<Self> {
        if !size.is_power_of_two() {
            return Err(LongfellowError::InvalidParameter(
                "FFT size must be a power of two".to_string(),
            ));
        }

        let log_size = size.trailing_zeros() as usize;
        let omega_inv = omega.invert().ok_or_else(|| {
            LongfellowError::InvalidParameter("Root of unity is not invertible".to_string())
        })?;

        let twiddle_factors = compute_twiddle_factors(&omega, size);
        let inv_twiddle_factors = compute_twiddle_factors(&omega_inv, size);

        Ok(Self {
            size,
            log_size,
            omega,
            omega_inv,
            twiddle_factors,
            inv_twiddle_factors,
        })
    }

    pub fn forward(&self, coeffs: &mut [F]) -> Result<()> {
        self.transform(coeffs, &self.twiddle_factors)
    }

    pub fn inverse(&self, values: &mut [F]) -> Result<()> {
        self.transform(values, &self.inv_twiddle_factors)?;
        
        let inv_size = F::from_u64(self.size as u64)
            .invert()
            .ok_or_else(|| LongfellowError::ArithmeticError("Cannot invert size".to_string()))?;
        
        values.par_iter_mut().for_each(|v| *v *= inv_size);
        Ok(())
    }

    fn transform(&self, data: &mut [F], twiddles: &[F]) -> Result<()> {
        if data.len() != self.size {
            return Err(LongfellowError::InvalidParameter(format!(
                "Input size {} does not match FFT size {}",
                data.len(),
                self.size
            )));
        }

        // Use SIMD version if available and size is large enough
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        if self.size >= 1024 {
            crate::fft_simd::fft_vectorized(data, twiddles, self.log_size);
            return Ok(());
        }

        bit_reverse_inplace(data);

        let mut stride = 1;
        for level in 0..self.log_size {
            let half_stride = stride;
            stride <<= 1;

            let num_blocks = self.size / stride;
            
            if num_blocks >= 64 {
                data.par_chunks_mut(stride).for_each(|block| {
                    for j in 0..half_stride {
                        let twiddle = twiddles[j << (self.log_size - level - 1)];
                        let a = block[j];
                        let b = block[j + half_stride] * twiddle;
                        block[j] = a + b;
                        block[j + half_stride] = a - b;
                    }
                });
            } else {
                for block in data.chunks_mut(stride) {
                    for j in 0..half_stride {
                        let twiddle = twiddles[j << (self.log_size - level - 1)];
                        let a = block[j];
                        let b = block[j + half_stride] * twiddle;
                        block[j] = a + b;
                        block[j + half_stride] = a - b;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn log_size(&self) -> usize {
        self.log_size
    }
}

fn compute_twiddle_factors<F: Field>(omega: &F, size: usize) -> Vec<F> {
    let mut twiddles = Vec::with_capacity(size);
    let mut current = F::one();
    
    for _ in 0..size {
        twiddles.push(current);
        current *= omega;
    }
    
    twiddles
}

pub struct RFFT<F: Field> {
    fft: FFT<F>,
}

impl<F: Field> RFFT<F> {
    pub fn new(size: usize, omega: F) -> Result<Self> {
        Ok(Self {
            fft: FFT::new(size / 2, omega.square())?,
        })
    }

    pub fn forward(&self, real_coeffs: &mut [F]) -> Result<()> {
        let n = real_coeffs.len();
        if n != self.fft.size * 2 {
            return Err(LongfellowError::InvalidParameter(
                "Input size mismatch".to_string(),
            ));
        }

        let mut complex = vec![F::zero(); self.fft.size];
        for i in 0..self.fft.size {
            complex[i] = real_coeffs[2 * i] + real_coeffs[2 * i + 1];
        }

        self.fft.forward(&mut complex)?;

        let omega_n = self.fft.omega.pow(&[self.fft.size as u64 / 2]);
        let mut omega_power = F::one();
        
        for k in 0..self.fft.size / 2 {
            let ck = complex[k];
            let cn_k = complex[self.fft.size - k];
            
            let even = (ck + cn_k) * F::from_u64(2).invert().unwrap();
            let odd = (ck - cn_k) * omega_power * F::from_u64(2).invert().unwrap();
            
            real_coeffs[k] = even + odd;
            real_coeffs[k + self.fft.size / 2] = even - odd;
            
            omega_power *= omega_n;
        }

        Ok(())
    }

    pub fn inverse(&self, values: &mut [F]) -> Result<()> {
        let n = values.len();
        if n != self.fft.size * 2 {
            return Err(LongfellowError::InvalidParameter(
                "Input size mismatch".to_string(),
            ));
        }

        let omega_n_inv = self.fft.omega_inv.pow(&[self.fft.size as u64 / 2]);
        let mut omega_power = F::one();
        
        let mut complex = vec![F::zero(); self.fft.size];
        
        for k in 0..self.fft.size / 2 {
            let vk = values[k];
            let vn2_k = values[k + self.fft.size / 2];
            
            let ck_real = vk + vn2_k;
            let ck_imag = (vk - vn2_k) * omega_power;
            
            complex[k] = ck_real + ck_imag;
            complex[self.fft.size - k] = ck_real - ck_imag;
            
            omega_power *= omega_n_inv;
        }

        self.fft.inverse(&mut complex)?;

        for i in 0..self.fft.size {
            values[2 * i] = complex[i].double();
            values[2 * i + 1] = F::zero();
        }

        Ok(())
    }
}

pub fn polynomial_multiplication<F: Field>(a: &[F], b: &[F], omega: F) -> Result<Vec<F>> {
    let output_size = a.len() + b.len() - 1;
    let fft_size = output_size.next_power_of_two();
    
    let fft = FFT::new(fft_size, omega)?;
    
    let mut a_padded = a.to_vec();
    a_padded.resize(fft_size, F::zero());
    
    let mut b_padded = b.to_vec();
    b_padded.resize(fft_size, F::zero());
    
    fft.forward(&mut a_padded)?;
    fft.forward(&mut b_padded)?;
    
    a_padded
        .iter_mut()
        .zip(b_padded.iter())
        .for_each(|(a, b)| *a *= b);
    
    fft.inverse(&mut a_padded)?;
    
    a_padded.truncate(output_size);
    Ok(a_padded)
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

    impl Add for TestField {
        type Output = Self;
        fn add(self, rhs: Self) -> Self {
            TestField((self.0 + rhs.0) % 97)
        }
    }

    impl Sub for TestField {
        type Output = Self;
        fn sub(self, rhs: Self) -> Self {
            TestField((self.0 + 97 - rhs.0) % 97)
        }
    }

    impl Mul for TestField {
        type Output = Self;
        fn mul(self, rhs: Self) -> Self {
            TestField((self.0 * rhs.0) % 97)
        }
    }

    impl Neg for TestField {
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
    fn test_fft_forward_inverse() {
        let omega = TestField(35);
        let fft = FFT::new(4, omega).unwrap();
        
        let mut data = vec![
            TestField(1),
            TestField(2),
            TestField(3),
            TestField(4),
        ];
        let original = data.clone();
        
        fft.forward(&mut data).unwrap();
        fft.inverse(&mut data).unwrap();
        
        assert_eq!(data, original);
    }
}