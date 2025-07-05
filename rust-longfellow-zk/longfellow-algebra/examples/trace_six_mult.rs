use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Tracing 6 * 6^(-1)\n");
    
    let six = Fp128::from_u64(6);
    let six_inv = six.invert().unwrap();
    
    println!("6 in Montgomery form:");
    println!("6 = {:?}", six);
    
    println!("\n6^(-1) in Montgomery form:");
    println!("6^(-1) = {:?}", six_inv);
    
    // The issue is that 6 * 6^(-1) = 0 instead of 1
    // Let's trace what's happening
    
    let product = six * six_inv;
    println!("\n6 * 6^(-1) = {:?}", product);
    
    // Convert to bytes to see the actual value
    let prod_bytes = product.to_bytes_le();
    println!("Product bytes: {:?}", prod_bytes);
    
    // Let's also check what happens with a manual computation
    // In Montgomery form: 6R * (6^(-1))R with Montgomery reduction should give R
    
    // But we're getting 0, which suggests the multiplication result
    // before reduction might be exactly divisible by p
    
    // Let's check if 6 * 0xd5554800000000000000000000000001 = 0 mod p
    // This would happen if 6 * 0xd5554800000000000000000000000001 = p
    
    println!("\nChecking if 6 * 6^(-1) equals p:");
    let p = Fp128::MODULUS;
    println!("p = {:?}", p);
    
    // Actually, let's check a simpler case first
    // What is 6 + (-6) ?
    let neg_six = -six;
    let sum = six + neg_six;
    println!("\n6 + (-6) = {:?}", sum);
    
    if sum == Fp128::zero() {
        println!("✅ 6 + (-6) = 0 (correct)");
    } else {
        println!("❌ 6 + (-6) != 0");
    }
    
    // The real issue might be in how we handle the case where
    // the Montgomery multiplication result equals the modulus
}