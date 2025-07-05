use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Verifying Montgomery form storage\n");
    
    // Test: is from_bytes_le actually storing values in Montgomery form?
    
    let cpp_omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&cpp_omega_bytes).unwrap();
    let omega_regular = omega.from_montgomery();
    
    // Convert the regular value back to bytes and see if it matches
    let omega_regular_bytes = omega_regular.to_bytes_le();
    
    println!("Original bytes: {:02x?}", cpp_omega_bytes);
    println!("Regular form bytes: {:02x?}", omega_regular_bytes);
    
    if cpp_omega_bytes.to_vec() == omega_regular_bytes {
        println!("✓ Bytes represent regular form values");
        println!("  from_bytes_le loads as regular, then to_montgomery converts to Montgomery");
    } else {
        println!("✗ Bytes don't match regular form");
        println!("  This means from_bytes_le loads in Montgomery form");
    }
    
    // Let's check what from_bytes_le does exactly by looking at the expected values
    let expected_omega_regular = 164956748514267535023998284330560247862u128;
    let expected_omega_regular_bytes = expected_omega_regular.to_le_bytes();
    
    println!("\nExpected omega regular: {}", expected_omega_regular);
    println!("Expected regular bytes: {:02x?}", expected_omega_regular_bytes);
    
    if expected_omega_regular_bytes.to_vec() == cpp_omega_bytes.to_vec() {
        println!("✓ Input bytes are the regular form value");
        println!("  So from_bytes_le should convert to Montgomery form");
    } else {
        println!("✗ Input bytes are NOT the regular form value");
    }
    
    // Test the to_montgomery function directly
    println!("\n=== Testing to_montgomery directly ===");
    
    // Create omega in regular form manually
    let omega_regular_manual = longfellow_algebra::nat::Nat::<2>::from_bytes_le(&expected_omega_regular_bytes).unwrap();
    let omega_montgomery_manual = Fp128::to_montgomery(omega_regular_manual);
    
    println!("Manual regular: {:?}", omega_regular_manual);
    println!("Manual Montgomery: {:?}", omega_montgomery_manual);
    println!("Loaded from bytes: {:?}", omega);
    
    if omega_montgomery_manual == omega {
        println!("✓ to_montgomery works correctly");
    } else {
        println!("✗ to_montgomery is broken");
    }
    
    // Now test the fundamental issue: what should regular form multiplication give?
    println!("\n=== Testing regular form multiplication ===");
    
    let omega_regular_u128 = (omega_regular.limbs[1] as u128) << 64 | omega_regular.limbs[0] as u128;
    println!("omega regular as u128: {}", omega_regular_u128);
    
    // If we multiply regular form values and don't apply Montgomery,
    // we should get the regular form result
    let expected_omega2_regular = 78586892784590695660420324926014672584u128;
    
    // The question: is Montgomery multiplication working on regular form internally?
}