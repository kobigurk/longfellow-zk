use longfellow_algebra::field::fp128::Fp128;
use longfellow_algebra::traits::Field;

fn main() {
    println!("Testing Fp128 Montgomery form...\n");
    
    // Create values using from_u64
    let a = Fp128::from_u64(5);
    let b = Fp128::from_u64(7);
    let c = Fp128::from_u64(12);
    
    println!("Created values:");
    println!("a = Fp128::from_u64(5) = {:?}", a);
    println!("b = Fp128::from_u64(7) = {:?}", b);
    println!("c = Fp128::from_u64(12) = {:?}", c);
    
    // Test arithmetic
    let sum = a + b;
    println!("\nArithmetic tests:");
    println!("a + b = {:?}", sum);
    println!("Expected c = {:?}", c);
    println!("Are they equal? {}", sum == c);
    
    // Check the internal representation by converting to bytes
    println!("\nByte representations:");
    println!("a bytes: {:?}", a.to_bytes_le());
    println!("b bytes: {:?}", b.to_bytes_le());
    println!("c bytes: {:?}", c.to_bytes_le());
    println!("sum bytes: {:?}", sum.to_bytes_le());
    
    // Test multiplication
    let prod = a * b;
    let expected_prod = Fp128::from_u64(35);
    println!("\nMultiplication test:");
    println!("a * b = {:?}", prod);
    println!("Expected: {:?}", expected_prod);
    println!("Are they equal? {}", prod == expected_prod);
}