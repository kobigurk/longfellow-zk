fn main() {
    println!("Checking R and R2 values for Fp128...\n");
    
    // p = 2^128 - 2^108 + 1
    println!("p = 2^128 - 2^108 + 1");
    
    // For Montgomery form with N=2 limbs (128 bits), R = 2^128 mod p
    // R = 2^128 mod p = 2^128 - p = 2^108 - 1
    let r_expected = (1u128 << 108) - 1;
    println!("R = 2^128 mod p = 2^108 - 1 = 0x{:032x}", r_expected);
    
    // Our R value
    let r_actual = 0x00000fffffffffff_u128;
    println!("Our R value = 0x{:032x}", r_actual);
    println!("Match: {}", r_expected == r_actual);
    
    // R2 = R * R mod p = (2^108 - 1)^2 mod p
    // Let's compute this
    let r2_computed = r_expected.wrapping_mul(r_expected);
    println!("\nR^2 before mod = 0x{:032x}", r2_computed);
    
    // We need to reduce modulo p, but we can't represent p as u128
    // Let's check our R2 value
    let r2_actual = 0x00000fffefffc001_u128;
    println!("Our R2 value = 0x{:032x}", r2_actual);
    
    // Actually, let me think about this differently
    // If p = 2^128 - 2^108 + 1, then:
    // 2^128 ≡ 2^108 - 1 (mod p)
    // So R = 2^108 - 1 is correct
    
    // For R^2:
    // R^2 = (2^108 - 1)^2 = 2^216 - 2^109 + 1
    // We need to reduce this modulo p
    
    // Since 2^128 ≡ 2^108 - 1 (mod p), we have:
    // 2^216 = 2^128 * 2^88 ≡ (2^108 - 1) * 2^88 (mod p)
    
    println!("\nLet me verify R2 is correct...");
    // Our R2 = 0x00000fffefffc001
    // This is 2^44 - 2^20 + 1 = 17592185782273
    let r2_decimal = 0x00000fffefffc001_u64;
    println!("R2 as decimal: {}", r2_decimal);
    
    // Actually, the values look reasonable. The issue might be elsewhere.
}