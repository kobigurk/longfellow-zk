use longfellow_algebra::{Fp128, Field};
use longfellow_algebra::nat::{Nat, mul_wide, add_with_carry};

fn montgomery_reduce_debug(
    extended: &mut Vec<u64>,
    inv: u64,
    modulus: &Nat<2>
) {
    println!("=== Montgomery Reduction Debug ===");
    println!("Initial state:");
    for i in 0..extended.len() {
        if extended[i] != 0 {
            println!("  t[{}] = 0x{:016x}", i, extended[i]);
        }
    }
    
    for i in 0..2 {
        println!("\nIteration {}:", i);
        let k = extended[i].wrapping_mul(inv);
        println!("  k = t[{}] * INV = 0x{:016x}", i, k);
        
        let mut carry = 0u64;
        for j in 0..2 {
            let (lo, hi) = mul_wide(k, modulus.limbs[j]);
            let (sum, c) = add_with_carry(extended[i + j], lo, carry);
            println!("    j={}: t[{}] + {} + {} -> {} (carry={})",
                     j, i+j, lo, carry, sum, c);
            extended[i + j] = sum;
            carry = hi + c;
        }
        
        if carry > 0 && i + 2 < extended.len() {
            println!("  Propagating carry {} to t[{}]", carry, i+2);
            let (sum, c) = add_with_carry(extended[i + 2], 0, carry);
            extended[i + 2] = sum;
            if c > 0 && i + 3 < extended.len() {
                extended[i + 3] += c;
            }
        }
    }
    
    println!("\nBefore shifting:");
    for i in 0..extended.len() {
        if extended[i] != 0 {
            println!("  t[{}] = 0x{:016x}", i, extended[i]);
        }
    }
    
    // Shift right by 2 limbs
    for i in 0..2 {
        extended[i] = if i + 2 < extended.len() { extended[i + 2] } else { 0 };
    }
    extended.truncate(2);
    
    println!("\nAfter shifting (final result):");
    println!("  result = [0x{:016x}, 0x{:016x}]", extended[0], extended[1]);
    
    // Check if >= modulus
    let result = Nat::<2>::new([extended[0], extended[1]]);
    if result >= *modulus {
        println!("  Result >= modulus, subtracting");
        let mut temp = result;
        temp.sub_with_borrow(modulus);
        println!("  After subtraction: [0x{:016x}, 0x{:016x}]", 
                 temp.limbs[0], temp.limbs[1]);
    }
}

fn main() {
    println!("Debugging Montgomery reduction for failing cases\n");
    
    // Test case 1: R * R (should give R after reduction)
    println!("Test 1: R * R -> R");
    let r = Fp128::R;
    let (mut t1, _) = r.mul_wide(&r);
    montgomery_reduce_debug(&mut t1, Fp128::INV, &Fp128::MODULUS);
    
    // Test case 2: What 6 * 6^(-1) actually computes
    println!("\n\nTest 2: 6R * (6^(-1))R");
    
    // We need to get the Montgomery form values
    // From Python: 6R = 0x00005ffffffffffffffffffffffffffa
    //             6^(-1)R = 0x7ffffaaaaaaaaaaaaaaaaaaaaaaaaaab
    
    let six_r = Nat::<2>::new([0xfffffffffffffffa, 0x00005fffffffffff]);
    let six_inv_r = Nat::<2>::new([0xaaaaaaaaaaaaaab, 0x7ffffaaaaaaaaaaa]);
    
    let (mut t2, _) = six_r.mul_wide(&six_inv_r);
    montgomery_reduce_debug(&mut t2, Fp128::INV, &Fp128::MODULUS);
}