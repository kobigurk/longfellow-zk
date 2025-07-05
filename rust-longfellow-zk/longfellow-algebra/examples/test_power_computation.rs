use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing power computation for large exponents\n");
    
    let omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_bytes).unwrap();
    
    // Test small powers first
    println!("Testing small powers:");
    let omega2 = omega.pow(&[2]);
    let omega4 = omega.pow(&[4]);
    let omega8 = omega.pow(&[8]);
    
    println!("omega^2 = {:?}", omega2);
    println!("omega^4 = {:?}", omega4);
    println!("omega^8 = {:?}", omega8);
    
    // Test that omega^4 = (omega^2)^2
    let omega2_squared = omega2 * omega2;
    if omega4 == omega2_squared {
        println!("✓ omega^4 = (omega^2)^2 (small powers work correctly)");
    } else {
        println!("✗ omega^4 ≠ (omega^2)^2 (small powers broken)");
    }
    
    // Test medium powers
    println!("\nTesting medium powers:");
    let omega_256 = omega.pow(&[256]);
    let omega_512 = omega.pow(&[512]);
    let omega_1024 = omega.pow(&[1024]);
    
    println!("omega^256 = {:?}", omega_256);
    println!("omega^512 = {:?}", omega_512);
    println!("omega^1024 = {:?}", omega_1024);
    
    // Test that omega^512 = (omega^256)^2
    let omega_256_squared = omega_256 * omega_256;
    if omega_512 == omega_256_squared {
        println!("✓ omega^512 = (omega^256)^2 (medium powers work correctly)");
    } else {
        println!("✗ omega^512 ≠ (omega^256)^2 (medium powers broken)");
    }
    
    // Test large powers
    println!("\nTesting large powers:");
    let omega_2_20 = omega.pow(&[1u64 << 20]);
    let omega_2_25 = omega.pow(&[1u64 << 25]);
    let omega_2_30 = omega.pow(&[1u64 << 30]);
    
    println!("omega^(2^20) = {:?}", omega_2_20);
    println!("omega^(2^25) = {:?}", omega_2_25);
    println!("omega^(2^30) = {:?}", omega_2_30);
    
    // Test the critical case: omega^(2^31) should be -1
    println!("\nTesting critical case: omega^(2^31)");
    let omega_2_31 = omega.pow(&[1u64 << 31]);
    let minus_one = -Fp128::one();
    
    println!("omega^(2^31) = {:?}", omega_2_31);
    println!("-1 = {:?}", minus_one);
    
    if omega_2_31 == minus_one {
        println!("✅ omega^(2^31) = -1 (LARGE POWERS WORK!)");
    } else {
        println!("❌ omega^(2^31) ≠ -1 (large powers still broken)");
        
        // Let's check if they're close by comparing their regular forms
        let omega_2_31_regular = omega_2_31.from_montgomery();
        let minus_one_regular = minus_one.from_montgomery();
        
        let omega_val = (omega_2_31_regular.limbs[1] as u128) << 64 | omega_2_31_regular.limbs[0] as u128;
        let minus_one_val = (minus_one_regular.limbs[1] as u128) << 64 | minus_one_regular.limbs[0] as u128;
        
        println!("omega^(2^31) regular = {}", omega_val);
        println!("-1 regular = {}", minus_one_val);
        
        if omega_val == minus_one_val {
            println!("🤔 Values match in regular form but not in Montgomery form");
        } else {
            let diff = if omega_val > minus_one_val { 
                omega_val - minus_one_val 
            } else { 
                minus_one_val - omega_val 
            };
            println!("Difference: {}", diff);
        }
    }
    
    // Test omega^(2^32) = 1
    println!("\nTesting omega^(2^32):");
    let omega_2_32 = omega_2_31 * omega_2_31;
    let one = Fp128::one();
    
    println!("omega^(2^32) = (omega^(2^31))^2 = {:?}", omega_2_32);
    println!("1 = {:?}", one);
    
    if omega_2_32 == one {
        println!("✅ omega^(2^32) = 1 (omega is a 2^32 root of unity!)");
    } else {
        println!("❌ omega^(2^32) ≠ 1");
    }
}