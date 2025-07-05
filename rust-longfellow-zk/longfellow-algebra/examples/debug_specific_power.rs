use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging specific power computation\n");
    
    let omega_32_bytes = [
        0xf8, 0xc8, 0x9e, 0xfd, 0x53, 0xca, 0x47, 0x01,
        0x98, 0x7e, 0xd1, 0x30, 0x76, 0x31, 0x64, 0x5d,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_32_bytes).unwrap();
    println!("omega = {:?}", omega);
    
    // Let's compute omega^(2^31) step by step using repeated squaring
    // This will help us see where the divergence from Python happens
    
    println!("\nComputing omega^(2^31) step by step:");
    
    let mut current = omega;
    for k in 1..=31 {
        current = current.square();
        let power = 1u64 << k; // 2^k
        println!("omega^(2^{}) = omega^{} = {:?}", k, power, current);
        
        // Compare with pow function
        let pow_result = omega.pow(&[power]);
        if current == pow_result {
            if k <= 10 || k >= 28 {
                println!("  ✓ matches pow({power})");
            }
        } else {
            println!("  ✗ DIFFERS from pow({power})!");
            println!("    pow result = {:?}", pow_result);
            // This would indicate where the pow function diverges
            break;
        }
    }
    
    // Final comparison with expected result
    let final_result = current; // This is omega^(2^31)
    let expected_minus_one = -Fp128::one();
    
    println!("\nFinal comparison:");
    println!("omega^(2^31) (repeated squaring) = {:?}", final_result);
    println!("Expected -1 = {:?}", expected_minus_one);
    
    if final_result == expected_minus_one {
        println!("✓ omega^(2^31) = -1 (correct!)");
    } else {
        println!("✗ omega^(2^31) ≠ -1");
        
        // Check if squaring the result gives 1
        let squared = final_result.square();
        println!("(omega^(2^31))^2 = {:?}", squared);
        
        if squared == Fp128::one() {
            println!("✓ (omega^(2^31))^2 = 1, so omega^(2^32) = 1");
            println!("  This means omega IS a 2^32 root of unity, but omega^(2^31) ≠ -1");
            println!("  So omega is not a PRIMITIVE 2^32 root of unity");
        } else {
            println!("✗ (omega^(2^31))^2 ≠ 1, so omega is not a 2^32 root of unity at all");
        }
    }
    
    // Let's also check what -1 squared gives us
    let minus_one_squared = expected_minus_one.square();
    println!("\n(-1)^2 = {:?}", minus_one_squared);
    
    if minus_one_squared == Fp128::one() {
        println!("✓ (-1)^2 = 1 (sanity check passed)");
    } else {
        println!("✗ (-1)^2 ≠ 1 (serious arithmetic bug!)");
    }
}