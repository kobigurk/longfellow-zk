use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing Fp128 arithmetic...");
    
    // Test that 5 + 7 = 12
    let a = Fp128::from_u64(5);
    let b = Fp128::from_u64(7);
    let c = Fp128::from_u64(12);
    
    println!("a = {:?}", a);
    println!("b = {:?}", b);
    println!("c = {:?}", c);
    
    let sum = a + b;
    println!("a + b = {:?}", sum);
    println!("Expected: {:?}", c);
    
    if sum == c {
        println!("✅ SUCCESS: 5 + 7 = 12");
    } else {
        println!("❌ FAILED: 5 + 7 != 12");
    }
    
    // Test that 12 - 7 = 5
    let diff = c - b;
    if diff == a {
        println!("✅ SUCCESS: 12 - 7 = 5");
    } else {
        println!("❌ FAILED: 12 - 7 != 5");
    }
    
    // Test that 3 * 4 = 12
    let x = Fp128::from_u64(3);
    let y = Fp128::from_u64(4);
    let prod = x * y;
    
    if prod == c {
        println!("✅ SUCCESS: 3 * 4 = 12");
    } else {
        println!("❌ FAILED: 3 * 4 != 12");
        println!("  Got: {:?}", prod);
    }
    
    // Test constraint w[0] + w[1] - w[2] = 0
    let w0 = Fp128::from_u64(5);
    let w1 = Fp128::from_u64(7);
    let w2 = Fp128::from_u64(12);
    
    let result = w0 + w1 - w2;
    if result == Fp128::zero() {
        println!("✅ SUCCESS: 5 + 7 - 12 = 0");
    } else {
        println!("❌ FAILED: 5 + 7 - 12 != 0");
        println!("  Got: {:?}", result);
    }
    
    // Test one and zero
    let one = Fp128::one();
    let zero = Fp128::zero();
    
    if one + zero == one {
        println!("✅ SUCCESS: 1 + 0 = 1");
    } else {
        println!("❌ FAILED: 1 + 0 != 1");
    }
    
    if one * one == one {
        println!("✅ SUCCESS: 1 * 1 = 1");
    } else {
        println!("❌ FAILED: 1 * 1 != 1");
    }
}