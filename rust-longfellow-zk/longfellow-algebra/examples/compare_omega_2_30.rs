use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Comparing omega^(2^30) between Rust and Python\n");
    
    let omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_bytes).unwrap();
    
    // Compute omega^(2^30)
    let exp_30 = 1u64 << 30; // 2^30 = 1073741824
    let omega_2_30 = omega.pow(&[exp_30]);
    
    println!("Rust omega^(2^30) = {:?}", omega_2_30);
    
    // Convert to regular form to compare with Python
    let omega_2_30_regular = omega_2_30.from_montgomery();
    let rust_value = (omega_2_30_regular.limbs[1] as u128) << 64 | omega_2_30_regular.limbs[0] as u128;
    
    println!("Rust omega^(2^30) as u128 = {}", rust_value);
    println!("Rust omega^(2^30) hex = 0x{:032x}", rust_value);
    
    // Expected from Python
    let python_value = 232773359852847623982893314813751728182u128;
    println!("\nPython omega^(2^30) = {}", python_value);
    println!("Python omega^(2^30) hex = 0x{:032x}", python_value);
    
    if rust_value == python_value {
        println!("\n✓ Rust and Python match for omega^(2^30)");
        
        // Now test if squaring this gives -1
        let omega_2_30_squared = omega_2_30 * omega_2_30;
        let minus_one = -Fp128::one();
        
        println!("\nTesting (omega^(2^30))^2:");
        println!("(omega^(2^30))^2 = {:?}", omega_2_30_squared);
        println!("-1 = {:?}", minus_one);
        
        if omega_2_30_squared == minus_one {
            println!("✓ (omega^(2^30))^2 = -1 (correct!)");
        } else {
            println!("✗ (omega^(2^30))^2 ≠ -1 (multiplication bug)");
        }
    } else {
        println!("\n✗ Rust and Python differ for omega^(2^30)");
        println!("This means the error accumulates before reaching 2^31");
        
        // Find where the divergence starts by testing smaller powers
        println!("\nTesting where divergence starts:");
        
        let test_powers = [
            (1u64 << 20, 133289649860054206284024727208368918876u128), // 2^20 from Python
            (1u64 << 25, 157782224537855883099133597249074574644u128), // 2^25 from Python  
            (1u64 << 28, 237970330472927234059348937960177525570u128), // 2^28 from Python
            (1u64 << 29, 232773359852847623982893314813751728182u128), // 2^29 from Python (same as 2^30)
        ];
        
        for (exp, expected) in test_powers {
            let rust_result = omega.pow(&[exp]);
            let rust_regular = rust_result.from_montgomery();
            let rust_val = (rust_regular.limbs[1] as u128) << 64 | rust_regular.limbs[0] as u128;
            
            println!("omega^{}: Rust=0x{:032x}, Python=0x{:032x}, Match={}", 
                    exp, rust_val, expected, rust_val == expected);
        }
    }
}