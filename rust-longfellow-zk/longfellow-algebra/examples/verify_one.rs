use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Verifying Montgomery form ONE...\n");
    
    // Get ONE and convert from Montgomery
    let one = Fp128::one();
    let one_bytes = one.to_bytes_le();
    println!("Fp128::one().to_bytes_le() = {:?}", one_bytes);
    
    // The bytes we're getting
    let result_bytes = vec![0, 0, 0, 1, 0, 224, 255, 255, 0, 0, 0, 255, 255, 15, 0, 0];
    
    // Convert to u128 to check value
    let mut value = 0u128;
    for (i, &byte) in result_bytes.iter().enumerate() {
        if i < 16 {
            value |= (byte as u128) << (i * 8);
        }
    }
    
    println!("\nAs u128: 0x{:032x}", value);
    
    // This is 0x00000fffff000000ffffe00001000000
    // Let's check if this equals 1 modulo p
    
    // Actually, let me think differently. Maybe the issue is with how
    // we're interpreting the bytes. The to_bytes_le() function calls
    // from_montgomery() and then converts to bytes.
    
    // If from_montgomery is producing the wrong value, then all our
    // arithmetic will be wrong.
    
    // Let's test a different approach - create 1 directly and see
    // what its Montgomery form should be
    println!("\nTesting to_montgomery(1):");
    let one_nat = longfellow_algebra::nat::Nat::<2>::from_u64(1);
    let one_mont = Fp128::to_montgomery(one_nat);
    println!("to_montgomery(Nat(1)) = {:?}", one_mont);
    
    // This should give us R
    println!("Expected: R = {:?}", Fp128::R);
}