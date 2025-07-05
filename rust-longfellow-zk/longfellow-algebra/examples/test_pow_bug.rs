use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing pow function bug\n");
    
    // Test a simple case where we can verify manually
    let three = Fp128::from_u64(3);
    
    // Test 3^5 = 243
    // In binary, 5 = 101 (bits 0 and 2 are set)
    
    println!("Testing 3^5:");
    let three_pow_5 = three.pow(&[5]);
    let expected = Fp128::from_u64(243);
    
    println!("3^5 (pow) = {:?}", three_pow_5);
    println!("Expected 243 = {:?}", expected);
    
    if three_pow_5 == expected {
        println!("✓ 3^5 = 243 (correct)");
    } else {
        println!("✗ 3^5 ≠ 243 (pow function is buggy)");
    }
    
    // Test manually: 3^5 = 3 * 3^4 = 3 * 81 = 243
    let three_pow_4 = three.pow(&[4]);
    let manual_3_pow_5 = three * three_pow_4;
    
    println!("3^4 = {:?}", three_pow_4);
    println!("3 * 3^4 = {:?}", manual_3_pow_5);
    
    // Test another case: 3^8 vs 3^8 manual
    let three_pow_8 = three.pow(&[8]);
    let manual_3_8 = {
        let mut result = three;
        for _ in 1..8 {
            result = result * three;
        }
        result
    };
    
    println!("\nTesting 3^8:");
    println!("3^8 (pow) = {:?}", three_pow_8);
    println!("3^8 (manual) = {:?}", manual_3_8);
    
    if three_pow_8 == manual_3_8 {
        println!("✓ 3^8 matches manual calculation");
    } else {
        println!("✗ 3^8 doesn't match manual calculation");
    }
    
    // Test the specific case that's failing
    println!("\nTesting omega^4 (known good vs pow):");
    
    let omega_32_bytes = [
        0xf8, 0xc8, 0x9e, 0xfd, 0x53, 0xca, 0x47, 0x01,
        0x98, 0x7e, 0xd1, 0x30, 0x76, 0x31, 0x64, 0x5d,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_32_bytes).unwrap();
    
    let omega_pow_4_manual = omega * omega * omega * omega;
    let omega_pow_4_function = omega.pow(&[4]);
    
    println!("omega^4 (manual) = {:?}", omega_pow_4_manual);
    println!("omega^4 (pow) = {:?}", omega_pow_4_function);
    
    if omega_pow_4_manual == omega_pow_4_function {
        println!("✓ omega^4 matches (small exponents work)");
    } else {
        println!("✗ omega^4 doesn't match (pow is completely broken)");
    }
    
    // Test a power of 2 exponent that should be simple: omega^16
    let omega_pow_16_function = omega.pow(&[16]);
    
    // Manual: omega^16 = (omega^4)^4
    let omega_4 = omega * omega * omega * omega;
    let omega_pow_16_manual = omega_4 * omega_4 * omega_4 * omega_4;
    
    println!("\nTesting omega^16:");
    println!("omega^16 (pow) = {:?}", omega_pow_16_function);
    println!("omega^16 (manual) = {:?}", omega_pow_16_manual);
    
    if omega_pow_16_manual == omega_pow_16_function {
        println!("✓ omega^16 matches");
    } else {
        println!("✗ omega^16 doesn't match - this shows where pow breaks");
    }
}