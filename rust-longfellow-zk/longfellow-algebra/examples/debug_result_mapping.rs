use longfellow_algebra::{Fp128, Field};
use longfellow_algebra::nat::Nat;

fn main() {
    println!("Debugging result mapping\n");
    
    // The result we got from Montgomery reduction: [0x3fffffffffffffff, 0x00000ffffffffffc]
    let result_limbs = [0x3fffffffffffffff, 0x00000ffffffffffc];
    
    println!("Result from Montgomery reduction:");
    println!("  limbs = [0x{:016x}, 0x{:016x}]", result_limbs[0], result_limbs[1]);
    
    // What's this value?
    let val = result_limbs[0] as u128 | ((result_limbs[1] as u128) << 64);
    println!("  As u128: 0x{:032x}", val);
    
    // Compare with R
    let r_limbs = [0xFFFFFFFFFFFFFFFF, 0x00000FFFFFFFFFFF];
    let r_val = r_limbs[0] as u128 | ((r_limbs[1] as u128) << 64);
    println!("\nR = [0x{:016x}, 0x{:016x}]", r_limbs[0], r_limbs[1]);
    println!("R as u128: 0x{:032x}", r_val);
    
    println!("\nAre they equal? {}", result_limbs == r_limbs);
    
    // Check bit differences
    println!("\nBit differences:");
    println!("  Low limb:  result=0x{:016x} R=0x{:016x}", result_limbs[0], r_limbs[0]);
    println!("  High limb: result=0x{:016x} R=0x{:016x}", result_limbs[1], r_limbs[1]);
    
    // The issue: the result is NOT R!
    // Let's check what it should be
    println!("\n--- Expected behavior ---");
    println!("6 * 6^(-1) in Montgomery form should give R");
    println!("But we're getting a different value!");
    
    // Let's manually check the comparison
    let result_nat = Nat::<2>::new(result_limbs);
    let r_nat = Nat::<2>::new(r_limbs);
    
    println!("\nresult == R? {}", result_nat == r_nat);
    
    // What about the modulus?
    let p = Fp128::MODULUS;
    println!("\nModulus = [0x{:016x}, 0x{:016x}]", p.limbs[0], p.limbs[1]);
    
    // Is our result congruent to R mod p?
    // Since both should be < p, they should be equal if congruent
}