use longfellow_algebra::nat::add_with_carry;

fn main() {
    println!("Testing add_with_carry...\n");
    
    // Test: 0xffffffffffffffff + 1 should give (0, 1)
    let a = 0xffffffffffffffff_u64;
    let b = 1_u64;
    let carry_in = 0_u64;
    
    let (sum, carry_out) = add_with_carry(a, b, carry_in);
    
    println!("add_with_carry(0x{:016x}, 0x{:016x}, {}) = (0x{:016x}, {})",
             a, b, carry_in, sum, carry_out);
    
    if sum == 0 && carry_out == 1 {
        println!("✅ Correct!");
    } else {
        println!("❌ Wrong! Expected (0, 1)");
    }
    
    // Test the Rust overflowing_add directly
    let (sum2, overflow) = a.overflowing_add(b);
    println!("\nDirect overflowing_add: (0x{:016x}, {})", sum2, overflow);
}