use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing Fp128 arithmetic with direct value checks...\n");
    
    // Test that from_u64 and to_bytes_le work correctly
    println!("Test 1: from_u64 and to_bytes_le round trip");
    let a = Fp128::from_u64(5);
    let a_bytes = a.to_bytes_le();
    println!("Fp128::from_u64(5).to_bytes_le() = {:?}", a_bytes);
    
    // Should be [5, 0, 0, ..., 0] with trailing zeros removed
    if a_bytes == vec![5] {
        println!("✅ SUCCESS: from_u64(5) converts correctly\n");
    } else {
        println!("❌ FAILED: from_u64(5) doesn't convert correctly\n");
    }
    
    // Test that 5 + 7 = 12 by checking bytes
    println!("Test 2: Addition via bytes");
    let b = Fp128::from_u64(7);
    let c = Fp128::from_u64(12);
    let sum = a + b;
    let sum_bytes = sum.to_bytes_le();
    let c_bytes = c.to_bytes_le();
    
    println!("(5 + 7).to_bytes_le() = {:?}", sum_bytes);
    println!("12.to_bytes_le()      = {:?}", c_bytes);
    
    if sum_bytes == c_bytes {
        println!("✅ SUCCESS: 5 + 7 = 12\n");
    } else {
        println!("❌ FAILED: 5 + 7 != 12\n");
    }
    
    // Test multiplication
    println!("Test 3: Multiplication via bytes");
    let x = Fp128::from_u64(3);
    let y = Fp128::from_u64(4);
    let prod = x * y;
    let prod_bytes = prod.to_bytes_le();
    
    println!("(3 * 4).to_bytes_le() = {:?}", prod_bytes);
    println!("12.to_bytes_le()      = {:?}", c_bytes);
    
    if prod_bytes == c_bytes {
        println!("✅ SUCCESS: 3 * 4 = 12\n");
    } else {
        println!("❌ FAILED: 3 * 4 != 12\n");
    }
    
    // Test one() 
    println!("Test 4: Identity element one()");
    let one = Fp128::one();
    let one_bytes = one.to_bytes_le();
    println!("Fp128::one().to_bytes_le() = {:?}", one_bytes);
    
    if one_bytes == vec![1] {
        println!("✅ SUCCESS: one() is correct\n");
    } else {
        println!("❌ FAILED: one() is not 1\n");
    }
    
    // Test zero()
    println!("Test 5: Identity element zero()");
    let zero = Fp128::zero();
    let zero_bytes = zero.to_bytes_le();
    println!("Fp128::zero().to_bytes_le() = {:?}", zero_bytes);
    
    // Note: zero might be empty vec or vec![0]
    if zero_bytes.is_empty() || zero_bytes == vec![0] {
        println!("✅ SUCCESS: zero() is correct\n");
    } else {
        println!("❌ FAILED: zero() is not 0\n");
    }
    
    // Direct Montgomery form check
    println!("Test 6: Montgomery form internals");
    println!("R  = {:?}", Fp128::R);
    println!("R2 = {:?}", Fp128::R2);
    println!("MODULUS = {:?}", Fp128::MODULUS);
    println!("INV = 0x{:016x}", Fp128::INV);
}