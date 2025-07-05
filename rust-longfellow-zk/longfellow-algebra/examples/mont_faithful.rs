use longfellow_algebra::nat::{Nat, mul_wide, add_with_carry};

fn main() {
    println!("Faithful Montgomery reduction implementation\n");
    
    // Constants for Fp128
    let p_low = 0x0000000000000001_u64;
    let p_high = 0xfffff00000000000_u64;
    let mprime = 0xffffffffffffffff_u64;
    
    // Input: R = [0xffffffffffffffff, 0x00000fffffffffff]
    let r_low = 0xffffffffffffffff_u64;
    let r_high = 0x00000fffffffffff_u64;
    
    println!("Input R = [0x{:016x}, 0x{:016x}]", r_low, r_high);
    println!("Modulus = [0x{:016x}, 0x{:016x}]", p_low, p_high);
    println!("mprime = 0x{:016x}\n", mprime);
    
    // Create working array
    let mut t = [0u64; 4];
    t[0] = r_low;
    t[1] = r_high;
    
    println!("Initial t:");
    for i in 0..4 {
        println!("  t[{}] = 0x{:016x}", i, t[i]);
    }
    
    // Montgomery reduction - matching Python exactly
    for i in 0..2 {
        println!("\nIteration i = {}:", i);
        
        // m = t[i] * mprime mod 2^64
        let m = t[i].wrapping_mul(mprime);
        println!("  m = t[{}] * mprime = 0x{:016x} * 0x{:016x} = 0x{:016x}", 
                 i, t[i], mprime, m);
        
        // Compute m * p and add to t
        // Low part: m * p_low
        let (lo_low, hi_low) = mul_wide(m, p_low);
        println!("  m * p_low = 0x{:016x} * 0x{:016x} = (0x{:016x}, 0x{:016x})",
                 m, p_low, lo_low, hi_low);
        
        // High part: m * p_high
        let (lo_high, hi_high) = mul_wide(m, p_high);
        println!("  m * p_high = 0x{:016x} * 0x{:016x} = (0x{:016x}, 0x{:016x})",
                 m, p_high, lo_high, hi_high);
        
        // Add to t[i] and t[i+1]
        let (new_t_i, c1) = add_with_carry(t[i], lo_low, 0);
        println!("  t[{}] + lo_low = 0x{:016x} + 0x{:016x} = 0x{:016x} (carry={})",
                 i, t[i], lo_low, new_t_i, c1);
        t[i] = new_t_i;
        
        let (new_t_i1, c2) = add_with_carry(t[i+1], lo_high, hi_low + c1);
        println!("  t[{}] + lo_high + carry = 0x{:016x} + 0x{:016x} + {} = 0x{:016x} (carry={})",
                 i+1, t[i+1], lo_high, hi_low + c1, new_t_i1, c2);
        t[i+1] = new_t_i1;
        
        // Propagate carry to t[i+2]
        if i + 2 < 4 {
            let final_carry = hi_high + c2;
            if final_carry != 0 {
                println!("  Propagating carry {} to t[{}]", final_carry, i+2);
                t[i+2] = t[i+2].wrapping_add(final_carry);
            }
        }
        
        println!("  After adding m*p:");
        for j in 0..4 {
            println!("    t[{}] = 0x{:016x}", j, t[j]);
        }
    }
    
    println!("\nFinal result (upper 2 limbs):");
    println!("  t[2] = 0x{:016x}", t[2]);
    println!("  t[3] = 0x{:016x}", t[3]);
    
    // Check if this is 1
    if t[2] == 1 && t[3] == 0 {
        println!("\n✅ SUCCESS: Result is 1");
    } else {
        println!("\n❌ Result is not 1");
        
        // Check if it needs reduction
        if t[3] > p_high || (t[3] == p_high && t[2] >= p_low) {
            println!("Result >= p, needs reduction");
            
            // Subtract p
            let (new_low, b1) = t[2].overflowing_sub(p_low);
            let (new_high, _) = t[3].overflowing_sub(p_high + b1 as u64);
            
            println!("After reduction: [0x{:016x}, 0x{:016x}]", new_low, new_high);
            
            if new_low == 1 && new_high == 0 {
                println!("✅ After reduction, result is 1");
            }
        }
    }
}