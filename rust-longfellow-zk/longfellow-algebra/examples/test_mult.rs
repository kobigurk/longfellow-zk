use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing Fp128 multiplication in detail...\n");
    
    // Test 3 * 4 = 12
    let a = Fp128::from_u64(3);
    let b = Fp128::from_u64(4);
    let c = Fp128::from_u64(12);
    
    println!("a = Fp128::from_u64(3)");
    println!("b = Fp128::from_u64(4)");
    println!("c = Fp128::from_u64(12)");
    
    let prod = a * b;
    
    println!("\na * b = {:?}", prod);
    println!("Expected: {:?}", c);
    
    let prod_bytes = prod.to_bytes_le();
    let c_bytes = c.to_bytes_le();
    
    println!("\n(a * b).to_bytes_le() = {:?}", prod_bytes);
    println!("12.to_bytes_le()      = {:?}", c_bytes);
    
    // The issue is that we're getting [12, 0, 0, 0, 0, 240, 255, 255]
    // This looks like 12 + something * 2^40
    
    // Let's check if the values are equal in Montgomery form
    if prod == c {
        println!("\n✅ In Montgomery form, 3 * 4 == 12");
    } else {
        println!("\n❌ In Montgomery form, 3 * 4 != 12");
        
        // Debug the internal representations
        println!("\nInternal representations:");
        println!("prod internal = {:?}", prod);
        println!("c internal    = {:?}", c);
    }
    
    // Let's also check what happens with 1 * 1
    println!("\n\nTesting 1 * 1:");
    let one = Fp128::one();
    let one_squared = one * one;
    
    println!("one = {:?}", one);
    println!("one * one = {:?}", one_squared);
    
    if one_squared == one {
        println!("✅ 1 * 1 = 1");
    } else {
        println!("❌ 1 * 1 != 1");
    }
}