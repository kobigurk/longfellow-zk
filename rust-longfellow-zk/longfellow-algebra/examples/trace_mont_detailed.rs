use longfellow_algebra::{Fp128, Field};
use longfellow_algebra::nat::{Nat, mul_wide, add_with_carry};

fn main() {
    println!("Detailed trace of Montgomery reduction for 6 * 6^(-1)\n");
    
    // From Python: (6R)(6^(-1)R) mod p = R²
    // We expect REDC(R²) = R
    
    let six_r = Nat::<2>::new([0xfffffffffffffffa, 0x00005fffffffffff]);
    let six_inv_r = Nat::<2>::new([0xaaaaaaaaaaaaaaab, 0x7ffffaaaaaaaaaaa]);
    
    println!("6R = {:?}", six_r);
    println!("6^(-1)R = {:?}", six_inv_r);
    
    // Multiply
    let (mut extended, _) = six_r.mul_wide(&six_inv_r);
    
    println!("\nProduct before reduction:");
    println!("Initial extended.len() = {}", extended.len());
    for i in 0..extended.len() {
        if extended[i] != 0 {
            println!("  t[{}] = 0x{:016x}", i, extended[i]);
        }
    }
    
    // This should be R² mod p
    let r2 = Fp128::R2;
    println!("\nExpected R²:");
    println!("  [0x{:016x}, 0x{:016x}]", r2.limbs[0], r2.limbs[1]);
    
    // Montgomery reduction step by step
    let inv = Fp128::INV;
    let modulus = Fp128::MODULUS;
    
    println!("\n=== Montgomery Reduction ===");
    println!("INV = 0x{:016x}", inv);
    println!("p = [0x{:016x}, 0x{:016x}]", modulus.limbs[0], modulus.limbs[1]);
    
    // Make sure we have enough space for carries
    while extended.len() < 5 {
        extended.push(0);
    }
    println!("After extending: extended.len() = {}", extended.len());
    
    for i in 0..2 {
        println!("\nIteration {}:", i);
        println!("  Before: t[{}] = 0x{:016x}", i, extended[i]);
        
        let k = extended[i].wrapping_mul(inv);
        println!("  k = t[{}] * INV = 0x{:016x}", i, k);
        
        let mut carry = 0u64;
        
        for j in 0..2 {
            let (lo, hi) = mul_wide(k, modulus.limbs[j]);
            println!("    k * p[{}] = 0x{:016x} * 0x{:016x} = (0x{:016x}, 0x{:016x})",
                     j, k, modulus.limbs[j], lo, hi);
            
            let (sum, c) = add_with_carry(extended[i + j], lo, carry);
            println!("    t[{}] + lo + carry = 0x{:016x} + 0x{:016x} + {} = 0x{:016x} (c={})",
                     i+j, extended[i + j], lo, carry, sum, c);
            
            extended[i + j] = sum;
            carry = hi + c;
        }
        
        // Propagate carry
        let mut idx = i + 2;
        while idx < extended.len() && carry > 0 {
            let (sum, c) = add_with_carry(extended[idx], 0, carry);
            println!("  Propagating carry {} to t[{}]: {} -> {} (c={})",
                     carry, idx, extended[idx], sum, c);
            extended[idx] = sum;
            carry = c;
            idx += 1;
        }
        
        if carry > 0 {
            println!("  WARNING: Carry {} lost at end of array!", carry);
        }
        
        println!("  After iteration {}:", i);
        println!("  extended.len() = {}", extended.len());
        for j in 0..extended.len() {
            if extended[j] != 0 || j < 4 {
                println!("    t[{}] = 0x{:016x}", j, extended[j]);
            }
        }
    }
    
    println!("\nBefore shifting:");
    for i in 0..extended.len() {
        if extended[i] != 0 {
            println!("  t[{}] = 0x{:016x}", i, extended[i]);
        }
    }
    
    // The result is in positions 2 and 3 (and possibly 4)
    println!("\nResult before shift:");
    println!("  [0x{:016x}, 0x{:016x}, 0x{:016x}]", extended[2], extended[3], 
             if extended.len() > 4 { extended[4] } else { 0 });
    
    // Check if >= modulus
    let result_high = extended[3];
    let result_low = extended[2];
    
    let needs_sub = if result_high > modulus.limbs[1] {
        true
    } else if result_high < modulus.limbs[1] {
        false
    } else {
        result_low >= modulus.limbs[0]
    };
    
    println!("\nConditional subtraction:");
    println!("  result_high (0x{:016x}) vs modulus_high (0x{:016x})", 
             result_high, modulus.limbs[1]);
    println!("  result_low (0x{:016x}) vs modulus_low (0x{:016x})", 
             result_low, modulus.limbs[0]);
    println!("  needs subtraction? {}", needs_sub);
    
    if needs_sub {
        let mut result = Nat::<2>::new([result_low, result_high]);
        let borrow = result.sub_with_borrow(&modulus);
        println!("  After subtraction: [0x{:016x}, 0x{:016x}] (borrow={})",
                 result.limbs[0], result.limbs[1], borrow);
        
        // This should be R!
        let expected_r = Fp128::R;
        println!("\nExpected R = [0x{:016x}, 0x{:016x}]",
                 expected_r.limbs[0], expected_r.limbs[1]);
        println!("Match? {}", result == expected_r);
    }
}