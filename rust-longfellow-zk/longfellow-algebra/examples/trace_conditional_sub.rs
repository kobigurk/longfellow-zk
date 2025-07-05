use longfellow_algebra::{Fp128, Field};
use longfellow_algebra::nat::{Nat, Limb};

fn compare_nats(a: &Nat<2>, b: &Nat<2>) -> std::cmp::Ordering {
    for i in (0..2).rev() {
        if a.limbs[i] > b.limbs[i] {
            return std::cmp::Ordering::Greater;
        } else if a.limbs[i] < b.limbs[i] {
            return std::cmp::Ordering::Less;
        }
    }
    std::cmp::Ordering::Equal
}

fn main() {
    println!("Testing conditional subtraction in Montgomery reduction\n");
    
    // The value before conditional subtraction in 6 * 6^(-1)
    let before_sub = Nat::<2>::new([0x4000000000000000, 0xfffffffffffffffc]);
    let modulus = Fp128::MODULUS;
    
    println!("Before subtraction:");
    println!("  value = [0x{:016x}, 0x{:016x}]", before_sub.limbs[0], before_sub.limbs[1]);
    println!("  modulus = [0x{:016x}, 0x{:016x}]", modulus.limbs[0], modulus.limbs[1]);
    
    // Check if value >= modulus
    let cmp = compare_nats(&before_sub, &modulus);
    println!("\nComparison: {:?}", cmp);
    
    // Check limb by limb
    println!("\nLimb-by-limb comparison:");
    println!("  High limb: 0x{:016x} vs 0x{:016x}", before_sub.limbs[1], modulus.limbs[1]);
    if before_sub.limbs[1] > modulus.limbs[1] {
        println!("    value[1] > modulus[1] - should subtract");
    } else if before_sub.limbs[1] < modulus.limbs[1] {
        println!("    value[1] < modulus[1] - should NOT subtract");
    } else {
        println!("    value[1] == modulus[1] - check low limb");
        println!("  Low limb: 0x{:016x} vs 0x{:016x}", before_sub.limbs[0], modulus.limbs[0]);
        if before_sub.limbs[0] >= modulus.limbs[0] {
            println!("    value[0] >= modulus[0] - should subtract");
        } else {
            println!("    value[0] < modulus[0] - should NOT subtract");
        }
    }
    
    // Do the subtraction
    let mut result = before_sub.clone();
    let borrow = result.sub_with_borrow(&modulus);
    
    println!("\nAfter subtraction:");
    println!("  result = [0x{:016x}, 0x{:016x}]", result.limbs[0], result.limbs[1]);
    println!("  borrow = {}", borrow);
    
    // What is the result value?
    println!("\nResult analysis:");
    let result_val = result.limbs[0] as u128 | ((result.limbs[1] as u128) << 64);
    let modulus_val = modulus.limbs[0] as u128 | ((modulus.limbs[1] as u128) << 64);
    
    println!("  Result value: 0x{:032x}", result_val);
    println!("  Modulus value: 0x{:032x}", modulus_val);
    
    if result_val == 0 {
        println!("  Result is 0!");
    } else if result_val == modulus_val - 1 {
        println!("  Result is p-1!");
    } else {
        println!("  Result is something else: {}", result_val);
    }
}