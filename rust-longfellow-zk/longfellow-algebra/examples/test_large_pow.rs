use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing large exponents\n");
    
    let omega_32_bytes = [
        0xf8, 0xc8, 0x9e, 0xfd, 0x53, 0xca, 0x47, 0x01,
        0x98, 0x7e, 0xd1, 0x30, 0x76, 0x31, 0x64, 0x5d,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_32_bytes).unwrap();
    
    // Test progressively larger powers to find where it breaks
    let powers_to_test = [
        (16, "matches manual"),
        (32, "unknown"),
        (64, "unknown"),
        (128, "unknown"),
        (256, "unknown"),
        (512, "unknown"),
        (1024, "unknown"),
        (65536, "unknown"), // 2^16
        (1048576, "unknown"), // 2^20
    ];
    
    for (exponent, _note) in powers_to_test {
        let result = omega.pow(&[exponent]);
        println!("omega^{} = {:?}", exponent, result);
        
        // For smaller exponents, verify with manual calculation
        if exponent <= 32 {
            let mut manual = Fp128::one();
            for _ in 0..exponent {
                manual = manual * omega;
            }
            
            if result == manual {
                println!("  ✓ matches manual calculation");
            } else {
                println!("  ✗ DIFFERS from manual calculation!");
                println!("    Manual = {:?}", manual);
            }
        }
    }
    
    // Test the specific failing case: 2^31
    println!("\nTesting the critical exponent 2^31:");
    let exponent_2_31 = 1u64 << 31; // 2^31 = 2147483648
    println!("2^31 = {}", exponent_2_31);
    
    let result_2_31 = omega.pow(&[exponent_2_31]);
    println!("omega^(2^31) = {:?}", result_2_31);
    println!("omega^(2^31) regular = {:?}", result_2_31.from_montgomery());
    
    // Compare with expected -1
    let minus_one = -Fp128::one();
    println!("Expected -1 = {:?}", minus_one);
    println!("Expected -1 regular = {:?}", minus_one.from_montgomery());
    
    if result_2_31 == minus_one {
        println!("✓ omega^(2^31) = -1 (correct)");
    } else {
        println!("✗ omega^(2^31) ≠ -1 (incorrect)");
    }
    
    // Test 2^32
    println!("\nTesting 2^32:");
    let exponent_2_32 = 1u64 << 32; // This might overflow...
    println!("2^32 = {}", exponent_2_32);
    
    // Actually, 2^32 = 4294967296, which might be too large for a single u64
    // Let's check if the issue is overflow
    if exponent_2_32 == 0 {
        println!("2^32 overflowed! Using manual calculation instead.");
        
        // Compute omega^(2^32) = (omega^(2^31))^2
        let result_2_32_manual = result_2_31.square();
        println!("omega^(2^32) = (omega^(2^31))^2 = {:?}", result_2_32_manual);
        
        if result_2_32_manual == Fp128::one() {
            println!("✓ omega^(2^32) = 1 (correct)");
        } else {
            println!("✗ omega^(2^32) ≠ 1 (incorrect)");
        }
    } else {
        let result_2_32 = omega.pow(&[exponent_2_32]);
        println!("omega^(2^32) = {:?}", result_2_32);
        
        if result_2_32 == Fp128::one() {
            println!("✓ omega^(2^32) = 1 (correct)");
        } else {
            println!("✗ omega^(2^32) ≠ 1 (incorrect)");
        }
    }
    
    // Test if the algorithm has issues with very large single u64 values
    println!("\nTesting algorithm with large single values:");
    let large_exp = u64::MAX;
    println!("Testing with u64::MAX = {}", large_exp);
    
    // This will take too long, so let's just test the structure
    // The issue might be that 2^31 and 2^32 are being computed incorrectly
}