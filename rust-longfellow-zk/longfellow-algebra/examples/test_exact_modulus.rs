use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing multiplication that gives exactly the modulus\n");
    
    // In a field, if a * b = p (the modulus), then a * b ≡ 0 (mod p)
    // This is correct behavior
    
    // The issue with 6 * 6^(-1) giving 0 suggests that
    // the multiplication is producing p instead of 1
    
    // Let's verify this hypothesis
    let six = Fp128::from_u64(6);
    let six_inv = six.invert().unwrap();
    
    // Check what 6^(-1) actually is
    let six_inv_bytes = six_inv.to_bytes_le();
    println!("6^(-1) as bytes: {:?}", six_inv_bytes);
    
    // From Python, we know 6^(-1) = 0xd5554800000000000000000000000001
    // Let's check if this is what we got
    
    // Actually, let's approach this differently
    // Let's create a value that when multiplied by 6 gives p
    
    // p / 6 = (2^128 - 2^108 + 1) / 6
    // Since p ≡ 1 (mod 6), we have p = 6k + 1 for some k
    // So k = (p - 1) / 6
    
    // Let's check if our inverse is actually (p - 1) / 6 instead of the correct inverse
    
    // First, let's verify that 6 * (correct_inverse) = 1
    let correct_inv_bytes = vec![
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x48, 0x55, 0xd5
    ];
    
    let correct_inv = Fp128::from_bytes_le(&correct_inv_bytes).unwrap();
    let prod1 = six * correct_inv;
    
    println!("6 * correct_inverse = {:?}", prod1);
    if prod1 == Fp128::one() {
        println!("✅ Correct inverse works");
    } else {
        println!("❌ Even correct inverse doesn't work!");
    }
    
    // Now let's check what we computed
    println!("\nOur computed inverse: {:?}", six_inv);
    println!("Correct inverse:      {:?}", correct_inv);
    
    if six_inv == correct_inv {
        println!("✅ We computed the correct inverse");
    } else {
        println!("❌ We computed the wrong inverse");
    }
}