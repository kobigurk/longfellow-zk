use longfellow_equivalence_tests::{
    algebra_tests::{create_algebra_test_suites, FieldArithmeticTest, FFTTest},
    arrays_tests::{create_arrays_test_suites, DenseArrayTest},
    EquivalenceTest, TestCase, TestSuite,
};

fn main() {
    println!("Running Longfellow-ZK Equivalence Tests");
    println!("=======================================\n");

    // Run algebra tests
    println!("1. Field Arithmetic Tests");
    println!("-------------------------");
    run_field_arithmetic_tests();
    
    println!("\n2. FFT Tests");
    println!("------------");
    run_fft_tests();
    
    println!("\n3. Dense Array Tests");
    println!("-------------------");
    run_dense_array_tests();
    
    println!("\n4. Polynomial Tests");
    println!("------------------");
    run_polynomial_tests();
}

fn run_field_arithmetic_tests() {
    let test = FieldArithmeticTest;
    
    // Test addition
    let add_test = TestCase {
        name: "add_100_200".to_string(),
        input: longfellow_equivalence_tests::algebra_tests::FieldArithmeticInput {
            op: "add".to_string(),
            a: "100".to_string(),
            b: Some("200".to_string()),
        },
        expected_output: longfellow_equivalence_tests::algebra_tests::FieldArithmeticOutput {
            result: "300".to_string(), // This will be in hex format
        },
    };
    
    println!("  Testing addition (100 + 200):");
    match test.run_rust_test(&add_test.input) {
        Ok(output) => println!("    Rust output: {:?}", output),
        Err(e) => println!("    Rust error: {}", e),
    }
    
    // Test multiplication
    let mul_test = TestCase {
        name: "mul_7_11".to_string(),
        input: longfellow_equivalence_tests::algebra_tests::FieldArithmeticInput {
            op: "mul".to_string(),
            a: "7".to_string(),
            b: Some("11".to_string()),
        },
        expected_output: longfellow_equivalence_tests::algebra_tests::FieldArithmeticOutput {
            result: "77".to_string(),
        },
    };
    
    println!("  Testing multiplication (7 * 11):");
    match test.run_rust_test(&mul_test.input) {
        Ok(output) => println!("    Rust output: {:?}", output),
        Err(e) => println!("    Rust error: {}", e),
    }
    
    // Test inversion
    let inv_test = TestCase {
        name: "inv_3".to_string(),
        input: longfellow_equivalence_tests::algebra_tests::FieldArithmeticInput {
            op: "inv".to_string(),
            a: "3".to_string(),
            b: None,
        },
        expected_output: longfellow_equivalence_tests::algebra_tests::FieldArithmeticOutput {
            result: "inverse_of_3".to_string(),
        },
    };
    
    println!("  Testing inversion (1/3):");
    match test.run_rust_test(&inv_test.input) {
        Ok(output) => println!("    Rust output: {:?}", output),
        Err(e) => println!("    Rust error: {}", e),
    }
}

fn run_fft_tests() {
    let test = FFTTest;
    
    let fft_test = TestCase {
        name: "fft_size_4".to_string(),
        input: longfellow_equivalence_tests::algebra_tests::FFTInput {
            size: 4,
            omega: "35".to_string(), // A 4th root of unity in our test field
            coefficients: vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()],
            inverse: false,
        },
        expected_output: longfellow_equivalence_tests::algebra_tests::FFTOutput {
            values: vec![], // Will be filled by actual computation
        },
    };
    
    println!("  Testing FFT (size 4):");
    println!("    Input coefficients: [1, 2, 3, 4]");
    match test.run_rust_test(&fft_test.input) {
        Ok(output) => {
            println!("    Rust output (first 2 values):");
            for (i, val) in output.values.iter().take(2).enumerate() {
                println!("      [{}]: {}", i, val);
            }
        }
        Err(e) => println!("    Rust error: {}", e),
    }
}

fn run_dense_array_tests() {
    let test = DenseArrayTest;
    
    let bind_test = TestCase {
        name: "dense_bind".to_string(),
        input: longfellow_equivalence_tests::arrays_tests::DenseArrayInput {
            op: "bind".to_string(),
            n0: 4,
            n1: 2,
            values: vec![
                "10".to_string(), "20".to_string(),
                "30".to_string(), "40".to_string(),
                "50".to_string(), "60".to_string(),
                "70".to_string(), "80".to_string(),
            ],
            bind_value: Some("100".to_string()),
            scale_factor: None,
        },
        expected_output: longfellow_equivalence_tests::arrays_tests::DenseArrayOutput {
            n0: 2,
            n1: 2,
            values: vec![],
        },
    };
    
    println!("  Testing dense array bind operation:");
    println!("    Initial: 4x2 array with values [10,20,30,40,50,60,70,80]");
    println!("    Binding with r=100");
    match test.run_rust_test(&bind_test.input) {
        Ok(output) => {
            println!("    Result: {}x{} array", output.n0, output.n1);
            println!("    First 2 values:");
            for (i, val) in output.values.iter().take(2).enumerate() {
                println!("      [{}]: {}", i, val);
            }
        }
        Err(e) => println!("    Rust error: {}", e),
    }
}

fn run_polynomial_tests() {
    use longfellow_equivalence_tests::algebra_tests::{PolynomialTest, PolynomialInput, PolynomialOutput};
    
    let test = PolynomialTest;
    
    // Test polynomial evaluation
    let eval_test = TestCase {
        name: "poly_eval".to_string(),
        input: PolynomialInput {
            op: "eval".to_string(),
            poly1: vec!["1".to_string(), "2".to_string(), "3".to_string()], // 1 + 2x + 3x^2
            poly2: None,
            eval_point: Some("5".to_string()),
        },
        expected_output: PolynomialOutput {
            result: vec![],
            scalar_result: Some("86".to_string()),
        },
    };
    
    println!("  Testing polynomial evaluation:");
    println!("    Polynomial: 1 + 2x + 3x²");
    println!("    Evaluation point: x = 5");
    match test.run_rust_test(&eval_test.input) {
        Ok(output) => {
            println!("    Result: {:?}", output.scalar_result);
        }
        Err(e) => println!("    Rust error: {}", e),
    }
    
    // Test polynomial addition
    let add_test = TestCase {
        name: "poly_add".to_string(),
        input: PolynomialInput {
            op: "add".to_string(),
            poly1: vec!["1".to_string(), "2".to_string(), "3".to_string()],
            poly2: Some(vec!["4".to_string(), "5".to_string()]),
            eval_point: None,
        },
        expected_output: PolynomialOutput {
            result: vec![],
            scalar_result: None,
        },
    };
    
    println!("  Testing polynomial addition:");
    println!("    P1: 1 + 2x + 3x²");
    println!("    P2: 4 + 5x");
    match test.run_rust_test(&add_test.input) {
        Ok(output) => {
            println!("    Result coefficients:");
            for (i, coeff) in output.result.iter().enumerate() {
                println!("      x^{}: {}", i, coeff);
            }
        }
        Err(e) => println!("    Rust error: {}", e),
    }
}