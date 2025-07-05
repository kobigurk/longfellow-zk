use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing large exponent handling\n");
    
    let omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_bytes).unwrap();
    println!("omega = {:?}", omega);
    
    // Test powers of 2 to find where the issue occurs
    println!("\nTesting powers of 2:");
    
    for k in 0..33 {
        let exp = 1u64 << k;
        let result = omega.pow(&[exp]);
        println!("omega^(2^{}) = omega^{} = {:?}", k, exp, result);
        
        // Check if this power gives -1
        if k == 31 {
            let minus_one = -Fp128::one();
            println!("  Is this -1? {}", result == minus_one);
            println!("  -1 = {:?}", minus_one);
            
            // Check if this power squared gives 1
            let result_squared = result * result;
            let one = Fp128::one();
            println!("  (omega^(2^31))^2 = {:?}", result_squared);
            println!("  Is this 1? {}", result_squared == one);
            println!("  1 = {:?}", one);
        }
        
        if k >= 32 {
            break;  // 2^32 might overflow u64
        }
    }
    
    // Test if 2^31 is being computed correctly
    println!("\nTesting 2^31 directly:");
    let exp_31 = 1u64 << 31;
    println!("2^31 = {}", exp_31);
    println!("2^31 in hex = 0x{:016x}", exp_31);
    
    // Test with a different approach: compute omega^(2^30) first, then square it
    println!("\nAlternative computation:");
    let exp_30 = 1u64 << 30;
    let omega_2_30 = omega.pow(&[exp_30]);
    let omega_2_31_alt = omega_2_30 * omega_2_30;
    
    println!("omega^(2^30) = {:?}", omega_2_30);
    println!("(omega^(2^30))^2 = omega^(2^31) = {:?}", omega_2_31_alt);
    
    let omega_2_31_direct = omega.pow(&[exp_31]);
    println!("omega.pow(2^31) = {:?}", omega_2_31_direct);
    
    if omega_2_31_alt == omega_2_31_direct {
        println!("✓ Both methods give the same result");
    } else {
        println!("✗ Methods give different results - power function has issues!");
    }
}