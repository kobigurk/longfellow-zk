use longfellow_algebra::{Fp128, Field};
use longfellow_algebra::nat::{Nat, mul_wide, add_with_carry};

fn main() {
    println!("Tracing Montgomery multiplication: 1 * 1 = 1\n");
    
    // Test the simplest case: 1 * 1 = 1
    // In Montgomery form: R * R with reduction should give R
    
    let modulus = Fp128::MODULUS;
    let inv = Fp128::INV;
    let r = Fp128::R;
    
    println!("R = {:?}", r);
    println!("Modulus = {:?}", modulus);
    println!("INV = 0x{:016x}\n", inv);
    
    // Multiply R * R
    let (mut t, _) = r.mul_wide(&r);
    
    println!("R * R (before reduction):");
    for i in 0..t.len() {
        if t[i] != 0 {
            println!("  t[{}] = 0x{:016x}", i, t[i]);
        }
    }
    
    // Now perform Montgomery reduction
    println!("\nMontgomery reduction:");
    
    for i in 0..2 {
        println!("\nIteration {}:", i);
        let k = t[i].wrapping_mul(inv);
        println!("  k = t[{}] * INV = 0x{:016x}", i, k);
        
        let mut carry = 0u64;
        for j in 0..2 {
            let (lo, hi) = mul_wide(k, modulus.limbs[j]);
            let (sum, c1) = add_with_carry(t[i + j], lo, carry);
            println!("    t[{:}] = 0x{:016x} + 0x{:016x} + {} = 0x{:016x} (c={})",
                     i+j, t[i+j], lo, carry, sum, c1);
            t[i + j] = sum;
            carry = hi + (c1 as u64);
        }
        
        if carry != 0 {
            println!("    Final carry = 0x{:016x}", carry);
            t[i + 2] = t[i + 2].wrapping_add(carry);
        }
    }
    
    println!("\nAfter reduction, before shifting:");
    for i in 0..4 {
        if t[i] != 0 {
            println!("  t[{}] = 0x{:016x}", i, t[i]);
        }
    }
    
    // Result is in upper half
    let result_low = t[2];
    let result_high = t[3];
    
    println!("\nResult (upper half): [0x{:016x}, 0x{:016x}]", result_low, result_high);
    
    // This should be R
    if result_low == r.limbs[0] && result_high == r.limbs[1] {
        println!("✅ Result equals R (correct)");
    } else {
        println!("❌ Result does not equal R");
        
        // Check if we need final reduction
        let mut result = Nat::<2>::new([result_low, result_high]);
        if result >= modulus {
            println!("\nResult >= modulus, needs reduction");
            let borrow = result.sub_with_borrow(&modulus);
            println!("After reduction: {:?} (borrow={})", result, borrow);
            
            if result == r {
                println!("✅ After reduction, result equals R");
            }
        }
    }
}