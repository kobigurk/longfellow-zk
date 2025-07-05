use longfellow_algebra::field::fp128::Fp128;
use longfellow_algebra::traits::Field;

fn main() {
    println!("=== Understanding Fp128 Montgomery Form ===\n");
    
    // The modulus p = 2^128 - 2^108 + 1
    println!("Modulus p = 2^128 - 2^108 + 1");
    println!("p = 0xFFFFF00000000000000000000000001");
    
    // R = 2^128 mod p = 0x00000FFFFFFFFFFF
    println!("\nMontgomery constant R = 2^128 mod p");
    println!("R = 0x00000FFFFFFFFFFF");
    
    // For an element 'a', its Montgomery form is (a * R) mod p
    // So 1 in Montgomery form is R itself
    println!("\n1 in Montgomery form = 1 * R mod p = R");
    println!("Expected: 0x00000FFFFFFFFFFF");
    println!("Actual: {:?}", Fp128::one());
    
    // The issue is that Fp128::ONE is set to R directly, which is correct
    // But there seems to be a problem with the arithmetic operations
    
    println!("\n=== Testing Arithmetic ===");
    
    // When we do from_u64(5), it should compute 5 * R mod p
    let five = Fp128::from_u64(5);
    println!("\n5 in Montgomery form = 5 * R mod p");
    println!("5 in Montgomery: {:?}", five);
    
    // To verify, let's manually compute what 5 * R mod p should be
    // 5 * 0x00000FFFFFFFFFFF = 0x00004FFFFFFFF7FB
    // This is less than p, so it's the result
    println!("Expected 5 * R = 0x00004FFFFFFFF7FB");
    
    // The actual value we're getting is wrong, suggesting the to_montgomery 
    // function has issues
    
    // Test addition
    let a = Fp128::from_u64(5);
    let b = Fp128::from_u64(7);
    let c = a + b;
    let expected = Fp128::from_u64(12);
    
    println!("\n=== Addition Test ===");
    println!("a = 5: {:?}", a);
    println!("b = 7: {:?}", b);
    println!("a + b: {:?}", c);
    println!("Expected 12: {:?}", expected);
    println!("Are they equal? {}", c == expected);
    
    // The problem is likely in the to_montgomery conversion
    // Let's check the raw values
    println!("\n=== Raw Values (as bytes) ===");
    println!("5: {:02x?}", a.to_bytes_le());
    println!("7: {:02x?}", b.to_bytes_le());
    println!("5+7: {:02x?}", c.to_bytes_le());
    println!("12: {:02x?}", expected.to_bytes_le());
}