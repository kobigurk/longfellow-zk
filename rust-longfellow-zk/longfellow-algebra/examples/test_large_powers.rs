use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing large power computations with fixed Montgomery arithmetic\n");
    
    // Test omega^(2^31) to see if the power function works correctly now
    let omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_bytes).unwrap();
    println!("omega = {:?}", omega);
    
    // Test small powers first
    let omega2 = omega * omega;
    let omega4 = omega2 * omega2;
    let omega8 = omega4 * omega4;
    
    println!("omega^2 = {:?}", omega2);
    println!("omega^4 = {:?}", omega4); 
    println!("omega^8 = {:?}", omega8);
    
    // Test using pow function with small exponents
    let omega2_pow = omega.pow(&[2u64]);
    let omega4_pow = omega.pow(&[4u64]);
    let omega8_pow = omega.pow(&[8u64]);
    
    println!("\nUsing pow function:");
    println!("omega.pow(2) = {:?}", omega2_pow);
    println!("omega.pow(4) = {:?}", omega4_pow);
    println!("omega.pow(8) = {:?}", omega8_pow);
    
    if omega2 == omega2_pow && omega4 == omega4_pow && omega8 == omega8_pow {
        println!("✓ pow function matches manual multiplication for small powers");
    } else {
        println!("✗ pow function doesn't match manual multiplication");
    }
    
    // Test medium powers (where the bug occurred before)
    println!("\nTesting medium powers:");
    
    let exp_256 = 1u64 << 8;  // 2^8 = 256
    let exp_1024 = 1u64 << 10; // 2^10 = 1024
    let exp_4096 = 1u64 << 12; // 2^12 = 4096
    
    let omega_256 = omega.pow(&[exp_256]);
    let omega_1024 = omega.pow(&[exp_1024]);
    let omega_4096 = omega.pow(&[exp_4096]);
    
    println!("omega^256 = {:?}", omega_256);
    println!("omega^1024 = {:?}", omega_1024);
    println!("omega^4096 = {:?}", omega_4096);
    
    // Test the specific problematic power: 2^28
    println!("\nTesting problematic power: 2^28");
    let exp_28 = 1u64 << 28;  // 2^28
    let omega_2_28 = omega.pow(&[exp_28]);
    println!("omega^(2^28) = {:?}", omega_2_28);
    
    // Test the key power: 2^31 (should give a square root of -1)
    println!("\nTesting 2^31 power:");
    let exp_31 = 1u64 << 31;  // 2^31
    let omega_2_31 = omega.pow(&[exp_31]);
    println!("omega^(2^31) = {:?}", omega_2_31);
    
    // Check if omega^(2^31) is a square root of -1
    let omega_2_31_squared = omega_2_31 * omega_2_31;
    let minus_one = -Fp128::one();
    
    println!("(omega^(2^31))^2 = {:?}", omega_2_31_squared);
    println!("-1 = {:?}", minus_one);
    
    if omega_2_31_squared == minus_one {
        println!("✓ omega^(2^31) is a square root of -1");
        println!("✓ Large power computation is working correctly!");
    } else {
        println!("✗ omega^(2^31) is NOT a square root of -1");
        println!("  Large power computation still has issues");
    }
}