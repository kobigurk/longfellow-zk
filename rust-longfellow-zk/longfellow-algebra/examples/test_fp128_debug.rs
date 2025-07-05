use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing Fp128 Montgomery form details...\n");
    
    // Test basic values
    let one = Fp128::one();
    println!("Fp128::one() internal value: {:?}", one);
    
    let five = Fp128::from_u64(5);
    println!("Fp128::from_u64(5) internal value: {:?}", five);
    
    // Check the constants
    println!("\nConstants:");
    println!("R  = {:?}", Fp128::R);
    println!("R2 = {:?}", Fp128::R2);
    println!("MODULUS = {:?}", Fp128::MODULUS);
    
    // Test that R * R^(-1) = 1 mod p
    // Since ONE = R in Montgomery form, from_montgomery(ONE) should give 1
    println!("\nTesting from_montgomery on ONE (which is R):");
    let one_bytes = one.to_bytes_le();
    println!("one.to_bytes_le() = {:?}", one_bytes);
    
    // Manual check: What is 5 * R mod p?
    // R = 2^128 mod p = 2^128 - (2^128 - 2^108 + 1) = 2^108 - 1
    // So 5 * R mod p = 5 * (2^108 - 1) mod p
    println!("\nChecking 5 in Montgomery form:");
    println!("5 * R mod p should be 5 * (2^108 - 1)");
    
    // Test from_bytes_le 
    println!("\nTesting from_bytes_le:");
    let five_direct = Fp128::from_bytes_le(&[5]).unwrap();
    println!("Fp128::from_bytes_le(&[5]) = {:?}", five_direct);
    let five_back = five_direct.to_bytes_le();
    println!("...to_bytes_le() = {:?}", five_back);
}