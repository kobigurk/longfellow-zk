use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging Montgomery reduction issue\n");
    
    // The issue: When we compute R * R and reduce, we get
    // [0xffffffffffffffff, 0x00000ffffffffffe] instead of
    // [0xffffffffffffffff, 0x00000fffffffffff] (which is R)
    
    // The difference is exactly 1 in the high limb
    
    // Let's check what ONE * ONE gives us
    let one = Fp128::one();
    let one_squared = one * one;
    
    println!("ONE = {:?}", one);
    println!("ONE * ONE = {:?}", one_squared);
    
    // ONE should equal itself when squared
    if one_squared == one {
        println!("✅ 1 * 1 = 1");
    } else {
        println!("❌ 1 * 1 != 1");
        
        // Let's check the internal representation
        // We expect both to have the same internal value
        
        // Try to understand the pattern
        // When we multiply small values like 3 * 4, we get:
        // Expected: 0x000000000000000c
        // Got:      0xfffff0000000000c
        
        // The extra part 0xfffff00000000000 is the high part of the modulus
        // This suggests the final reduction isn't working
    }
    
    // Let's test if the issue is consistent
    println!("\nTesting pattern:");
    
    for i in 1u64..5 {
        let a = Fp128::from_u64(i);
        let a_squared = a * a;
        let expected = Fp128::from_u64(i * i);
        
        println!("\n{} * {} = {}", i, i, i * i);
        if a_squared == expected {
            println!("✅ Correct");
        } else {
            println!("❌ Wrong");
            println!("  Expected: {:?}", expected);
            println!("  Got:      {:?}", a_squared);
        }
    }
}