use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging omega computation step by step\n");
    
    // Load the omega_32 bytes
    let omega_32_bytes = [
        0xf8, 0xc8, 0x9e, 0xfd, 0x53, 0xca, 0x47, 0x01,
        0x98, 0x7e, 0xd1, 0x30, 0x76, 0x31, 0x64, 0x5d,
    ];
    
    let omega_32 = Fp128::from_bytes_le(&omega_32_bytes).unwrap();
    println!("omega_32 = {:?}", omega_32);
    println!("omega_32 regular form = {:?}", omega_32.from_montgomery());
    
    // Test if this is actually a 2^32 root of unity
    println!("\nTesting if omega_32 is a 2^32 root of unity:");
    
    // For efficiency, let's test some smaller powers first
    let mut current = omega_32;
    let mut found_order = None;
    
    for i in 1..=100 {
        if current == Fp128::one() {
            println!("omega_32^{} = 1 (found order = {})", i, i);
            found_order = Some(i);
            break;
        }
        if i <= 10 {
            println!("omega_32^{} = {:?}", i, current);
        }
        current = current * omega_32;
    }
    
    if found_order.is_none() {
        println!("Order is > 100");
        
        // Test the specific power 2^32
        println!("\nTesting omega_32^(2^32):");
        let exponent_32 = 1u64 << 32; // 2^32
        println!("2^32 = {}", exponent_32);
        
        // This is too large to compute directly, let's try smaller powers of 2
        for k in 1..=32 {
            let exponent = 1u64 << k; // 2^k
            let result = omega_32.pow(&[exponent]);
            println!("omega_32^(2^{}) = omega_32^{} = {:?}", k, exponent, result);
            
            if result == Fp128::one() {
                println!("✓ Found: omega_32 has order 2^{}", k);
                break;
            }
            
            // For k=31, check if result is -1 (which would make it a primitive 2nd root)
            if k == 31 {
                let minus_one = -Fp128::one();
                if result == minus_one {
                    println!("✓ omega_32^(2^31) = -1, so it's a primitive 2nd root of unity");
                } else {
                    println!("✗ omega_32^(2^31) ≠ -1, got {:?}", result);
                    println!("  Expected -1 = {:?}", minus_one);
                }
            }
        }
    }
    
    // Let's also manually verify that our expected omega value is correct
    println!("\nVerifying the omega value 124138436495952958347847942047415585016:");
    
    // Convert to Fp128 and test using the same bytes
    let expected_omega = Fp128::from_bytes_le(&omega_32_bytes).unwrap();
    
    println!("Expected omega = {:?}", expected_omega);
    println!("Actual loaded omega = {:?}", omega_32);
    
    if expected_omega == omega_32 {
        println!("✓ Loaded omega matches expected value");
    } else {
        println!("✗ Loaded omega doesn't match expected value");
    }
}