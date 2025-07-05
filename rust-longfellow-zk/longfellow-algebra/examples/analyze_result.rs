fn main() {
    println!("Analyzing the from_montgomery(R) result...\n");
    
    // The result we're getting
    let result_low = 0xffffe00001000000_u64;
    let result_high = 0x00000fffff000000_u64;
    
    println!("Result: [{:016x}, {:016x}]", result_low, result_high);
    
    // Let's see what this is in decimal
    // Actually, let me think about this differently
    
    // The pattern looks like it might be related to the field structure
    // p = 2^128 - 2^108 + 1
    
    // Wait! I just realized something. When I print the bytes with to_bytes_le(),
    // it shows [0, 0, 0, 1, 0, 224, 255, 255, 0, 0, 0, 255, 255, 15]
    
    // Let me reconstruct the value from these bytes
    let bytes = [0u8, 0, 0, 1, 0, 224, 255, 255, 0, 0, 0, 255, 255, 15, 0, 0];
    
    let mut value_low = 0u64;
    let mut value_high = 0u64;
    
    for i in 0..8 {
        value_low |= (bytes[i] as u64) << (i * 8);
    }
    for i in 0..8 {
        value_high |= (bytes[i + 8] as u64) << (i * 8);
    }
    
    println!("\nReconstructed from bytes:");
    println!("Low:  0x{:016x}", value_low);
    println!("High: 0x{:016x}", value_high);
    
    // Hmm, the bytes look different. Let me check if the bytes are being
    // truncated or reordered somehow
    
    // Actually, I notice that bytes[3] = 1. This suggests the value might
    // actually be close to 1, but with some extra bits
    
    // Let's check byte-by-byte
    println!("\nByte analysis:");
    for (i, &b) in bytes.iter().enumerate() {
        if b != 0 {
            println!("  bytes[{}] = {} (0x{:02x})", i, b, b);
        }
    }
    
    // bytes[3] = 1 means bit 24-31 is 1
    // This would be 0x01000000 = 16777216
    
    // Wait, I think the issue might be endianness or how we're interpreting
    // the result. Let me check if this is actually representing 1 in some way.
}