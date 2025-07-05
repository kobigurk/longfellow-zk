use longfellow_algebra::nat::{mul_wide, add_with_carry};

fn main() {
    println!("Testing potential carry overflow in mul_wide\n");
    
    // Test edge case where hi + c might overflow
    let a = u64::MAX;
    let b = u64::MAX;
    
    let (lo, hi) = mul_wide(a, b);
    println!("mul_wide({}, {}) = ({}, {})", a, b, lo, hi);
    
    // Now test add_with_carry with large values
    let sum_input = u64::MAX;
    let add_input = u64::MAX;
    let carry_input = u64::MAX;
    
    let (sum, c) = add_with_carry(sum_input, add_input, carry_input);
    println!("add_with_carry({}, {}, {}) = ({}, {})", sum_input, add_input, carry_input, sum, c);
    
    // Test the overflow case: hi + c
    println!("\nTesting carry overflow:");
    println!("hi = {}", hi);
    println!("c = {}", c);
    
    let carry_sum = hi.wrapping_add(c);
    let (carry_sum_safe, overflow) = hi.overflowing_add(c);
    
    println!("hi + c (wrapping) = {}", carry_sum);
    println!("hi + c (safe) = {}, overflow = {}", carry_sum_safe, overflow);
    
    if overflow {
        println!("⚠️  OVERFLOW DETECTED in carry computation!");
        println!("This could cause the 1-bit error we're seeing");
    } else {
        println!("✓ No overflow in this case");
    }
    
    // Test with more realistic values from omega computation
    println!("\nTesting with realistic omega values...");
    
    // Use some limbs from omega computations that might cause overflow
    let realistic_tests = [
        (0x7c19839f48d38eb4u64, 0x42ff6f23413dd836u64),
        (0x3b1f48f616c4d7a7u64, 0x63e9f4d09188cec8u64),
        (0xffffffffffffffffu64, 0x1u64),
    ];
    
    for (a, b) in realistic_tests {
        let (lo, hi) = mul_wide(a, b);
        let prev_sum = 0u64;
        let carry_in = 0u64;
        
        let (sum, c) = add_with_carry(prev_sum, lo, carry_in);
        let final_carry = hi + c;
        
        println!("mul_wide(0x{:016x}, 0x{:016x}) = (0x{:016x}, 0x{:016x})", a, b, lo, hi);
        println!("add_with_carry(0, 0x{:016x}, 0) = (0x{:016x}, {})", lo, sum, c);
        println!("final carry = 0x{:016x} + {} = 0x{:016x}", hi, c, final_carry);
        
        // Check if this would overflow
        let (safe_carry, overflow) = hi.overflowing_add(c);
        if overflow {
            println!("⚠️  OVERFLOW in realistic case!");
        }
        println!();
    }
}