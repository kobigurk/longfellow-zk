use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging pow function specifically\n");
    
    // Test simple cases first
    let two = Fp128::from_u64(2);
    
    println!("Testing simple powers:");
    println!("2^1 = {:?}", two.pow(&[1]));
    println!("2^2 = {:?}", two.pow(&[2]));
    println!("2^3 = {:?}", two.pow(&[3]));
    println!("2^4 = {:?}", two.pow(&[4]));
    
    let expected_2_4 = Fp128::from_u64(16);
    if two.pow(&[4]) == expected_2_4 {
        println!("✓ 2^4 = 16 (correct)");
    } else {
        println!("✗ 2^4 ≠ 16 (pow function is broken)");
    }
    
    // Test against manual multiplication
    let two_squared = two * two;
    let two_cubed = two_squared * two;
    let two_fourth = two_cubed * two;
    
    println!("\nManual multiplication:");
    println!("2 * 2 = {:?}", two_squared);
    println!("(2^2) * 2 = {:?}", two_cubed);
    println!("(2^3) * 2 = {:?}", two_fourth);
    
    if two.pow(&[4]) == two_fourth {
        println!("✓ pow([4]) matches manual multiplication");
    } else {
        println!("✗ pow([4]) doesn't match manual multiplication");
    }
    
    // Test larger powers
    println!("\nTesting larger powers:");
    let pow_8 = two.pow(&[8]);
    let pow_16 = two.pow(&[16]);
    let pow_32 = two.pow(&[32]);
    
    println!("2^8 = {:?}", pow_8);
    println!("2^16 = {:?}", pow_16);
    println!("2^32 = {:?}", pow_32);
    
    // 2^32 should be a very large number
    // Let's check if it's reasonable
    
    // Test with the omega value
    println!("\nTesting omega with simple powers:");
    
    let omega_32_bytes = [
        0xf8, 0xc8, 0x9e, 0xfd, 0x53, 0xca, 0x47, 0x01,
        0x98, 0x7e, 0xd1, 0x30, 0x76, 0x31, 0x64, 0x5d,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_32_bytes).unwrap();
    println!("omega = {:?}", omega);
    
    // Test omega^1 vs omega (should be the same)
    let omega_pow_1 = omega.pow(&[1]);
    println!("omega^1 = {:?}", omega_pow_1);
    
    if omega == omega_pow_1 {
        println!("✓ omega^1 = omega");
    } else {
        println!("✗ omega^1 ≠ omega (serious pow bug)");
    }
    
    // Test omega^2 vs omega * omega
    let omega_pow_2 = omega.pow(&[2]);
    let omega_mult_2 = omega * omega;
    println!("omega^2 (pow) = {:?}", omega_pow_2);
    println!("omega * omega = {:?}", omega_mult_2);
    
    if omega_pow_2 == omega_mult_2 {
        println!("✓ omega^2 matches omega * omega");
    } else {
        println!("✗ omega^2 doesn't match omega * omega");
    }
    
    // Test a small specific case that should work
    println!("\nTesting specific case that failed:");
    
    // We know that omega^(2^32) should be 1 in Python
    // Let's test a smaller power first: omega^4 using both methods
    let omega_pow_4_manual = omega * omega * omega * omega;
    let omega_pow_4_function = omega.pow(&[4]);
    
    println!("omega^4 (manual) = {:?}", omega_pow_4_manual);
    println!("omega^4 (pow fn) = {:?}", omega_pow_4_function);
    
    if omega_pow_4_manual == omega_pow_4_function {
        println!("✓ omega^4 matches between methods");
    } else {
        println!("✗ omega^4 differs between methods - pow function is buggy");
    }
}