fn main() {
    // For Fp128: p = 2^128 - 2^108 + 1
    let p_high = 0xfffff00000000000u64;
    let p_low = 0x0000000000000001u64;
    
    println!("Field Fp128:");
    println!("p = 2^128 - 2^108 + 1");
    println!("p = 0x{:016x}{:016x}", p_high, p_low);
    
    // R should be 2^128 mod p
    // 2^128 mod p = 2^128 - p = 2^108 - 1
    let r_value = (1u128 << 108) - 1;
    println!("\nR = 2^128 mod p = 2^108 - 1");
    println!("R = 0x{:032x}", r_value);
    println!("R = 0x00000fffffffffffffffffffffffffff");
    
    // But we have R stored as 2 limbs
    let r_low = 0x00000fffffffffff_u64;
    let r_high = 0x0000000000000000_u64;
    println!("\nAs 2 limbs: [0x{:016x}, 0x{:016x}]", r_low, r_high);
    
    // So R is correct. Now what should from_montgomery(R) return?
    // from_montgomery(R) = R * R^(-1) mod p = 1
    println!("\nfrom_montgomery(R) should return 1");
    
    // But we're getting 0x00000fffff000000ffffe00001000000
    // Let's check what this value is
    let result_low = 0xffffe00001000000u64;
    let result_high = 0x00000fffff000000u64;
    let result_u128 = (result_high as u128) << 64 | (result_low as u128);
    println!("\nActual result: 0x{:032x}", result_u128);
    
    // This doesn't look right. Let me think about the Montgomery reduction algorithm...
}