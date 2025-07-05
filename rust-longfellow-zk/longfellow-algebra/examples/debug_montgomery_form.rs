use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging Montgomery form representation\n");
    
    // Load the C++ omega value
    let cpp_omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&cpp_omega_bytes).unwrap();
    println!("omega loaded from bytes = {:?}", omega);
    
    // Get the regular form
    let omega_regular = omega.from_montgomery();
    println!("omega regular form = {:?}", omega_regular);
    
    // Convert back to Montgomery to see if we get the same value
    let omega_back_to_montgomery = Fp128::to_montgomery(omega_regular);
    println!("omega back to Montgomery = {:?}", omega_back_to_montgomery);
    
    if omega == omega_back_to_montgomery {
        println!("✓ Round-trip conversion works");
    } else {
        println!("✗ Round-trip conversion fails!");
        println!("  This suggests from_montgomery or to_montgomery is broken");
    }
    
    // Check the specific values
    let omega_regular_as_u128 = (omega_regular.limbs[1] as u128) << 64 | omega_regular.limbs[0] as u128;
    println!("\nomega regular as u128: {}", omega_regular_as_u128);
    
    // This should be the C++ omega value: 164956748514267535023998284330560247862
    let expected_cpp_omega = 164956748514267535023998284330560247862u128;
    println!("Expected C++ omega: {}", expected_cpp_omega);
    
    if omega_regular_as_u128 == expected_cpp_omega {
        println!("✓ Regular form matches expected C++ omega");
        println!("  This means omega is correctly stored in Montgomery form");
    } else {
        println!("✗ Regular form doesn't match expected C++ omega");
        println!("  This means there's an issue with Montgomery conversion");
    }
    
    // Now test what happens when we manually create the Montgomery form
    println!("\n=== Manual Montgomery form test ===");
    
    // Create omega in regular form, then convert to Montgomery using bytes
    let omega_regular_manual = Fp128::from_bytes_le(&cpp_omega_bytes).unwrap().from_montgomery();
    let omega_montgomery_manual = Fp128::to_montgomery(omega_regular_manual);
    
    println!("Manual regular form = {:?}", omega_regular_manual);
    println!("Manual Montgomery form = {:?}", omega_montgomery_manual);
    
    if omega == omega_montgomery_manual {
        println!("✓ Loaded omega matches manually created Montgomery form");
    } else {
        println!("✗ Loaded omega differs from manually created Montgomery form");
        
        // This might be because the u64 can't hold the full value
        println!("  (Note: u64 overflow - let's use a different approach)");
    }
    
    // Test what we expect omega^2 to be in Montgomery form
    println!("\n=== Expected omega^2 analysis ===");
    
    // From Python: omega^2 regular = 78586892784590695660420324926014672584
    // In Montgomery form it should be: omega^2 * R mod p = 246396945533409485782370812413168629798
    
    let expected_omega2_regular_bytes = 78586892784590695660420324926014672584u128.to_le_bytes();
    let expected_omega2_montgomery_bytes = 246396945533409485782370812413168629798u128.to_le_bytes();
    
    let expected_omega2_regular = Fp128::from_bytes_le(&expected_omega2_regular_bytes).unwrap();
    let expected_omega2_montgomery = Fp128::from_bytes_le(&expected_omega2_montgomery_bytes).unwrap();
    
    println!("Expected omega^2 (regular) = {:?}", expected_omega2_regular);
    println!("Expected omega^2 (Montgomery) = {:?}", expected_omega2_montgomery);
    
    // What we actually get
    let actual_omega2 = omega * omega;
    println!("Actual omega^2 = {:?}", actual_omega2);
    
    if actual_omega2 == expected_omega2_montgomery {
        println!("✓ Rust gives correct Montgomery form result");
    } else if actual_omega2 == expected_omega2_regular {
        println!("✗ Rust gives regular form instead of Montgomery form");
        println!("  This confirms the Montgomery multiplication is not working correctly");
    } else {
        println!("✗ Rust gives completely wrong result");
    }
}