use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging omega computation in detail\n");
    
    // Load the omega_32 bytes
    let omega_32_bytes = [
        0xf8, 0xc8, 0x9e, 0xfd, 0x53, 0xca, 0x47, 0x01,
        0x98, 0x7e, 0xd1, 0x30, 0x76, 0x31, 0x64, 0x5d,
    ];
    
    let omega_32 = Fp128::from_bytes_le(&omega_32_bytes).unwrap();
    println!("Loaded omega_32 = {:?}", omega_32);
    
    // Check what it looks like in non-Montgomery form
    let omega_32_regular = omega_32.from_montgomery();
    println!("omega_32 non-Montgomery = {:?}", omega_32_regular);
    
    // Check its order by computing powers
    println!("\nTesting powers of omega_32:");
    let mut current = omega_32;
    for i in 1..=35 {
        if current == Fp128::one() {
            println!("omega_32^{} = 1 (order = {})", i, i);
            break;
        }
        if i <= 5 || i >= 30 {
            println!("omega_32^{} = {:?}", i, current);
        } else if i == 6 {
            println!("...");
        }
        current = current * omega_32;
    }
    
    // Test the computation for n=2
    println!("\n=== Computing omega_2 ===");
    
    // For n=2, log_n = 1, so exponent = 2^(32-1) = 2^31
    let log_n = 2u32.trailing_zeros(); // This gives 1
    println!("n = 2, log_n = {}", log_n);
    
    let exponent = 1u64 << (32 - log_n);
    println!("exponent = 2^(32-{}) = 2^{} = {}", log_n, 32-log_n, exponent);
    
    // Compute omega_32^exponent
    let omega_2 = omega_32.pow(&[exponent]);
    println!("omega_2 = omega_32^{} = {:?}", exponent, omega_2);
    
    // Test if it's a 2nd root of unity
    let omega_2_squared = omega_2.square();
    println!("omega_2^2 = {:?}", omega_2_squared);
    
    if omega_2_squared == Fp128::one() {
        println!("✓ omega_2 is a 2nd root of unity");
    } else {
        println!("✗ omega_2 is NOT a 2nd root of unity");
    }
    
    // Check if omega_2 = -1
    let minus_one = -Fp128::one();
    println!("\n-1 = {:?}", minus_one);
    if omega_2 == minus_one {
        println!("✓ omega_2 = -1 (which is correct for a primitive 2nd root of unity)");
    } else {
        println!("✗ omega_2 ≠ -1");
    }
}