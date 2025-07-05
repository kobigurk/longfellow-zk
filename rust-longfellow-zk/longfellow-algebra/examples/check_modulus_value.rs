use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Checking modulus value\n");
    
    let modulus = Fp128::MODULUS;
    println!("MODULUS = {:?}", modulus);
    println!("MODULUS[0] = 0x{:016x}", modulus.limbs[0]);
    println!("MODULUS[1] = 0x{:016x}", modulus.limbs[1]);
    
    // The modulus should be p = 2^128 - 2^108 + 1
    // In limbs: [1, 0xfffff00000000000]
    
    // Let's check -1
    let minus_one = -Fp128::one();
    println!("\n-1 = {:?}", minus_one);
    
    // -1 should be p-1 in the field
    // Expected: [0, 0xfffff00000000000]
    
    // Actually wait, let me think about this more carefully
    // When we have a value that's exactly p-1, it's valid in the field
    // It represents -1
    
    // The issue is that when we multiply -1 * -1, we should get 1
    // But we're getting 0
    
    // This suggests that (p-1) * (p-1) after Montgomery reduction
    // is giving us 0 instead of 1
    
    // Let's trace this manually
    println!("\n\nTracing (-1) * (-1):");
    
    // In Montgomery form:
    // -1 is represented as (p-1)R mod p
    // But wait, what is (p-1)R mod p?
    
    // Since p-1 < p, we have (p-1)R mod p = (p-1)R - kp for some k
    // We need to find k such that (p-1)R - kp < p
    
    // Actually, let's just check what -1 is in our implementation
    let minus_one_internal = minus_one; // This calls Debug which converts from Montgomery
    println!("-1 internal = {:?}", minus_one_internal);
    
    // The debug output shows it's 0xfffff000000000000000000000000000
    // This is p-1, which is correct for -1
    
    // So the issue is that when we multiply (p-1) by (p-1) in Montgomery form,
    // we're not getting the right result
}