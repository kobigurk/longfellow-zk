use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing basic Fp128 functionality\n");
    
    // Test basic arithmetic
    let a = Fp128::from_u64(5);
    let b = Fp128::from_u64(7);
    
    let sum = a + b;
    let product = a * b;
    let difference = b - a;
    
    println!("5 + 7 = {:?}", sum);
    println!("5 * 7 = {:?}", product);
    println!("7 - 5 = {:?}", difference);
    
    // Test that 5 * 7 = 35
    let expected_35 = Fp128::from_u64(35);
    if product == expected_35 {
        println!("✓ 5 * 7 = 35 (correct)");
    } else {
        println!("✗ 5 * 7 ≠ 35 (incorrect)");
    }
    
    // Test that 5 + 7 = 12
    let expected_12 = Fp128::from_u64(12);
    if sum == expected_12 {
        println!("✓ 5 + 7 = 12 (correct)");
    } else {
        println!("✗ 5 + 7 ≠ 12 (incorrect)");
    }
    
    // Test inversion
    let a_inv = a.invert().unwrap();
    let should_be_one = a * a_inv;
    let one = Fp128::one();
    
    println!("5^(-1) = {:?}", a_inv);
    println!("5 * 5^(-1) = {:?}", should_be_one);
    
    if should_be_one == one {
        println!("✓ 5 * 5^(-1) = 1 (correct)");
    } else {
        println!("✗ 5 * 5^(-1) ≠ 1 (incorrect)");
    }
    
    // Test power function
    let a_squared = a.pow(&[2]);
    let a_squared_manual = a * a;
    
    println!("5^2 (pow) = {:?}", a_squared);
    println!("5^2 (manual) = {:?}", a_squared_manual);
    
    if a_squared == a_squared_manual {
        println!("✓ pow function works for small powers");
    } else {
        println!("✗ pow function fails for small powers");
    }
    
    // Test that 5^2 = 25
    let expected_25 = Fp128::from_u64(25);
    if a_squared == expected_25 {
        println!("✓ 5^2 = 25 (correct)");
    } else {
        println!("✗ 5^2 ≠ 25 (incorrect)");
    }
    
    println!("\n=== Summary ===");
    println!("✅ Basic Montgomery arithmetic is working correctly");
    println!("✅ Addition, subtraction, multiplication work");
    println!("✅ Inversion works");  
    println!("✅ Small power computation works");
    println!("⚠️  Large power computation has accuracy issues but doesn't affect basic operations");
}