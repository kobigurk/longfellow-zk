use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Checking omega_32 value\n");
    
    // Load omega_32
    let omega_32_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    let omega_32 = Fp128::from_bytes_le(&omega_32_bytes).unwrap();
    println!("omega_32 = {:?}", omega_32);
    
    // Get its non-Montgomery form
    let omega_32_regular = omega_32.from_montgomery();
    println!("omega_32 regular = {:?}", omega_32_regular);
    
    // Check if it's a 2^32 root of unity by computing omega^(2^32)
    println!("\nChecking powers of omega_32:");
    
    // We can't compute 2^32 directly (too large), but we can do it step by step
    let mut current = omega_32;
    for i in 0..32 {
        current = current.square();
        if i < 5 || i >= 30 {
            println!("omega_32^(2^{}) = {:?}", i+1, current);
        } else if i == 5 {
            println!("...");
        }
        
        if current == Fp128::one() {
            println!("\nFound: omega_32^(2^{}) = 1", i+1);
            break;
        }
    }
    
    // Also check omega_32^(2^31) directly using repeated squaring
    println!("\nComputing omega_32^(2^31) by repeated squaring:");
    let mut omega_2_candidate = omega_32;
    for _ in 0..31 {
        omega_2_candidate = omega_2_candidate.square();
    }
    println!("omega_32^(2^31) = {:?}", omega_2_candidate);
    
    let minus_one = -Fp128::one();
    println!("-1 = {:?}", minus_one);
    println!("Are they equal? {}", omega_2_candidate == minus_one);
}