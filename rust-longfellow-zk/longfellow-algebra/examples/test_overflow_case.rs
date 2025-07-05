use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing overflow case directly\n");
    
    // Test 6 * 6^(-1)
    let six = Fp128::from_u64(6);
    let six_inv = six.invert().unwrap();
    let product = six * six_inv;
    
    println!("6 * 6^(-1) = {:?}", product);
    println!("Expected: {:?}", Fp128::one());
    
    // Get raw bytes to see the actual value
    let product_bytes = unsafe {
        std::mem::transmute::<_, [u64; 2]>(product)
    };
    
    println!("\nProduct limbs: [0x{:016x}, 0x{:016x}]", 
             product_bytes[0], product_bytes[1]);
    
    let one_bytes = unsafe {
        std::mem::transmute::<_, [u64; 2]>(Fp128::one())
    };
    
    println!("One limbs: [0x{:016x}, 0x{:016x}]",
             one_bytes[0], one_bytes[1]);
    
    // The result 0xffffe000010000000000000000000001
    // As limbs: [0x0000000000000001, 0xffffe00001000000]
    
    println!("\nChecking the value we got:");
    let val_low = 0x0000000000000001u64;
    let val_high = 0xffffe00001000000u64;
    
    println!("Got: [0x{:016x}, 0x{:016x}]", val_low, val_high);
    
    // This looks wrong. Let's check what R should be
    println!("\nExpected R = {:?}", Fp128::R);
}