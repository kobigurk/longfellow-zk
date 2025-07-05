use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Tracing Montgomery reduction step by step\n");
    
    // Let's trace through what happens in Montgomery multiplication manually
    // For omega * omega where omega = 164956748514267535023998284330560247862
    
    let cpp_omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&cpp_omega_bytes).unwrap();
    let omega_regular = omega.from_montgomery();
    
    println!("omega = {:?}", omega);
    println!("omega regular = {:?}", omega_regular);
    
    // Extract the raw limbs for manual calculation
    let omega_bytes = omega.to_bytes_le();
    println!("omega bytes: {:02x?}", omega_bytes);
    
    // In Python, I verified:
    // omega regular = 164956748514267535023998284330560247862
    // omega * omega regular = 78586892784590695660420324926014672584
    // omega * omega Montgomery = 246396945533409485782370812413168629798
    
    let expected_regular = 78586892784590695660420324926014672584u128;
    let expected_montgomery = 246396945533409485782370812413168629798u128;
    
    println!("Expected omega^2 regular: {}", expected_regular);
    println!("Expected omega^2 Montgomery: {}", expected_montgomery);
    
    // What Rust actually gives us
    let actual_result = omega * omega;
    let actual_regular = actual_result.from_montgomery();
    let actual_regular_u128 = (actual_regular.limbs[1] as u128) << 64 | actual_regular.limbs[0] as u128;
    
    println!("Rust omega^2 = {:?}", actual_result);
    println!("Rust omega^2 regular = {:?}", actual_regular);
    println!("Rust omega^2 as u128 = {}", actual_regular_u128);
    
    // The issue: Rust is giving us the regular form result directly
    // This suggests that the Montgomery reduction is dividing by R twice instead of once
    
    // Let's examine the specific issue:
    // Montgomery multiplication should compute: (a*R) * (b*R) / R = a*b*R
    // But we're getting: a*b (which is (a*R) * (b*R) / R^2)
    
    if actual_regular_u128 == expected_regular {
        println!("✓ Rust gives the correct regular form result");
        println!("  BUT this means Montgomery reduction is dividing by R^2 instead of R");
        println!("  The result should be in Montgomery form (a*b*R), not regular form (a*b)");
    } else {
        println!("✗ Rust gives incorrect result entirely");
    }
    
    // Let's trace what should happen:
    // 1. omega is stored as omega_regular * R mod p
    // 2. omega * omega should compute (omega_regular * R) * (omega_regular * R) / R mod p
    // 3. This should give omega_regular^2 * R mod p (Montgomery form of omega^2)
    // 4. from_montgomery() should then give omega_regular^2 mod p
    
    println!("\n=== Montgomery algorithm analysis ===");
    
    // The Montgomery reduction algorithm should:
    // 1. Take a 2N-limb product: a*b (where a,b are in Montgomery form aR, bR)
    // 2. Apply REDC to get (a*b*R) mod p
    // 3. This is the Montgomery form of a*b
    
    // But we're getting the regular form, which suggests REDC is doing an extra division by R
}