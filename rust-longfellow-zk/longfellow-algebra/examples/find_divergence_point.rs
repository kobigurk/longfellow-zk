use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Finding exact divergence point\n");
    
    let omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_bytes).unwrap();
    
    // Python values for powers of omega up to omega^512
    let python_values = [
        ("omega^1", 0x7c19839f48d38eb442ff6f23413dd836u128),
        ("omega^2", 0x3b1f48f616c4d7a763e9f4d09188cec8u128),
        ("omega^4", 0xebe90ad2e68030704ae16dffa9c8db65u128),
        ("omega^8", 0x115e36a8d89465f515f525ea04608456u128),
        ("omega^16", 0xae0cbe23320c5591652c6b06114263d2u128),
        ("omega^32", 0x7b96aea1396b3a8e457a53cfd7e124a2u128),
        ("omega^64", 0x141fb015a1f40c88b89343c2dc732ddcu128),
        ("omega^128", 0x2b1fd3a15b8a032506d9c08b6d47e7b9u128),
        ("omega^256", 0x0b87828fc5db6302411accf1996d0511u128),
        ("omega^512", 0xd0eee9a6e1ba19ecf8b974d4756a3182u128),
    ];
    
    let mut current = omega;
    
    for (i, (name, expected)) in python_values.iter().enumerate() {
        if i == 0 {
            // omega^1 is just omega
        } else {
            // Square to get next power of 2
            current = current * current;
        }
        
        let current_regular = current.from_montgomery();
        let rust_value = (current_regular.limbs[1] as u128) << 64 | current_regular.limbs[0] as u128;
        
        println!("{}: Rust=0x{:032x}, Python=0x{:032x}", name, rust_value, expected);
        
        if rust_value == *expected {
            println!("  ✓ Match");
        } else {
            println!("  ✗ DIVERGENCE FOUND!");
            println!("  This is where the Montgomery arithmetic error accumulates");
            
            // Let's test a single multiplication to see if that's where the error is
            if i > 0 {
                let prev_name = &python_values[i-1].0;
                println!("\n  Debug: checking previous step");
                println!("  {} squared should give {}", prev_name, name);
                
                // Get the previous value manually
                let mut prev = omega;
                for _ in 1..i {
                    prev = prev * prev;
                }
                
                let prev_squared = prev * prev;
                let prev_squared_regular = prev_squared.from_montgomery();
                let prev_squared_value = (prev_squared_regular.limbs[1] as u128) << 64 | prev_squared_regular.limbs[0] as u128;
                
                println!("  {} * {} = 0x{:032x}", prev_name, prev_name, prev_squared_value);
                
                if prev_squared_value == rust_value {
                    println!("  ✓ Squaring is consistent");
                } else {
                    println!("  ✗ Squaring is inconsistent!");
                }
            }
            break;
        }
    }
}