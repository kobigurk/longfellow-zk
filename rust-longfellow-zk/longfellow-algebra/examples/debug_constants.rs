use longfellow_algebra::field::fp_generic::FieldReduction;
use longfellow_algebra::field::fp128::Fp128Reduce;

fn main() {
    println!("Debugging Montgomery constants\n");
    
    println!("MODULUS = {:?}", Fp128Reduce::MODULUS);
    println!("R = {:?}", Fp128Reduce::R);
    println!("R2 = {:?}", Fp128Reduce::R2);
    println!("INV = 0x{:016x}", Fp128Reduce::INV);
    
    // R should be 2^128 mod p where p = 2^128 - 2^108 + 1
    // 2^128 = p + 2^108 - 1
    // So 2^128 mod p = 2^108 - 1
    
    // In hex, 2^108 = 0x1000000000000000000000000000 (1 followed by 27 zeros)
    // So 2^108 - 1 = 0x0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
    
    println!("\nExpected R (2^108 - 1):");
    println!("2^108 = 0x1000000000000000000000000000");
    println!("2^108 - 1 = 0x0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    
    // Check if our R matches this
    let expected_r_hi = 0x00000FFFFFFFFFFF_u64;
    let expected_r_lo = 0xFFFFFFFFFFFFFFFF_u64;
    
    println!("\nExpected R limbs:");
    println!("  [0] = 0x{:016x}", expected_r_lo);
    println!("  [1] = 0x{:016x}", expected_r_hi);
    
    println!("\nActual R limbs:");
    println!("  [0] = 0x{:016x}", Fp128Reduce::R.limbs[0]);
    println!("  [1] = 0x{:016x}", Fp128Reduce::R.limbs[1]);
    
    if Fp128Reduce::R.limbs[0] == expected_r_lo && Fp128Reduce::R.limbs[1] == expected_r_hi {
        println!("✓ R constant is correct");
    } else {
        println!("✗ R constant is wrong!");
    }
}