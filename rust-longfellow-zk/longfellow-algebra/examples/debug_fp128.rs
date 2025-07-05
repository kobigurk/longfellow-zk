use longfellow_algebra::field::fp128::Fp128;
use longfellow_algebra::traits::Field;

fn main() {
    // Print field constants
    println!("Field constants:");
    println!("Modulus p = 2^128 - 2^108 + 1");
    println!("p = 0x{:032x}{:016x}", Fp128::MODULUS.limbs[1], Fp128::MODULUS.limbs[0]);
    println!("Expected: 0xFFFFF00000000000000000000000001");
    
    println!("\nR = 2^128 mod p");
    println!("R = 0x{:016x}{:016x}", Fp128::R.limbs[1], Fp128::R.limbs[0]);
    
    println!("\nR2 = R^2 mod p");
    println!("R2 = 0x{:016x}{:016x}", Fp128::R2.limbs[1], Fp128::R2.limbs[0]);
    
    println!("\nINV = -p^(-1) mod 2^64");
    println!("INV = 0x{:016x}", Fp128::INV);
    
    // Test Montgomery form conversion
    println!("\n\nTesting Montgomery form:");
    
    // Create 1 in normal form and convert to Montgomery
    let one_normal = longfellow_algebra::nat::Nat::<2>::from_u64(1);
    let one_mont = Fp128::to_montgomery(one_normal);
    println!("1 in Montgomery form: {:?}", one_mont);
    println!("Fp128::one(): {:?}", Fp128::one());
    
    // Test from_u64 conversion
    println!("\n\nTesting from_u64:");
    let five = Fp128::from_u64(5);
    println!("Fp128::from_u64(5) internal: {:?}", five);
    
    // Convert back from Montgomery
    let five_normal = five.from_montgomery();
    println!("5 from Montgomery: {:?}", five_normal);
    
    // Manual calculation of what 5 should be in Montgomery form
    // 5 * R mod p
    println!("\n\nManual verification:");
    println!("5 * R mod p should give us the Montgomery form of 5");
    
    // Test basic arithmetic
    println!("\n\nTesting arithmetic:");
    let a = Fp128::from_u64(5);
    let b = Fp128::from_u64(7);
    let c = a + b;
    
    println!("a = 5: {:?}", a);
    println!("b = 7: {:?}", b);
    println!("a + b: {:?}", c);
    
    let c_normal = c.from_montgomery();
    println!("(a + b) from Montgomery: {:?}", c_normal);
    
    // Also test the expected value
    let expected = Fp128::from_u64(12);
    println!("Expected (12): {:?}", expected);
    let expected_normal = expected.from_montgomery();
    println!("12 from Montgomery: {:?}", expected_normal);
}