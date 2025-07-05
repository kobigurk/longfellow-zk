use longfellow_algebra::{Fp128, Field};

fn debug_pow(base: &Fp128, exp: &[u64]) -> Fp128 {
    println!("Computing base^{:?}", exp);
    
    if exp.is_empty() {
        println!("Empty exponent, returning 1");
        return Fp128::one();
    }
    
    let mut result = Fp128::one();
    let mut current_base = *base;
    
    println!("Initial: result = {:?}, base = {:?}", result, current_base);
    
    for (limb_idx, &limb) in exp.iter().enumerate() {
        println!("\nProcessing limb {}: 0x{:016x} ({})", limb_idx, limb, limb);
        
        let mut remaining = limb;
        let mut bit_pos = 0;
        
        while remaining > 0 {
            let bit = remaining & 1;
            println!("  Bit {}: {}, current_base = {:?}", bit_pos, bit, current_base);
            
            if bit == 1 {
                println!("    Multiplying result by current_base");
                result *= &current_base;
                println!("    result = {:?}", result);
            }
            
            println!("    Squaring current_base");
            current_base = current_base.square();
            println!("    current_base = {:?}", current_base);
            
            remaining >>= 1;
            bit_pos += 1;
        }
    }
    
    println!("\nFinal result: {:?}", result);
    result
}

fn main() {
    println!("Debugging power algorithm step by step\n");
    
    let omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_bytes).unwrap();
    println!("omega = {:?}", omega);
    
    // Test omega^16 with debug output
    let omega16_debug = debug_pow(&omega, &[16]);
    let omega16_normal = omega.pow(&[16]);
    
    // Compute omega^16 manually step by step
    let omega2 = omega * omega;
    let omega4 = omega2 * omega2;
    let omega8 = omega4 * omega4;
    let omega16_manual = omega8 * omega8;
    
    println!("\nManual computation:");
    println!("omega^2 = {:?}", omega2);
    println!("omega^4 = {:?}", omega4);
    println!("omega^8 = {:?}", omega8);
    println!("omega^16 = {:?}", omega16_manual);
    
    println!("\nComparisons:");
    println!("Debug result matches manual: {}", omega16_debug == omega16_manual);
    println!("Normal result matches manual: {}", omega16_normal == omega16_manual);
    println!("Debug matches normal: {}", omega16_debug == omega16_normal);
}