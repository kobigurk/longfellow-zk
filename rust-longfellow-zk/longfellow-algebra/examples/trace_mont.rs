use longfellow_algebra::{Fp128, Field};
use longfellow_algebra::nat::{Nat, mul_wide, add_with_carry};

fn main() {
    println!("Detailed trace of from_montgomery...\n");
    
    // Test with R (which is ONE in Montgomery form)
    let r = Fp128::R;
    let modulus = Fp128::MODULUS;
    let inv = Fp128::INV;
    
    println!("Input: R = {:?}", r);
    println!("Expected output: 1\n");
    
    // Create 2N-limb array
    let mut t = vec![0u64; 4];
    t[0] = r.limbs[0];
    t[1] = r.limbs[1];
    
    println!("Initial t:");
    for i in 0..4 {
        println!("  t[{}] = 0x{:016x}", i, t[i]);
    }
    
    // Perform Montgomery reduction
    for i in 0..2 {
        println!("\nIteration i = {}:", i);
        
        // k = t[i] * INV mod 2^64
        let k = t[i].wrapping_mul(inv);
        println!("  k = t[{}] * INV = 0x{:016x} * 0x{:016x} = 0x{:016x}", i, t[i], inv, k);
        
        // t = t + k * p * 2^(64*i)
        let mut carry = 0;
        for j in 0..2 {
            let (lo, hi) = mul_wide(k, modulus.limbs[j]);
            println!("    j={}: k * modulus[{}] = 0x{:016x} * 0x{:016x} = (0x{:016x}, 0x{:016x})", 
                     j, j, k, modulus.limbs[j], lo, hi);
            
            let (sum1, c1) = add_with_carry(t[i + j], lo, carry);
            println!("    t[{}] + lo + carry = 0x{:016x} + 0x{:016x} + {} = (0x{:016x}, {})",
                     i + j, t[i + j], lo, carry, sum1, c1);
            t[i + j] = sum1;
            carry = hi.wrapping_add(c1);
        }
        
        // Propagate carry - this is the key part!
        if carry != 0 {
            println!("    Final carry = 0x{:016x}", carry);
            t[i + 2] = t[i + 2].wrapping_add(carry);
            println!("    Added carry to t[{}]: t[{}] = 0x{:016x}", i + 2, i + 2, t[i + 2]);
        }
        
        println!("  After iteration {}:", i);
        for j in 0..4 {
            println!("    t[{}] = 0x{:016x}", j, t[j]);
        }
    }
    
    println!("\nFinal result (upper 2 limbs):");
    println!("  t[2] = 0x{:016x}", t[2]);
    println!("  t[3] = 0x{:016x}", t[3]);
    
    // Create result
    let mut result = Nat::<2>::default();
    result.limbs[0] = t[2];
    result.limbs[1] = t[3];
    
    println!("\nResult = {:?}", result);
    
    // Check if we need final reduction
    if result >= modulus {
        println!("Result >= modulus, subtracting modulus");
        result.sub_with_borrow(&modulus);
    }
    
    println!("Final result = {:?}", result);
    
    // Convert to bytes to see the value
    let bytes = result.to_bytes_le();
    println!("As bytes: {:?}", bytes);
}