use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging simple Montgomery operations\n");
    
    // The core issue: when we create Fp128::from_u64(5), it should create 5*R mod p
    // But from_montgomery() should then return 5
    // Currently both give the same value, suggesting multiplication isn't working
    
    // Test basic multiplication
    let a = Fp128::from_u64(2);
    let b = Fp128::from_u64(3);
    let c = a * b;
    let expected = Fp128::from_u64(6);
    
    println!("Testing basic multiplication:");
    println!("a = 2 = {:?}", a);
    println!("b = 3 = {:?}", b);
    println!("c = a * b = {:?}", c);
    println!("expected = 6 = {:?}", expected);
    
    if c == expected {
        println!("✓ Basic multiplication works");
    } else {
        println!("✗ Basic multiplication is broken");
    }
    
    // Now test if Montgomery form is actually different
    // The key insight: if Montgomery form was working, these values should be different
    // from their regular counterparts, but still multiply correctly
    
    println!("\nTesting if values are actually in Montgomery form:");
    
    // If 2 is in Montgomery form, it should be 2*R mod p, which is NOT just 2
    let two_montgomery = Fp128::from_u64(2);
    let two_regular = two_montgomery.from_montgomery();
    
    println!("from_u64(2) = {:?}", two_montgomery);
    println!("from_montgomery() = {:?}", two_regular);
    
    // If Montgomery is working, the conversion should give us the regular value 2
    let expected_two = longfellow_algebra::nat::Nat::<2>::from_u64(2);
    
    if two_regular == expected_two {
        println!("✓ from_montgomery gives correct regular value");
    } else {
        println!("✗ from_montgomery doesn't give correct regular value");
        println!("Expected: {:?}", expected_two);
        println!("Got: {:?}", two_regular);
    }
    
    // The smoking gun test: create ONE using the Montgomery constant
    println!("\nTesting ONE constant:");
    let one = Fp128::one();
    println!("Fp128::one() = {:?}", one);
    
    let one_regular = one.from_montgomery();
    println!("one.from_montgomery() = {:?}", one_regular);
    
    let expected_one = longfellow_algebra::nat::Nat::<2>::from_u64(1);
    if one_regular == expected_one {
        println!("✓ ONE converts correctly");
    } else {
        println!("✗ ONE doesn't convert correctly");
        println!("Expected: {:?}", expected_one);
        println!("Got: {:?}", one_regular);
    }
}