use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging omega value loading\n");
    
    // Load the C++ omega value
    let cpp_omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&cpp_omega_bytes).unwrap();
    let omega_regular = omega.from_montgomery();
    
    println!("C++ omega bytes: {:02x?}", cpp_omega_bytes);
    println!("Loaded omega (Montgomery): {:?}", omega);
    println!("Loaded omega (regular): {:?}", omega_regular);
    
    // Convert regular form to u128 for comparison
    let omega_u128 = (omega_regular.limbs[1] as u128) << 64 | omega_regular.limbs[0] as u128;
    println!("Omega as u128: {}", omega_u128);
    
    // Expected value from C++/Python
    let expected_omega = 164956748514267535023998284330560247862u128;
    println!("Expected omega: {}", expected_omega);
    
    if omega_u128 == expected_omega {
        println!("✓ Omega value loaded correctly");
    } else {
        println!("✗ Omega value loaded incorrectly");
        println!("  Difference: {}", (omega_u128 as i128) - (expected_omega as i128));
    }
    
    // Test a simple manual power calculation
    println!("\nTesting manual power calculation:");
    
    // Compute omega^2 manually and compare with omega.pow(&[2])
    let omega2_manual = omega * omega;
    let omega2_pow = omega.pow(&[2]);
    
    println!("omega^2 (manual): {:?}", omega2_manual);
    println!("omega^2 (pow): {:?}", omega2_pow);
    
    if omega2_manual == omega2_pow {
        println!("✓ Manual and pow match for omega^2");
    } else {
        println!("✗ Manual and pow differ for omega^2");
    }
    
    // Test omega^4
    let omega4_manual = omega2_manual * omega2_manual;
    let omega4_pow = omega.pow(&[4]);
    
    println!("omega^4 (manual): {:?}", omega4_manual);
    println!("omega^4 (pow): {:?}", omega4_pow);
    
    if omega4_manual == omega4_pow {
        println!("✓ Manual and pow match for omega^4");
    } else {
        println!("✗ Manual and pow differ for omega^4");
    }
    
    // Test a larger power to see where divergence occurs
    println!("\nTesting larger powers:");
    
    let omega8_manual = omega4_manual * omega4_manual;
    let omega16_manual = omega8_manual * omega8_manual;
    let omega16_pow = omega.pow(&[16]);
    
    println!("omega^16 (manual): {:?}", omega16_manual);
    println!("omega^16 (pow): {:?}", omega16_pow);
    
    if omega16_manual == omega16_pow {
        println!("✓ Manual and pow match for omega^16");
    } else {
        println!("✗ Manual and pow differ for omega^16");
        println!("  Power function has a bug!");
    }
}