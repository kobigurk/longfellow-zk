use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Checking ONE constant...\n");
    
    // ONE should be R in Montgomery form
    let one = Fp128::ONE;
    let r = Fp128::R;
    
    println!("ONE = {:?}", one);
    println!("R   = {:?}", r);
    
    // Check by converting to bytes
    let one_bytes = one.to_bytes_le();
    println!("\nONE.to_bytes_le() = {:?}", one_bytes);
    
    // ONE in Montgomery form should convert to 1
    if one_bytes == vec![1] {
        println!("✅ ONE converts to 1 (correct)");
    } else {
        println!("❌ ONE doesn't convert to 1!");
    }
    
    // Check what from_u64(1) produces
    let one_from_u64 = Fp128::from_u64(1);
    println!("\nFp128::from_u64(1) = {:?}", one_from_u64);
    
    // This should equal ONE
    if one_from_u64 == one {
        println!("✅ from_u64(1) == ONE");
    } else {
        println!("❌ from_u64(1) != ONE");
        
    }
}