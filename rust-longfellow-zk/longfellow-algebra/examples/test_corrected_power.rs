use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing corrected power function for large exponents\n");
    
    let omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_bytes).unwrap();
    
    // Test the original power function
    println!("Original power function:");
    let omega_2_31_original = omega.pow(&[1u64 << 31]);
    println!("omega^(2^31) = {:?}", omega_2_31_original);
    
    // Test the corrected power function
    println!("\nCorrected power function:");
    let omega_2_31_corrected = omega.pow_corrected(&[1u64 << 31]);
    println!("omega^(2^31) = {:?}", omega_2_31_corrected);
    
    // Check if the corrected version equals -1
    let minus_one = -Fp128::one();
    println!("-1 = {:?}", minus_one);
    
    if omega_2_31_corrected == minus_one {
        println!("✓ Corrected omega^(2^31) = -1 (mathematically correct!)");
    } else {
        println!("✗ Corrected version still doesn't equal -1");
    }
    
    // Test that (omega^(2^31))^2 = 1 with the corrected version
    let omega_2_31_squared = omega_2_31_corrected * omega_2_31_corrected;
    let one = Fp128::one();
    
    println!("\n(omega^(2^31))^2 = {:?}", omega_2_31_squared);
    println!("1 = {:?}", one);
    
    if omega_2_31_squared == one {
        println!("✓ (omega^(2^31))^2 = 1 (correct!)");
    } else {
        println!("✗ (omega^(2^31))^2 ≠ 1");
    }
    
    // Test that small powers still work correctly
    println!("\nTesting small powers still work:");
    let omega2_corrected = omega.pow_corrected(&[2]);
    let omega2_original = omega.pow(&[2]);
    
    if omega2_corrected == omega2_original {
        println!("✓ Small powers unchanged by correction");
    } else {
        println!("✗ Correction broke small powers");
    }
    
    // Test omega^(2^32) = 1
    println!("\nTesting omega^(2^32):");
    let omega_2_32 = omega_2_31_corrected * omega_2_31_corrected;
    println!("omega^(2^32) = (omega^(2^31))^2 = {:?}", omega_2_32);
    
    if omega_2_32 == one {
        println!("✓ omega^(2^32) = 1 (omega is a 2^32 root of unity!)");
        println!("🎉 LARGE POWERS ARE NOW WORKING CORRECTLY!");
    } else {
        println!("✗ omega^(2^32) ≠ 1");
    }
}