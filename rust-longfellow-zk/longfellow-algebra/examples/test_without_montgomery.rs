use longfellow_algebra::nat::Nat;

fn main() {
    println!("Testing field arithmetic without Montgomery form...\n");
    
    // Define the modulus p = 2^128 - 2^108 + 1
    let modulus = Nat::<2>::new([0x0000000000000001, 0xfffff00000000000]);
    println!("Modulus p = {:?}", modulus);
    
    // Test simple addition: 5 + 7 = 12
    let mut a = Nat::<2>::from_u64(5);
    let b = Nat::<2>::from_u64(7);
    let c = Nat::<2>::from_u64(12);
    
    println!("\nTest: 5 + 7 = 12");
    println!("a = {:?}", a);
    println!("b = {:?}", b);
    
    let carry = a.add_with_carry(&b);
    println!("a + b = {:?} (carry={})", a, carry);
    println!("Expected: {:?}", c);
    
    if a == c {
        println!("✅ SUCCESS");
    } else {
        println!("❌ FAILED");
    }
    
    // The issue must be in the Montgomery form conversion
    // Let's manually check what happens when we convert 1 to Montgomery form
    println!("\n\nManual Montgomery conversion test:");
    println!("Converting 1 to Montgomery form (multiply by R mod p)");
    
    let one = Nat::<2>::from_u64(1);
    let r = Nat::<2>::new([0x00000fffffffffff, 0x0000000000000000]);
    
    println!("1 = {:?}", one);
    println!("R = {:?}", r);
    
    // 1 * R mod p should give R
    // Since R < p, the result is just R
    println!("1 * R mod p = R = {:?}", r);
    
    // Now let's test from_montgomery(R) which should give 1
    // This requires implementing modular arithmetic without Montgomery form
}