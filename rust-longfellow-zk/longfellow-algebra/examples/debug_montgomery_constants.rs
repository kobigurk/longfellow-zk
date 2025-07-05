use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging Montgomery constants\n");
    
    // Test if INV is correct
    // INV should be such that p * INV ≡ -1 (mod 2^64)
    // For p = 2^128 - 2^108 + 1, the low limb is 1
    // So we need: 1 * INV ≡ -1 (mod 2^64)
    // Therefore INV = 2^64 - 1 = 0xFFFFFFFFFFFFFFFF
    
    let p_low = 1u64;
    let inv = 0xFFFFFFFFFFFFFFFFu64;
    let product = p_low.wrapping_mul(inv);
    println!("p_low * INV = 0x{:016x}", product);
    println!("Should be 0xFFFFFFFFFFFFFFFF (i.e., -1 mod 2^64)");
    
    // Test a simple value conversion
    let five = Fp128::from_u64(5);
    println!("\nTesting value 5:");
    println!("five in Montgomery = {:?}", five);
    
    let five_regular = five.from_montgomery();
    println!("five from_montgomery = {:?}", five_regular);
    
    // Convert back to Montgomery to see if it's correct
    let five_back = Fp128::to_montgomery(five_regular);
    println!("five back to Montgomery = {:?}", five_back);
    
    // They should be equal if the conversion works
    if five == five_back {
        println!("✓ Montgomery conversion is working correctly");
    } else {
        println!("✗ Montgomery conversion is broken");
    }
    
    // Test what 5 * R^(-1) should be
    // If 5 is stored as 5*R mod p, then from_montgomery should give us 5
    println!("\nAnalyzing what from_montgomery should produce:");
    println!("If from_u64(5) stores 5*R mod p, then from_montgomery should give 5");
    println!("But we're getting the same value, which suggests the function isn't working");
}