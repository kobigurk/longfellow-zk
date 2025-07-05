use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing Montgomery multiplication fix\n");
    
    // The issue: Montgomery multiplication is producing regular form instead of Montgomery form
    // (a*R) * (b*R) should give (a*b*R), but we're getting (a*b)
    
    // This means we need to multiply the result by R to get the correct Montgomery form
    
    let cpp_omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&cpp_omega_bytes).unwrap();
    println!("omega = {:?}", omega);
    
    // Current (broken) result
    let broken_result = omega * omega;
    println!("Current omega^2 = {:?}", broken_result);
    
    // The broken result is in regular form, but we need Montgomery form
    // So we need to convert it to Montgomery form
    let broken_regular = broken_result.from_montgomery();
    let fixed_result = Fp128::to_montgomery(broken_regular);
    
    println!("Broken regular = {:?}", broken_regular);
    println!("Fixed Montgomery = {:?}", fixed_result);
    
    // Check if this matches the expected result
    let expected_omega2_montgomery_bytes = 246396945533409485782370812413168629798u128.to_le_bytes();
    let expected_omega2_montgomery = Fp128::from_bytes_le(&expected_omega2_montgomery_bytes).unwrap();
    
    println!("Expected omega^2 = {:?}", expected_omega2_montgomery);
    
    if fixed_result == expected_omega2_montgomery {
        println!("✓ Fix works! The issue is that multiplication result needs to be converted to Montgomery form");
    } else {
        println!("✗ Fix doesn't work");
    }
    
    // Test with a simpler approach: since the current result is the regular form,
    // and we need the Montgomery form, we just need to apply to_montgomery to the result
    
    println!("\n=== Simple fix test ===");
    
    // If the multiplication is giving us regular form instead of Montgomery form,
    // we can fix it by not calling from_montgomery on the result
    let omega_regular = omega.from_montgomery();
    let omega2_regular = (omega_regular.limbs[1] as u128) << 64 | omega_regular.limbs[0] as u128;
    let omega2_regular_squared = (omega2_regular * omega2_regular) % (2u128.pow(128) - 2u128.pow(108) + 1);
    
    println!("omega regular = {}", omega2_regular);
    println!("omega^2 regular = {}", omega2_regular_squared);
    
    // Convert this back to Montgomery form
    let omega2_regular_bytes = omega2_regular_squared.to_le_bytes();
    let omega2_correct = Fp128::from_bytes_le(&omega2_regular_bytes).unwrap();
    
    println!("omega^2 correct = {:?}", omega2_correct);
    
    if omega2_correct == expected_omega2_montgomery {
        println!("✓ This approach gives the correct result");
    } else {
        println!("✗ This approach doesn't work either");
    }
}