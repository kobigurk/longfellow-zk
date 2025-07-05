use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging pow function\n");
    
    // Load omega_32
    let omega_32_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    let omega_32 = Fp128::from_bytes_le(&omega_32_bytes).unwrap();
    println!("omega_32 = {:?}", omega_32);
    
    // For n=2, exponent = 2^(32-1) = 2^31
    let exponent = 1u64 << 31;
    println!("\nExponent = 2^31 = {}", exponent);
    println!("Exponent = 0x{:016x}", exponent);
    
    // Compute omega_2
    let omega_2 = omega_32.pow(&[exponent]);
    println!("\nomega_2 = omega_32^(2^31) = {:?}", omega_2);
    
    // Compare with -1
    let minus_one = -Fp128::one();
    println!("-1 = {:?}", minus_one);
    
    if omega_2 == minus_one {
        println!("\n✓ omega_2 = -1");
    } else {
        println!("\n✗ omega_2 ≠ -1");
    }
    
    // Test squaring
    let omega_2_squared = omega_2.square();
    println!("\nomega_2² = {:?}", omega_2_squared);
    if omega_2_squared == Fp128::one() {
        println!("✓ omega_2² = 1");
    } else {
        println!("✗ omega_2² ≠ 1");
    }
}