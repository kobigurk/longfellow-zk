fn main() {
    // For Fp128: p = 2^128 - 2^108 + 1
    // We need to compute -p^(-1) mod 2^64
    
    // p mod 2^64 = 1 (since p = ...0000000000000001)
    // So p^(-1) mod 2^64 = 1
    // Therefore -p^(-1) mod 2^64 = -1 mod 2^64 = 0xFFFFFFFFFFFFFFFF
    
    let p_low = 1u64;
    println!("p mod 2^64 = {}", p_low);
    println!("-p^(-1) mod 2^64 = 0x{:016X}", 0xFFFFFFFFFFFFFFFFu64);
    
    // Let's verify: p * (-p^(-1)) = -1 (mod 2^64)
    let inv = 0xFFFFFFFFFFFFFFFFu64;
    let product = p_low.wrapping_mul(inv);
    println!("\nVerification: p * INV mod 2^64 = {} * 0x{:X} = 0x{:X}", p_low, inv, product);
    println!("This should be -1 mod 2^64 = 0xFFFFFFFFFFFFFFFF");
    
    // So the correct INV is 0xFFFFFFFFFFFFFFFF, not 0xFFFFEFFFFFFFFFFF
}