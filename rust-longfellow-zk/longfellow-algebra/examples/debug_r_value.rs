use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging R value representation...\n");
    
    // Check our R constant
    let r = Fp128::R;
    println!("Fp128::R = {:?}", r);
    println!("R.limbs[0] = 0x{:016x}", r.limbs[0]);
    println!("R.limbs[1] = 0x{:016x}", r.limbs[1]);
    
    // Python shows R = 0x00000fffffffffffffffffffffffffff
    // This is 324518553658426726783156020576255
    // In 64-bit limbs:
    // Low limb:  0xffffffffffffffff (all of the low 64 bits)
    // High limb: 0x00000fffffffffff (the remaining 44 bits)
    
    println!("\nExpected R representation:");
    println!("R.limbs[0] = 0xffffffffffffffff");
    println!("R.limbs[1] = 0x00000fffffffffff");
    
    // Check if our R is correct
    let expected_low = 0xffffffffffffffff_u64;
    let expected_high = 0x00000fffffffffff_u64;
    
    if r.limbs[0] == expected_low && r.limbs[1] == expected_high {
        println!("\n❌ Our R has wrong limb order!");
    } else if r.limbs[0] == expected_high && r.limbs[1] == expected_low {
        println!("\n❌ Our R has swapped limbs!");
    } else {
        println!("\n✅ Our R matches expected value");
    }
}