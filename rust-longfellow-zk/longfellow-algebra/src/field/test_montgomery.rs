use super::fp128::Fp128;
use crate::traits::Field;

#[test]
fn test_montgomery_representation() {
    // Create values
    let a = Fp128::from_u64(5);
    let b = Fp128::from_u64(7);
    let c = Fp128::from_u64(12);
    
    // Debug prints to see what's happening
    println!("\nDebug representation (should show actual values):");
    println!("a = {:?}", a);
    println!("b = {:?}", b);
    println!("c = {:?}", c);
    
    // Test addition
    let sum = a + b;
    println!("\nAddition test:");
    println!("a + b = {:?}", sum);
    println!("Expected c = {:?}", c);
    println!("Are they equal? {}", sum == c);
    
    // Check bytes (this converts from Montgomery first)
    println!("\nByte representations:");
    println!("a bytes: {:?}", a.to_bytes_le());
    println!("b bytes: {:?}", b.to_bytes_le());
    println!("c bytes: {:?}", c.to_bytes_le());
    println!("sum bytes: {:?}", sum.to_bytes_le());
    
    // More arithmetic tests
    let diff = b - a;
    println!("\nSubtraction test:");
    println!("b - a = {:?}", diff);
    println!("Expected 2 = {:?}", Fp128::from_u64(2));
    
    let prod = a * b;
    println!("\nMultiplication test:");
    println!("a * b = {:?}", prod);
    println!("Expected 35 = {:?}", Fp128::from_u64(35));
    
    // All assertions
    assert_eq!(sum, c, "5 + 7 should equal 12");
    assert_eq!(diff, Fp128::from_u64(2), "7 - 5 should equal 2");
    assert_eq!(prod, Fp128::from_u64(35), "5 * 7 should equal 35");
}