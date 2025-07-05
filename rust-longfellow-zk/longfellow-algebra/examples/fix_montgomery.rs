use longfellow_algebra::Fp128;

fn main() {
    println!("Understanding the Montgomery reduction bug\n");
    
    // The pattern we see:
    // Expected: x
    // Got: x + modulus_high_limb
    
    // For example:
    // 1 * 1 should give 1, but gives 0xfffff00000000001
    // 3 * 4 should give 12, but gives 0xfffff0000000000c
    
    // This means the Montgomery reduction is producing results that are
    // exactly p (the modulus) more than they should be.
    
    // In Montgomery arithmetic:
    // - Values are stored as aR mod p
    // - Multiplication of (aR) * (bR) gives abR²
    // - Montgomery reduction computes abR² * R⁻¹ = abR mod p
    
    // The fact that we're getting results that are p more than expected
    // suggests the reduction is producing abR + p instead of abR mod p
    
    // This could happen if:
    // 1. The reduction algorithm has an off-by-one error
    // 2. The final conditional subtraction is not triggered when it should be
    // 3. The comparison with modulus is wrong
    
    // Looking at the code in montgomery_reduce_wide:
    // - We do the reduction steps
    // - Shift right by M limbs
    // - Then do: if result >= modulus, subtract modulus
    
    // But our results suggest this final subtraction is not happening
    // even though result >= modulus
    
    println!("Let's check if the issue is in the comparison or subtraction:");
    
    let one = Fp128::from_u64(1);
    let two = Fp128::from_u64(2);
    let product = one * two; // Should be 2
    
    println!("1 * 2 = {:?}", product);
    println!("Expected: {:?}", two);
    
    if product == two {
        println!("✅ Correct");
    } else {
        println!("❌ Wrong - has extra 0xfffff00000000000");
    }
    
    // The fix should be in montgomery_reduce_wide
    // We need to ensure the conditional subtraction works correctly
}