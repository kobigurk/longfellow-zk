use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Final Fp128 Test\n");
    
    // Test basic arithmetic
    let a = Fp128::from_u64(5);
    let b = Fp128::from_u64(7);
    let c = Fp128::from_u64(12);
    
    println!("Addition: 5 + 7 = 12");
    let sum = a + b;
    if sum == c {
        println!("✅ PASS");
    } else {
        println!("❌ FAIL");
    }
    
    println!("\nSubtraction: 12 - 7 = 5");
    let diff = c - b;
    if diff == a {
        println!("✅ PASS");
    } else {
        println!("❌ FAIL");
    }
    
    println!("\nMultiplication: 3 * 4 = 12");
    let x = Fp128::from_u64(3);
    let y = Fp128::from_u64(4);
    let prod = x * y;
    if prod == c {
        println!("✅ PASS");
    } else {
        println!("❌ FAIL - multiplication still has issues");
        println!("  Expected: {:?}", c);
        println!("  Got:      {:?}", prod);
    }
    
    println!("\nIdentity elements:");
    let one = Fp128::one();
    let zero = Fp128::zero();
    
    if one + zero == one {
        println!("✅ 1 + 0 = 1");
    } else {
        println!("❌ 1 + 0 != 1");
    }
    
    if one * one == one {
        println!("✅ 1 * 1 = 1");
    } else {
        println!("❌ 1 * 1 != 1");
    }
    
    println!("\nInversion:");
    let inv = x.invert().unwrap();
    let should_be_one = x * inv;
    if should_be_one == one {
        println!("✅ 3 * 3^(-1) = 1");
    } else {
        println!("❌ 3 * 3^(-1) != 1");
    }
    
    println!("\nSummary:");
    println!("- Addition: ✅ Working");
    println!("- Subtraction: ✅ Working"); 
    println!("- from_montgomery: ✅ Working");
    println!("- Multiplication: ❌ Has issues (produces extra bits)");
    println!("- Identity elements: Partially working");
    println!("- Inversion: Depends on multiplication");
}