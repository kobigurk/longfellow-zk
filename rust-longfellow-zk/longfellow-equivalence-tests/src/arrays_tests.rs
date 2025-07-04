use crate::{EquivalenceTest, TestCase, TestSuite};
use anyhow::Result;
use longfellow_algebra::{traits::Field, Fp128};
use longfellow_arrays::{Dense, Sparse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenseArrayInput {
    pub op: String,
    pub n0: usize,
    pub n1: usize,
    pub values: Vec<String>,
    pub bind_value: Option<String>,
    pub scale_factor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DenseArrayOutput {
    pub n0: usize,
    pub n1: usize,
    pub values: Vec<String>,
}

pub struct DenseArrayTest;

impl EquivalenceTest for DenseArrayTest {
    type Input = DenseArrayInput;
    type Output = DenseArrayOutput;

    fn run_cpp_test(&self, input: &Self::Input) -> Result<Self::Output> {
        let ctx = crate::ffi::TestContext::new();
        let input_json = serde_json::to_string(input)?;
        let output_json = ctx.run_test(crate::ffi::run_dense_array_test, &input_json)?;
        Ok(serde_json::from_str(&output_json)?)
    }

    fn run_rust_test(&self, input: &Self::Input) -> Result<Self::Output> {
        let values: Vec<Fp128> = input
            .values
            .iter()
            .map(|s| Ok(Fp128::from_u64(u64::from_str_radix(s, 10)?)))
            .collect::<Result<Vec<_>>>()?;
        
        let mut dense = Dense::from_vec(input.n0, input.n1, values)?;
        
        match input.op.as_str() {
            "bind" => {
                let r = Fp128::from_u64(u64::from_str_radix(
                    &input.bind_value.as_ref().unwrap(),
                    10,
                )?);
                dense.bind(r);
            }
            "scale" => {
                let factor = Fp128::from_u64(u64::from_str_radix(
                    &input.scale_factor.as_ref().unwrap(),
                    10,
                )?);
                dense.scale(factor, factor);
            }
            _ => anyhow::bail!("Unknown dense array operation: {}", input.op),
        }
        
        let output_values = dense
            .as_slice()
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
        
        Ok(DenseArrayOutput {
            n0: dense.n0(),
            n1: dense.n1(),
            values: output_values,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseArrayInput {
    pub op: String,
    pub n: usize,
    pub corners: Vec<(usize, usize, usize, String)>,
    pub bind_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SparseArrayOutput {
    pub n: usize,
    pub corners: Vec<(usize, usize, usize, String)>,
}

pub struct SparseArrayTest;

impl EquivalenceTest for SparseArrayTest {
    type Input = SparseArrayInput;
    type Output = SparseArrayOutput;

    fn run_cpp_test(&self, input: &Self::Input) -> Result<Self::Output> {
        let ctx = crate::ffi::TestContext::new();
        let input_json = serde_json::to_string(input)?;
        let output_json = ctx.run_test(crate::ffi::run_sparse_array_test, &input_json)?;
        Ok(serde_json::from_str(&output_json)?)
    }

    fn run_rust_test(&self, input: &Self::Input) -> Result<Self::Output> {
        let corners: Vec<(usize, usize, usize, Fp128)> = input
            .corners
            .iter()
            .map(|(p0, p1, p2, v)| {
                Ok((
                    *p0,
                    *p1,
                    *p2,
                    Fp128::from_u64(u64::from_str_radix(v, 10)?),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        
        let mut sparse = Sparse::from_corners(input.n, corners)?;
        
        match input.op.as_str() {
            "bind" => {
                let r = Fp128::from_u64(u64::from_str_radix(
                    &input.bind_value.as_ref().unwrap(),
                    10,
                )?);
                sparse.bind(r);
            }
            _ => anyhow::bail!("Unknown sparse array operation: {}", input.op),
        }
        
        let output_corners = sparse
            .to_vec()
            .iter()
            .map(|(p0, p1, p2, v)| {
                let bytes = v.to_bytes_le();
                let v_str = bytes
                    .iter()
                    .rev()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();
                (*p0, *p1, *p2, v_str)
            })
            .collect();
        
        Ok(SparseArrayOutput {
            n: sparse.n(),
            corners: output_corners,
        })
    }
}

pub fn create_arrays_test_suites() -> Vec<Box<dyn Fn() -> Result<()>>> {
    vec![
        Box::new(|| {
            let suite = TestSuite {
                module: "arrays".to_string(),
                description: "Dense array tests".to_string(),
                test_cases: vec![
                    TestCase {
                        name: "bind_operation".to_string(),
                        input: DenseArrayInput {
                            op: "bind".to_string(),
                            n0: 4,
                            n1: 2,
                            values: vec![
                                "1".to_string(),
                                "2".to_string(),
                                "3".to_string(),
                                "4".to_string(),
                                "5".to_string(),
                                "6".to_string(),
                                "7".to_string(),
                                "8".to_string(),
                            ],
                            bind_value: Some("50".to_string()),
                            scale_factor: None,
                        },
                        expected_output: DenseArrayOutput {
                            n0: 2,
                            n1: 2,
                            values: vec![
                                "00000000000000000000000000000003".to_string(),
                                "00000000000000000000000000000004".to_string(),
                                "00000000000000000000000000000005".to_string(),
                                "00000000000000000000000000000006".to_string(),
                            ],
                        },
                    },
                ],
            };
            
            let test = DenseArrayTest;
            test.run_test_suite(&suite)
        }),
        Box::new(|| {
            let suite = TestSuite {
                module: "arrays".to_string(),
                description: "Sparse array tests".to_string(),
                test_cases: vec![
                    TestCase {
                        name: "bind_operation".to_string(),
                        input: SparseArrayInput {
                            op: "bind".to_string(),
                            n: 4,
                            corners: vec![
                                (0, 0, 0, "10".to_string()),
                                (1, 0, 0, "20".to_string()),
                                (2, 1, 0, "30".to_string()),
                            ],
                            bind_value: Some("50".to_string()),
                        },
                        expected_output: SparseArrayOutput {
                            n: 2,
                            corners: vec![
                                (0, 0, 0, "0000000000000000000000000000000f".to_string()),
                                (0, 1, 0, "0000000000000000000000000000000f".to_string()),
                            ],
                        },
                    },
                ],
            };
            
            let test = SparseArrayTest;
            test.run_test_suite(&suite)
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dense_array() {
        let test = DenseArrayTest;
        let input = DenseArrayInput {
            op: "bind".to_string(),
            n0: 2,
            n1: 2,
            values: vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()],
            bind_value: Some("50".to_string()),
            scale_factor: None,
        };
        
        let rust_output = test.run_rust_test(&input).unwrap();
        assert_eq!(rust_output.n0, 1);
        assert_eq!(rust_output.values.len(), 2);
    }
}