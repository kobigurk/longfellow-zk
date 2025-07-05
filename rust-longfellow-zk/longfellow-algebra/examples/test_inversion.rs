use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing Fp128 inversion\n");
    
    // Test inversion of 3
    let three = Fp128::from_u64(3);
    let three_inv = three.invert();
    
    if let Some(inv) = three_inv {
        println!("3^(-1) exists");
        
        let product = three * inv;
        let one = Fp128::one();
        
        println!("3 * 3^(-1) = {:?}", product);
        println!("Expected 1 = {:?}", one);
        
        if product == one {
            println!("✅ 3 * 3^(-1) = 1");
        } else {
            println!("❌ 3 * 3^(-1) != 1");
            
            // Let's check the bytes
            let prod_bytes = product.to_bytes_le();
            println!("\nProduct bytes: {:?}", prod_bytes);
            println!("Expected: [1]");
        }
    } else {
        println!("❌ 3^(-1) doesn't exist (this shouldn't happen)");
    }
    
    // Test a few more inversions
    println!("\n\nTesting more inversions:");
    
    for i in 1u64..10 {
        let a = Fp128::from_u64(i);
        if let Some(a_inv) = a.invert() {
            let prod = a * a_inv;
            if prod == Fp128::one() {
                println!("✅ {} * {}^(-1) = 1", i, i);
            } else {
                println!("❌ {} * {}^(-1) != 1", i, i);
            }
        } else {
            println!("❌ {}^(-1) doesn't exist", i);
        }
    }
}