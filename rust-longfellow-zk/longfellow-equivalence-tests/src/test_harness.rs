use anyhow::Result;
use std::process::Command;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestHarness {
    cpp_build_dir: PathBuf,
    rust_build_dir: PathBuf,
    temp_dir: TempDir,
}

impl TestHarness {
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        
        let cpp_build_dir = PathBuf::from("../../clang-build-release");
        let rust_build_dir = PathBuf::from("../target/release");
        
        Ok(Self {
            cpp_build_dir,
            rust_build_dir,
            temp_dir,
        })
    }

    pub fn compile_cpp_test(&self, test_name: &str) -> Result<PathBuf> {
        let test_exe = self.temp_dir.path().join(format!("cpp_test_{}", test_name));
        
        let output = Command::new("clang++")
            .args(&["-std=c++17", "-O3", "-march=native"])
            .arg("-I")
            .arg("../../lib")
            .arg("-L")
            .arg(&self.cpp_build_dir)
            .arg("-o")
            .arg(&test_exe)
            .arg(format!("cpp_tests/{}.cc", test_name))
            .arg("-llongfellow")
            .output()?;
        
        if !output.status.success() {
            anyhow::bail!(
                "Failed to compile C++ test {}: {}",
                test_name,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        
        Ok(test_exe)
    }

    pub fn run_cpp_executable(&self, exe_path: &Path, input: &str) -> Result<String> {
        let output = Command::new(exe_path)
            .arg("--json")
            .arg(input)
            .output()?;
        
        if !output.status.success() {
            anyhow::bail!(
                "C++ test failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        
        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn generate_cpp_bindings(&self) -> Result<()> {
        let bindings = bindgen::Builder::default()
            .header("cpp_tests/test_wrapper.h")
            .clang_arg("-I../../lib")
            .clang_arg("-std=c++17")
            .generate()
            .map_err(|e| anyhow::anyhow!("Failed to generate bindings: {:?}", e))?;
        
        let out_path = PathBuf::from("src/generated_bindings.rs");
        bindings.write_to_file(out_path)?;
        
        Ok(())
    }

    pub fn build_cpp_wrapper(&self) -> Result<()> {
        cc::Build::new()
            .cpp(true)
            .file("cpp_tests/test_wrapper.cc")
            .include("../../lib")
            .include("cpp_tests")
            .flag("-std=c++17")
            .flag("-O3")
            .compile("cpp_test_wrapper");
        
        Ok(())
    }
}

pub fn compare_outputs<T: PartialEq + std::fmt::Debug>(
    cpp_output: &T,
    rust_output: &T,
    test_name: &str,
) -> Result<()> {
    if cpp_output != rust_output {
        anyhow::bail!(
            "Output mismatch for test '{}': C++ = {:?}, Rust = {:?}",
            test_name,
            cpp_output,
            rust_output
        );
    }
    Ok(())
}

pub fn run_property_test<F, I, O>(
    test_fn: F,
    num_cases: usize,
) -> Result<()>
where
    F: Fn(&I) -> Result<(O, O)>,
    I: proptest::arbitrary::Arbitrary,
    O: PartialEq + std::fmt::Debug,
{
    use proptest::prelude::*;
    
    proptest!(|(input: I)| {
        match test_fn(&input) {
            Ok((cpp_out, rust_out)) => {
                prop_assert_eq!(cpp_out, rust_out);
            }
            Err(e) => {
                prop_assert!(false, "Test failed: {}", e);
            }
        }
    });
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_creation() {
        let harness = TestHarness::new().unwrap();
        assert!(harness.temp_dir.path().exists());
    }
}