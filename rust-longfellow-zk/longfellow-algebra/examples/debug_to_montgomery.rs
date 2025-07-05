use longfellow_algebra::Fp128;
use longfellow_algebra::field::fp_generic::FieldReduction;
use longfellow_algebra::field::fp128::Fp128Reduce;

fn main() {
    println!("Debugging to_montgomery function\n");
    
    // The issue: to_montgomery should convert a regular value to Montgomery form
    // For a regular value a, Montgomery form should be a * R mod p
    // But to_montgomery is multiplying by R^2, which assumes input is already in Montgomery form
    
    // Let's test the R and R^2 constants
    println!("Testing R and R^2 constants:");
    
    // R should be 2^128 mod p = 0x00000FFFFFFFFFFF0000000000000000
    println!("R = {:?}", Fp128Reduce::R);
    
    // R^2 should be (2^128)^2 mod p = 2^256 mod p
    println!("R^2 = {:?}", Fp128Reduce::R2);
    
    // Test: if we have value 1 in regular form, converting to Montgomery should give R
    let one_regular = longfellow_algebra::nat::Nat::<2>::from_u64(1);
    println!("\nTesting conversion of 1:");
    println!("1 regular = {:?}", one_regular);
    
    let one_montgomery = Fp128::to_montgomery(one_regular);
    println!("1 in Montgomery = {:?}", one_montgomery);
    
    // The Montgomery form of 1 should be 1 * R mod p = R
    println!("Expected: R = {:?}", Fp128Reduce::R);
    
    // Test the reverse
    let back_to_regular = one_montgomery.from_montgomery();
    println!("Back to regular = {:?}", back_to_regular);
    
    // This should be 1
    let expected_one = longfellow_algebra::nat::Nat::<2>::from_u64(1);
    if back_to_regular == expected_one {
        println!("✓ Round-trip conversion works for 1");
    } else {
        println!("✗ Round-trip conversion failed for 1");
    }
    
    // Test with 5
    println!("\nTesting conversion of 5:");
    let five_regular = longfellow_algebra::nat::Nat::<2>::from_u64(5);
    println!("5 regular = {:?}", five_regular);
    
    let five_montgomery = Fp128::to_montgomery(five_regular);
    println!("5 in Montgomery = {:?}", five_montgomery);
    
    let five_back = five_montgomery.from_montgomery();
    println!("5 back to regular = {:?}", five_back);
    
    if five_back == five_regular {
        println!("✓ Round-trip conversion works for 5");
    } else {
        println!("✗ Round-trip conversion failed for 5");
    }
}