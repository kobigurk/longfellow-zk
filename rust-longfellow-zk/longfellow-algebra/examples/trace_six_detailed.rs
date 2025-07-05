use longfellow_algebra::{Fp128, Field};
use longfellow_algebra::nat::Nat;

fn main() {
    println!("Detailed trace of 6 * 6^(-1)\n");
    
    // Get the values
    let six = Fp128::from_u64(6);
    let six_inv = six.invert().unwrap();
    
    println!("6 = {:?}", six);
    println!("6^(-1) = {:?}", six_inv);
    
    // We know from Python that:
    // 6R mod p = 0x00005ffffffffffffffffffffffffffa
    // 6^(-1)R mod p = 0x7ffffaaaaaaaaaaaaaaaaaaaaaaaaaab
    // (6R) * (6^(-1)R) mod p = R^2
    
    // After Montgomery reduction, R^2 -> R (which represents 1)
    // But we're getting 0
    
    // Let's manually compute what we expect
    let r = Fp128::R;
    let r2 = Fp128::R2;
    
    println!("\nR = {:?}", r);
    println!("R^2 = {:?}", r2);
    
    // The multiplication 6 * 6^(-1) should give us 1
    let product = six * six_inv;
    println!("\n6 * 6^(-1) = {:?}", product);
    
    let one = Fp128::one();
    let zero = Fp128::zero();
    
    if product == one {
        println!("✅ Product equals 1");
    } else if product == zero {
        println!("❌ Product equals 0");
        
        // This suggests that somewhere in the Montgomery multiplication,
        // we're reducing R to 0 instead of keeping it as R
        
        // The fact that 6 * 6^(-1) = 0 suggests that
        // the Montgomery reduction is reducing R to 0
        
        println!("\nThis suggests R is being reduced to 0 somewhere");
    } else {
        println!("❌ Product is something else");
    }
    
    // Let's check what ONE really is
    println!("\n\nChecking ONE constant:");
    println!("ONE = {:?}", one);
    println!("ONE should internally be R");
    
    // And check if multiplying by one preserves values
    let six_times_one = six * one;
    if six_times_one == six {
        println!("✅ 6 * 1 = 6");
    } else {
        println!("❌ 6 * 1 != 6");
    }
}