use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging Montgomery multiplication step by step\n");
    
    // Load the C++ omega value that we know is correct
    let cpp_omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&cpp_omega_bytes).unwrap();
    println!("omega = {:?}", omega);
    println!("omega regular = {:?}", omega.from_montgomery());
    
    // Test a sequence of squarings and compare each step with Python
    // We know the divergence starts around step 28, so let's focus on earlier steps
    
    println!("\nTesting repeated squaring step by step:");
    
    let mut current = omega;
    
    // Expected Python results for first few steps (from our earlier calculation)
    let python_results_hex = [
        "0x7c19839f48d38eb442ff6f23413dd836", // omega^1 (original)
        "0xb8388d15751c1040c5c0895524e4275f", // omega^2
        "0x5a616d3fc87cb058134a42f04fc15483", // omega^4
        "0xa0e294484be0a234e4bd28a76cec3426", // omega^8
    ];
    
    for step in 0..4 {
        let power = 1u64 << step; // 2^step
        
        if step > 0 {
            current = current.square();
        }
        
        println!("Step {}: omega^{} = {:?}", step, power, current);
        println!("  Regular form = {:?}", current.from_montgomery());
        
        // Compare with expected Python result
        let expected_python = u128::from_str_radix(&python_results_hex[step][2..], 16).unwrap();
        let current_regular = current.from_montgomery();
        
        // Convert current to u128 for comparison
        let current_as_u128 = (current_regular.limbs[1] as u128) << 64 | current_regular.limbs[0] as u128;
        
        println!("  As u128: {}", current_as_u128);
        println!("  Expected: {}", expected_python);
        println!("  Match: {}", current_as_u128 == expected_python);
        
        if current_as_u128 != expected_python {
            println!("  ❌ DIVERGENCE at step {}!", step);
            
            // Let's examine the Montgomery multiplication in detail
            if step > 0 {
                println!("\n  Debugging the squaring operation that failed:");
                let prev = if step == 1 { omega } else {
                    // Recreate the previous value
                    let mut temp = omega;
                    for _ in 1..step {
                        temp = temp.square();
                    }
                    temp
                };
                
                println!("    prev = {:?}", prev);
                println!("    prev.square() = {:?}", prev.square());
                
                // Test if the issue is in square() vs manual multiplication
                let manual_square = prev * prev;
                println!("    prev * prev = {:?}", manual_square);
                
                if prev.square() == manual_square {
                    println!("    ✓ square() matches manual multiplication");
                } else {
                    println!("    ❌ square() differs from manual multiplication!");
                }
            }
            break;
        } else {
            println!("  ✓ Matches Python");
        }
    }
    
    // Test a single multiplication in detail
    println!("\n=== Detailed multiplication test ===");
    
    let a = omega;
    let b = omega;
    let product = a * b;
    
    println!("a = {:?}", a);
    println!("b = {:?}", b);
    println!("a * b = {:?}", product);
    println!("(a * b) regular = {:?}", product.from_montgomery());
    
    // The second element should match python_results_hex[1]
    let expected = u128::from_str_radix(&python_results_hex[1][2..], 16).unwrap();
    let product_regular = product.from_montgomery();
    let product_as_u128 = (product_regular.limbs[1] as u128) << 64 | product_regular.limbs[0] as u128;
    
    println!("Expected omega^2: {}", expected);
    println!("Got omega^2: {}", product_as_u128);
    println!("Single multiplication correct: {}", product_as_u128 == expected);
}