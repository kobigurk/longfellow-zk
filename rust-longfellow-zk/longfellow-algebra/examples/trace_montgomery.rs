use longfellow_algebra::field::fp128::Fp128;
use longfellow_algebra::traits::Field;

fn main() {
    println!("=== Tracing Montgomery Conversion ===\n");
    
    // Let's trace what happens when we create Fp128::from_u64(1)
    println!("Creating Fp128::from_u64(1):");
    
    // from_u64 calls to_montgomery(Nat::from_u64(1))
    // Nat::from_u64(1) creates Nat { limbs: [1, 0] }
    
    // to_montgomery multiplies by R2 and reduces
    // So we compute (1 * R2) mod p, then reduce by R to get R
    
    let one = Fp128::from_u64(1);
    println!("Result: {:?}", one);
    println!("As bytes: {:02x?}", one.to_bytes_le());
    
    // The expected result should be R = 0x00000FFFFFFFFFFF
    // In little-endian bytes: [FF, FF, FF, FF, FF, 0F, 00, 00, 00, 00, 00, 00, 00, 00]
    println!("\nExpected R in bytes: [ff, ff, ff, ff, ff, 0f, 00, 00, 00, 00, 00, 00, 00, 00]");
    
    // But we're getting: [00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, ff, ff, 0f]
    // This is 0x0FFFFFFFFFFF00000000000000000000 in little-endian
    // Or 0x00000000000000000FFFFFFFFFFF0000 in big-endian
    
    println!("\n=== The Problem ===");
    println!("The to_bytes_le() function is not working correctly!");
    println!("It seems to be outputting the bytes in the wrong order or the conversion from Montgomery is wrong.");
    
    // Let's check Fp128::ONE directly
    println!("\n=== Checking Fp128::ONE constant ===");
    println!("Fp128::ONE: {:?}", Fp128::ONE);
    println!("Fp128::ONE as bytes: {:02x?}", Fp128::ONE.to_bytes_le());
    
    // And compare with one()
    println!("\nFp128::one(): {:?}", Fp128::one());
    println!("Fp128::one() as bytes: {:02x?}", Fp128::one().to_bytes_le());
    
    // Test the from_montgomery conversion
    println!("\n=== Testing from_montgomery ===");
    let one_mont = Fp128::ONE;
    let one_normal = one_mont.from_montgomery();
    println!("ONE in Montgomery: {:?}", one_mont);
    println!("ONE from Montgomery: {:?}", one_normal);
    println!("Should be Nat(0x00000000000000000000000000000001)");
}