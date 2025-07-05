use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Verifying new omega value is loaded\n");
    
    // The C++ omega bytes
    let cpp_omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let cpp_omega = Fp128::from_bytes_le(&cpp_omega_bytes).unwrap();
    println!("C++ omega loaded directly = {:?}", cpp_omega);
    println!("C++ omega regular form = {:?}", cpp_omega.from_montgomery());
    
    // Test if this matches what get_root_of_unity returns
    let omega_32_from_function = Fp128::get_root_of_unity(1 << 32);
    
    if let Some(omega_from_fn) = omega_32_from_function {
        println!("omega from get_root_of_unity(2^32) = {:?}", omega_from_fn);
        
        if omega_from_fn == cpp_omega {
            println!("✓ Function returns the C++ omega value");
        } else {
            println!("✗ Function returns a different value");
        }
    } else {
        println!("✗ get_root_of_unity returned None");
    }
    
    // Test the specific computation for n=2
    let omega_2_from_function = Fp128::get_root_of_unity(2);
    
    if let Some(omega_2) = omega_2_from_function {
        println!("\nomega_2 from get_root_of_unity(2) = {:?}", omega_2);
        
        // Test if it's a 2nd root of unity
        let omega_2_squared = omega_2.square();
        println!("omega_2^2 = {:?}", omega_2_squared);
        
        if omega_2_squared == Fp128::one() {
            println!("✓ omega_2 is a 2nd root of unity");
        } else {
            println!("✗ omega_2 is NOT a 2nd root of unity");
        }
        
        // Test if it's -1
        let minus_one = -Fp128::one();
        if omega_2 == minus_one {
            println!("✓ omega_2 = -1 (correct for primitive 2nd root)");
        } else {
            println!("✗ omega_2 ≠ -1");
            println!("  Expected -1 = {:?}", minus_one);
        }
    } else {
        println!("✗ get_root_of_unity(2) returned None");
    }
    
    // Manually compute what omega_2 should be
    // omega_2 = omega_32^(2^(32-1)) = omega_32^(2^31)
    let manual_omega_2 = cpp_omega.pow(&[1u64 << 31]);
    println!("\nManual omega_2 = cpp_omega^(2^31) = {:?}", manual_omega_2);
    
    let manual_omega_2_squared = manual_omega_2.square();
    println!("Manual omega_2^2 = {:?}", manual_omega_2_squared);
    
    let minus_one = -Fp128::one();
    
    if manual_omega_2_squared == Fp128::one() {
        println!("✓ Manual omega_2 is a 2nd root of unity");
        
        if manual_omega_2 == minus_one {
            println!("✓ Manual omega_2 = -1 (perfect!)");
        } else {
            println!("✗ Manual omega_2 ≠ -1 (still has the power bug)");
        }
    } else {
        println!("✗ Manual omega_2 is NOT a 2nd root of unity (power bug persists)");
    }
}