fn main() {
    // Simple Montgomery reduction test
    // p = 2^128 - 2^108 + 1
    let p = (1u128 << 128) - (1u128 << 108) + 1;
    
    // R = 2^128 mod p = 2^108 - 1
    let r = (1u128 << 108) - 1;
    
    println!("p = 0x{:032x}", p);
    println!("R = 0x{:032x}", r);
    
    // Check: R < p
    println!("\nR < p: {}", r < p);
    
    // Compute R * R^(-1) mod p = 1
    // For Montgomery reduction, we need -p^(-1) mod 2^64
    // Since p = ...01, we have p^(-1) = 1 mod 2^64
    // So -p^(-1) = -1 = 2^64 - 1 mod 2^64
    
    // Let's verify our from_montgomery logic differently
    // If we have a value 'a' in Montgomery form (i.e., a*R mod p)
    // Then from_montgomery should give us 'a'
    
    // Test: from_montgomery(R) should give 1
    // because R = 1*R mod p
    
    println!("\nTesting from_montgomery(R) = 1:");
    
    // Actually, let me compute what our algorithm is producing
    // and see if it's correct modulo p
    let result = 0x00000fffff000000ffffe00001000000u128;
    println!("Our result: 0x{:032x}", result);
    
    // Is this 1 mod p?
    let remainder = result % p;
    println!("result mod p = {}", remainder);
    
    // Hmm, it should be 1. Let me check the computation...
}