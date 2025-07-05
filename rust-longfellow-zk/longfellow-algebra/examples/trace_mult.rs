use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Tracing multiplication: 3 * 4 = 12\n");
    
    // In regular form: 3, 4, 12
    // In Montgomery form: 3R, 4R, 12R
    
    // When we compute (3R) * (4R) in Montgomery multiplication:
    // We get (3R * 4R) * R^(-1) = 12R
    
    // Let's trace this
    let three = Fp128::from_u64(3);
    let four = Fp128::from_u64(4);
    
    println!("Step 1: Convert to Montgomery form");
    println!("3 -> 3R mod p");
    println!("4 -> 4R mod p");
    
    // The issue might be in to_montgomery
    // Let's manually compute what 3R should be
    
    // R = 2^108 - 1
    // 3R = 3 * (2^108 - 1) = 3 * 2^108 - 3
    
    println!("\nStep 2: Multiplication in Montgomery form");
    println!("(3R) * (4R) with Montgomery reduction = 12R");
    
    let product = three * four;
    println!("\nResult: {:?}", product);
    
    // Check the raw bytes
    let prod_bytes = product.to_bytes_le();
    println!("Product bytes: {:?}", prod_bytes);
    
    // The bytes [12, 0, 0, 0, 0, 240, 255, 255] represent:
    // 12 + 0xfffff00000000000 >> 32
    // = 12 + something
    
    // Let's check if this is actually 12 * something
    let twelve = Fp128::from_u64(12);
    println!("\nExpected (12): {:?}", twelve);
    println!("Expected bytes: {:?}", twelve.to_bytes_le());
    
    // Let's also manually check the to_montgomery function
    println!("\n\nChecking to_montgomery:");
    
    // to_montgomery(x) = x * R^2 * R^(-1) mod p = x * R mod p
    // For x = 1, we should get R
    
    let one_nat = longfellow_algebra::nat::Nat::<2>::from_u64(1);
    let one_mont = Fp128::to_montgomery(one_nat);
    println!("to_montgomery(1) = {:?}", one_mont);
    
    // This should be R
    let r = Fp128::R;
    println!("R = {:?}", r);
    
    // But wait, from_u64 also calls to_montgomery
    // So from_u64(3) is already in Montgomery form
    // The issue might be elsewhere
}