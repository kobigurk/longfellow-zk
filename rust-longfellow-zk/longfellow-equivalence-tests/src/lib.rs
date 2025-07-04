pub mod ffi;
pub mod algebra_tests;
pub mod arrays_tests;
pub mod merkle_tests;
pub mod random_tests;
pub mod ec_tests;
pub mod util_tests;
pub mod cbor_tests;
pub mod circuits_tests;
pub mod test_harness;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase<I, O> {
    pub name: String,
    pub input: I,
    pub expected_output: O,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite<I, O> {
    pub module: String,
    pub description: String,
    pub test_cases: Vec<TestCase<I, O>>,
}

pub trait EquivalenceTest {
    type Input: Serialize + for<'de> Deserialize<'de>;
    type Output: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug;

    fn run_cpp_test(&self, input: &Self::Input) -> Result<Self::Output>;
    fn run_rust_test(&self, input: &Self::Input) -> Result<Self::Output>;

    fn verify_equivalence(&self, test_case: &TestCase<Self::Input, Self::Output>) -> Result<()> {
        let cpp_output = self.run_cpp_test(&test_case.input)?;
        let rust_output = self.run_rust_test(&test_case.input)?;

        if cpp_output != test_case.expected_output {
            anyhow::bail!(
                "C++ output mismatch for test '{}': expected {:?}, got {:?}",
                test_case.name,
                test_case.expected_output,
                cpp_output
            );
        }

        if rust_output != test_case.expected_output {
            anyhow::bail!(
                "Rust output mismatch for test '{}': expected {:?}, got {:?}",
                test_case.name,
                test_case.expected_output,
                rust_output
            );
        }

        if cpp_output != rust_output {
            anyhow::bail!(
                "C++ and Rust outputs differ for test '{}': C++ = {:?}, Rust = {:?}",
                test_case.name,
                cpp_output,
                rust_output
            );
        }

        Ok(())
    }

    fn run_test_suite(&self, suite: &TestSuite<Self::Input, Self::Output>) -> Result<()> {
        println!("Running test suite: {} - {}", suite.module, suite.description);
        
        let mut passed = 0;
        let mut failed = 0;

        for test_case in &suite.test_cases {
            print!("  Test '{}': ", test_case.name);
            match self.verify_equivalence(test_case) {
                Ok(_) => {
                    println!("PASSED");
                    passed += 1;
                }
                Err(e) => {
                    println!("FAILED - {}", e);
                    failed += 1;
                }
            }
        }

        println!(
            "  Summary: {} passed, {} failed, {} total",
            passed,
            failed,
            suite.test_cases.len()
        );

        if failed > 0 {
            anyhow::bail!("{} tests failed", failed);
        }

        Ok(())
    }
}

pub fn load_test_suite<I, O>(path: &Path) -> Result<TestSuite<I, O>>
where
    I: for<'de> Deserialize<'de>,
    O: for<'de> Deserialize<'de>,
{
    let content = std::fs::read_to_string(path)?;
    let suite: TestSuite<I, O> = serde_json::from_str(&content)?;
    Ok(suite)
}

pub fn save_test_suite<I, O>(suite: &TestSuite<I, O>, path: &Path) -> Result<()>
where
    I: Serialize,
    O: Serialize,
{
    let content = serde_json::to_string_pretty(suite)?;
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let suite = TestSuite {
            module: "test".to_string(),
            description: "Test suite".to_string(),
            test_cases: vec![TestCase {
                name: "test1".to_string(),
                input: vec![1, 2, 3],
                expected_output: 6,
            }],
        };

        let json = serde_json::to_string(&suite).unwrap();
        let deserialized: TestSuite<Vec<i32>, i32> = serde_json::from_str(&json).unwrap();
        
        assert_eq!(suite.module, deserialized.module);
        assert_eq!(suite.test_cases.len(), deserialized.test_cases.len());
    }
}