use crate::{EquivalenceTest, TestCase, TestSuite};
use anyhow::Result;
use longfellow_algebra::{
    fft::FFT,
    interpolation::{lagrange_interpolate, newton_interpolate},
    polynomial::Polynomial,
    traits::Field,
    Fp128,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldArithmeticInput {
    pub op: String,
    pub a: String,
    pub b: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldArithmeticOutput {
    pub result: String,
}

pub struct FieldArithmeticTest;

impl EquivalenceTest for FieldArithmeticTest {
    type Input = FieldArithmeticInput;
    type Output = FieldArithmeticOutput;

    fn run_cpp_test(&self, input: &Self::Input) -> Result<Self::Output> {
        let ctx = crate::ffi::TestContext::new();
        let input_json = serde_json::to_string(input)?;
        let output_json = ctx.run_test(crate::ffi::run_field_arithmetic_test, &input_json)?;
        Ok(serde_json::from_str(&output_json)?)
    }

    fn run_rust_test(&self, input: &Self::Input) -> Result<Self::Output> {
        let a = Fp128::from_u64(u64::from_str_radix(&input.a, 10)?);
        
        let result = match input.op.as_str() {
            "add" => {
                let b = Fp128::from_u64(u64::from_str_radix(&input.b.as_ref().unwrap(), 10)?);
                a + b
            }
            "sub" => {
                let b = Fp128::from_u64(u64::from_str_radix(&input.b.as_ref().unwrap(), 10)?);
                a - b
            }
            "mul" => {
                let b = Fp128::from_u64(u64::from_str_radix(&input.b.as_ref().unwrap(), 10)?);
                a * b
            }
            "neg" => -a,
            "inv" => a.invert().ok_or_else(|| anyhow::anyhow!("Cannot invert zero"))?,
            _ => anyhow::bail!("Unknown operation: {}", input.op),
        };
        
        let result_bytes = result.to_bytes_le();
        let result_str = result_bytes
            .iter()
            .rev()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        
        Ok(FieldArithmeticOutput { result: result_str })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFTInput {
    pub size: usize,
    pub omega: String,
    pub coefficients: Vec<String>,
    pub inverse: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FFTOutput {
    pub values: Vec<String>,
}

pub struct FFTTest;

impl EquivalenceTest for FFTTest {
    type Input = FFTInput;
    type Output = FFTOutput;

    fn run_cpp_test(&self, input: &Self::Input) -> Result<Self::Output> {
        let ctx = crate::ffi::TestContext::new();
        let input_json = serde_json::to_string(input)?;
        let output_json = ctx.run_test(crate::ffi::run_fft_test, &input_json)?;
        Ok(serde_json::from_str(&output_json)?)
    }

    fn run_rust_test(&self, input: &Self::Input) -> Result<Self::Output> {
        let omega = Fp128::from_u64(u64::from_str_radix(&input.omega, 10)?);
        let fft = FFT::new(input.size, omega)?;
        
        let mut values: Vec<Fp128> = input
            .coefficients
            .iter()
            .map(|s| Ok(Fp128::from_u64(u64::from_str_radix(s, 10)?)))
            .collect::<Result<Vec<_>>>()?;
        
        if input.inverse {
            fft.inverse(&mut values)?;
        } else {
            fft.forward(&mut values)?;
        }
        
        let output_values = values
            .iter()
            .map(|v| {
                let bytes = v.to_bytes_le();
                bytes
                    .iter()
                    .rev()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>()
            })
            .collect();
        
        Ok(FFTOutput {
            values: output_values,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolynomialInput {
    pub op: String,
    pub poly1: Vec<String>,
    pub poly2: Option<Vec<String>>,
    pub eval_point: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolynomialOutput {
    pub result: Vec<String>,
    pub scalar_result: Option<String>,
}

pub struct PolynomialTest;

impl EquivalenceTest for PolynomialTest {
    type Input = PolynomialInput;
    type Output = PolynomialOutput;

    fn run_cpp_test(&self, input: &Self::Input) -> Result<Self::Output> {
        let ctx = crate::ffi::TestContext::new();
        let input_json = serde_json::to_string(input)?;
        let output_json = ctx.run_test(crate::ffi::run_polynomial_test, &input_json)?;
        Ok(serde_json::from_str(&output_json)?)
    }

    fn run_rust_test(&self, input: &Self::Input) -> Result<Self::Output> {
        let coeffs1: Vec<Fp128> = input
            .poly1
            .iter()
            .map(|s| Ok(Fp128::from_u64(u64::from_str_radix(s, 10)?)))
            .collect::<Result<Vec<_>>>()?;
        
        let poly1 = Polynomial::new(coeffs1);
        
        match input.op.as_str() {
            "add" => {
                let coeffs2: Vec<Fp128> = input
                    .poly2
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|s| Ok(Fp128::from_u64(u64::from_str_radix(s, 10)?)))
                    .collect::<Result<Vec<_>>>()?;
                let poly2 = Polynomial::new(coeffs2);
                let result = poly1 + poly2;
                
                let result_strs = result
                    .coefficients
                    .iter()
                    .map(|c| {
                        let bytes = c.to_bytes_le();
                        bytes
                            .iter()
                            .rev()
                            .map(|b| format!("{:02x}", b))
                            .collect::<String>()
                    })
                    .collect();
                
                Ok(PolynomialOutput {
                    result: result_strs,
                    scalar_result: None,
                })
            }
            "eval" => {
                let x = Fp128::from_u64(u64::from_str_radix(&input.eval_point.as_ref().unwrap(), 10)?);
                let result = poly1.evaluate(&x);
                
                let bytes = result.to_bytes_le();
                let result_str = bytes
                    .iter()
                    .rev()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();
                
                Ok(PolynomialOutput {
                    result: vec![],
                    scalar_result: Some(result_str),
                })
            }
            _ => anyhow::bail!("Unknown polynomial operation: {}", input.op),
        }
    }
}

pub fn create_algebra_test_suites() -> Vec<Box<dyn Fn() -> Result<()>>> {
    vec![
        Box::new(|| {
            let suite = TestSuite {
                module: "algebra".to_string(),
                description: "Field arithmetic tests".to_string(),
                test_cases: vec![
                    TestCase {
                        name: "addition".to_string(),
                        input: FieldArithmeticInput {
                            op: "add".to_string(),
                            a: "12345".to_string(),
                            b: Some("67890".to_string()),
                        },
                        expected_output: FieldArithmeticOutput {
                            result: "00000000000000000000000000014b8d".to_string(),
                        },
                    },
                    TestCase {
                        name: "multiplication".to_string(),
                        input: FieldArithmeticInput {
                            op: "mul".to_string(),
                            a: "12345".to_string(),
                            b: Some("67890".to_string()),
                        },
                        expected_output: FieldArithmeticOutput {
                            result: "0000000000000000000002f66a6aed2".to_string(),
                        },
                    },
                ],
            };
            
            let test = FieldArithmeticTest;
            test.run_test_suite(&suite)
        }),
        Box::new(|| {
            let suite = TestSuite {
                module: "algebra".to_string(),
                description: "FFT tests".to_string(),
                test_cases: vec![
                    TestCase {
                        name: "forward_fft".to_string(),
                        input: FFTInput {
                            size: 4,
                            omega: "17166008163159356379329005055841088858".to_string(),
                            coefficients: vec![
                                "1".to_string(),
                                "2".to_string(),
                                "3".to_string(),
                                "4".to_string(),
                            ],
                            inverse: false,
                        },
                        expected_output: FFTOutput {
                            values: vec![
                                "0000000000000000000000000000000a".to_string(),
                                "ffffe00000000001fffffffffffffffc".to_string(),
                                "ffffe00000000001fffffffffffffffe".to_string(),
                                "00000000000000000000000000000004".to_string(),
                            ],
                        },
                    },
                ],
            };
            
            let test = FFTTest;
            test.run_test_suite(&suite)
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_arithmetic() {
        let test = FieldArithmeticTest;
        let input = FieldArithmeticInput {
            op: "add".to_string(),
            a: "100".to_string(),
            b: Some("200".to_string()),
        };
        
        let rust_output = test.run_rust_test(&input).unwrap();
        assert!(!rust_output.result.is_empty());
    }
}