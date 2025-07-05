use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging REDC implementation\n");
    
    // I suspect the issue might be in the carry propagation or the way
    // we're handling the modular arithmetic in montgomery_reduce_wide
    
    // Let's test a simple case manually and compare with the algorithm
    
    // Simple test: multiply 1 * 1 in Montgomery form
    let one = Fp128::one();
    println!("one = {:?}", one);
    
    let one_times_one = one * one;
    println!("one * one = {:?}", one_times_one);
    
    // This should give us 1 (the Montgomery form of 1)
    if one_times_one == one {
        println!("✓ 1 * 1 = 1 (correct)");
    } else {
        println!("✗ 1 * 1 ≠ 1 (basic multiplication is broken)");
    }
    
    // Test 2 * 2
    let two = Fp128::from_u64(2);
    let four = Fp128::from_u64(4);
    let two_times_two = two * two;
    
    println!("\ntwo = {:?}", two);
    println!("four = {:?}", four);
    println!("two * two = {:?}", two_times_two);
    
    if two_times_two == four {
        println!("✓ 2 * 2 = 4 (correct)");
    } else {
        println!("✗ 2 * 2 ≠ 4 (multiplication is broken)");
    }
    
    // The issue might be more subtle. Let me check if the problem is
    // specifically with large numbers, not small ones.
    
    println!("\nTesting regular vs Montgomery form interpretation:");
    
    // Check what ONE actually contains
    let one_regular = one.from_montgomery();
    println!("one.from_montgomery() = {:?}", one_regular);
    
    // Create 1 in regular form manually
    let one_manual = longfellow_algebra::nat::Nat::<2>::from_u64(1);
    println!("manual 1 = {:?}", one_manual);
    
    if one_regular == one_manual {
        println!("✓ from_montgomery gives correct regular form");
    } else {
        println!("✗ from_montgomery is broken");
    }
    
    // The key test: if to_montgomery is broken, then the problem
    // might be that all values are stored in regular form, not Montgomery form
    
    // This would explain why multiplication gives regular results:
    // If inputs are in regular form, then regular multiplication
    // followed by "Montgomery reduction" that does nothing
    // would give regular results.
    
    println!("\n=== Hypothesis: Values are stored in regular form ===");
    
    // If values are actually stored in regular form (not Montgomery form),
    // then omega * omega would just be regular multiplication,
    // which would give the correct regular result we're seeing.
    
    let omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_bytes).unwrap();
    let omega_regular_form = omega.from_montgomery();
    
    // If omega is actually stored in regular form, then:
    // omega.from_montgomery() should give a different value (omega / R)
    // But if they're the same, then omega is stored in regular form
    
    let omega_bytes_out = omega.to_bytes_le();
    let omega_regular_bytes_out = omega_regular_form.to_bytes_le();
    
    println!("omega bytes in: {:02x?}", omega_bytes);
    println!("omega bytes out: {:02x?}", omega_bytes_out);
    println!("omega regular bytes: {:02x?}", omega_regular_bytes_out);
    
    if omega_bytes_out == omega_regular_bytes_out {
        println!("✓ omega and omega.from_montgomery() give same bytes");
        println!("  This proves values are stored in REGULAR form, not Montgomery form!");
        println!("  The Montgomery multiplication is a no-op!");
    } else {
        println!("✗ omega and omega.from_montgomery() give different bytes");
        println!("  Values are correctly stored in Montgomery form");
    }
}