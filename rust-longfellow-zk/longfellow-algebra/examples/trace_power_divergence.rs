use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Tracing where power computation diverges from Python\n");
    
    // Use the C++ omega value that we know is mathematically correct
    let omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_bytes).unwrap();
    println!("omega = {:?}", omega);
    
    // Expected values from Python computation
    let expected_values = [
        ("omega^1", "0x7c19839f48d38eb442ff6f23413dd836"),
        ("omega^2", "0x3b1f48f616c4d7a763e9f4d09188cec8"),  
        ("omega^4", "0xebe90ad2e68030704ae16dffa9c8db65"),
        ("omega^8", "0x115e36a8d89465f515f525ea04608456"),
        ("omega^16", "0xae0cbe23320c5591652c6b06114263d2"),
        ("omega^32", "0x7b96aea1396b3a8e457a53cfd7e124a2"),
        ("omega^64", "0x141fb015a1f40c88b89343c2dc732ddc"),
        ("omega^128", "0x2b1fd3a15b8a032506d9c08b6d47e7b9"),
        ("omega^256", "0x0b87828fc5db6302411accf1996d0511"),
    ];
    
    println!("Comparing Rust vs Python values:");
    
    let mut all_match = true;
    let mut current = omega;
    
    for (i, (name, expected_hex)) in expected_values.iter().enumerate() {
        if i == 0 {
            // omega^1 is just omega itself
        } else {
            // Square the previous result to get the next power of 2
            current = current * current;
        }
        
        // Convert expected hex to compare
        let expected_val = u128::from_str_radix(&expected_hex[2..], 16).unwrap();
        let current_regular = current.from_montgomery();
        let current_val = (current_regular.limbs[1] as u128) << 64 | current_regular.limbs[0] as u128;
        
        println!("{}: Rust = 0x{:032x}, Python = {}", name, current_val, expected_hex);
        
        if current_val == expected_val {
            println!("  ✓ Match");
        } else {
            println!("  ✗ MISMATCH - divergence starts here!");
            all_match = false;
            break;
        }
    }
    
    if all_match {
        println!("\n✓ All powers match up to omega^256");
        
        // Continue testing larger powers
        println!("\nTesting larger powers where we expect divergence:");
        
        let large_powers = [512, 1024, 2048, 4096, 8192];
        for &exp in &large_powers {
            let result = omega.pow(&[exp]);
            println!("omega^{} = {:?}", exp, result);
        }
        
        // Test the problematic 2^31 power
        println!("\nTesting omega^(2^31):");
        let omega_2_31 = omega.pow(&[1u64 << 31]);
        println!("omega^(2^31) = {:?}", omega_2_31);
        
        // This should be -1 = 0xfffff000000000000000000000000000
        let minus_one = -Fp128::one();
        println!("-1 = {:?}", minus_one);
        
        if omega_2_31 == minus_one {
            println!("✓ omega^(2^31) = -1 (correct!)");
        } else {
            println!("✗ omega^(2^31) ≠ -1 (still has error accumulation)");
        }
    } else {
        println!("\n⚠️ Found divergence point - investigating Montgomery multiplication");
    }
}