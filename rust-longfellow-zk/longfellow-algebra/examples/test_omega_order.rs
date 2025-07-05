use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing omega order\n");
    
    let omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_bytes).unwrap();
    let one = Fp128::one();
    
    println!("Testing if omega is a 2^32 root of unity:");
    
    // Test omega^(2^32) = 1
    // Since u64 can't hold 2^32, we need to use larger representation
    // 2^32 = 4294967296 = 0x100000000, which requires more than 32 bits
    
    // We can compute this as (omega^(2^31))^2 since we already computed omega^(2^31)
    let omega_2_31 = omega.pow(&[1u64 << 31]);
    let omega_2_32 = omega_2_31 * omega_2_31;
    
    println!("omega^(2^31) = {:?}", omega_2_31);
    println!("omega^(2^32) = (omega^(2^31))^2 = {:?}", omega_2_32);
    println!("1 = {:?}", one);
    
    if omega_2_32 == one {
        println!("✓ omega^(2^32) = 1 (omega is a 2^32 root of unity)");
    } else {
        println!("✗ omega^(2^32) ≠ 1 (omega is NOT a 2^32 root of unity)");
        
        // Check what power of omega gives 1
        println!("\nTesting smaller orders:");
        let mut current = omega;
        for k in 1..64 {
            if current == one {
                println!("✓ omega^{} = 1 (omega has order {})", k, k);
                break;
            }
            current *= omega;
            if k <= 10 {
                println!("omega^{} = {:?}", k, current);
            }
        }
    }
    
    // Double-check by testing if omega^(2^k) = 1 for smaller k
    println!("\nTesting if omega^(2^k) = 1 for smaller k:");
    for k in 1..33 {
        if k > 31 {
            // For k=32, use the computed value
            if omega_2_32 == one {
                println!("✓ omega^(2^{}) = 1", k);
            } else {
                println!("✗ omega^(2^{}) ≠ 1", k);
            }
        } else {
            let omega_2_k = omega.pow(&[1u64 << k]);
            if omega_2_k == one {
                println!("✓ omega^(2^{}) = 1", k);
                break;  // Found the actual order
            } else {
                println!("✗ omega^(2^{}) ≠ 1", k);
            }
        }
    }
    
    // Test if the -1 we're getting is correct
    let minus_one = -one;
    println!("\nChecking -1:");
    println!("-1 = {:?}", minus_one);
    
    // Convert to regular form to verify it's correct
    let minus_one_regular = minus_one.from_montgomery();
    println!("-1 regular form = {:?}", minus_one_regular);
    
    // This should be p-1
    // p = 2^128 - 2^108 + 1, so p-1 = 2^128 - 2^108 = 0xfffff000000000000000000000000000
    let minus_one_u128 = (minus_one_regular.limbs[1] as u128) << 64 | minus_one_regular.limbs[0] as u128;
    
    // From our python calculation: p-1 = 340282042402384805036647824275747635200
    let expected_minus_one = 340282042402384805036647824275747635200u128;
    
    println!("Expected -1 = {}", expected_minus_one);
    println!("Actual -1 = {}", minus_one_u128);
    
    if minus_one_u128 == expected_minus_one {
        println!("✓ -1 is correct");
    } else {
        println!("✗ -1 is incorrect");
    }
}