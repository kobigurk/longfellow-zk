use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing Fp128 arithmetic after Montgomery fixes...\n");
    
    // Test that 5 + 7 = 12
    let a = Fp128::from_u64(5);
    let b = Fp128::from_u64(7);
    let c = Fp128::from_u64(12);
    
    println!("Test 1: Addition");
    println!("a = Fp128::from_u64(5)");
    println!("b = Fp128::from_u64(7)");
    println!("c = Fp128::from_u64(12)");
    
    let sum = a + b;
    println!("a + b = {:?}", sum);
    println!("c     = {:?}", c);
    
    if sum == c {
        println!("✅ SUCCESS: 5 + 7 = 12\n");
    } else {
        println!("❌ FAILED: 5 + 7 != 12\n");
    }
    
    // Test that 12 - 7 = 5
    println!("Test 2: Subtraction");
    let diff = c - b;
    println!("c - b = {:?}", diff);
    println!("a     = {:?}", a);
    if diff == a {
        println!("✅ SUCCESS: 12 - 7 = 5\n");
    } else {
        println!("❌ FAILED: 12 - 7 != 5\n");
    }
    
    // Test that 3 * 4 = 12
    println!("Test 3: Multiplication");
    let x = Fp128::from_u64(3);
    let y = Fp128::from_u64(4);
    let prod = x * y;
    
    println!("3 * 4 = {:?}", prod);
    println!("12    = {:?}", c);
    
    if prod == c {
        println!("✅ SUCCESS: 3 * 4 = 12\n");
    } else {
        println!("❌ FAILED: 3 * 4 != 12\n");
    }
    
    // Test constraint w[0] + w[1] - w[2] = 0
    println!("Test 4: Constraint satisfaction");
    let w0 = Fp128::from_u64(5);
    let w1 = Fp128::from_u64(7);
    let w2 = Fp128::from_u64(12);
    
    let result = w0 + w1 - w2;
    println!("5 + 7 - 12 = {:?}", result);
    println!("zero       = {:?}", Fp128::zero());
    
    if result == Fp128::zero() {
        println!("✅ SUCCESS: 5 + 7 - 12 = 0\n");
    } else {
        println!("❌ FAILED: 5 + 7 - 12 != 0\n");
    }
    
    // Test one and zero
    println!("Test 5: Identity elements");
    let one = Fp128::one();
    let zero = Fp128::zero();
    
    println!("one  = {:?}", one);
    println!("zero = {:?}", zero);
    
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
    
    // Summary
    println!("\n=== SUMMARY ===");
    println!("If all tests passed, Fp128 Montgomery arithmetic is fixed!");
}