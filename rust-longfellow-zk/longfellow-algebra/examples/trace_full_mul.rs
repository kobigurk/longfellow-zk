use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Tracing full multiplication for 6 * 6^(-1)\n");
    
    let six = Fp128::from_u64(6);
    let six_inv = six.invert().unwrap();
    
    println!("6 in Montgomery form: {:?}", six);
    println!("6^(-1) in Montgomery form: {:?}", six_inv);
    
    // Get the actual limb values
    let six_limbs = unsafe { 
        std::mem::transmute::<_, [u64; 2]>(six)
    };
    let six_inv_limbs = unsafe {
        std::mem::transmute::<_, [u64; 2]>(six_inv)
    };
    
    println!("\n6R = [0x{:016x}, 0x{:016x}]", six_limbs[0], six_limbs[1]);
    println!("6^(-1)R = [0x{:016x}, 0x{:016x}]", six_inv_limbs[0], six_inv_limbs[1]);
    
    // Perform the multiplication
    let product = six * six_inv;
    let product_limbs = unsafe {
        std::mem::transmute::<_, [u64; 2]>(product)
    };
    
    println!("\nProduct = [0x{:016x}, 0x{:016x}]", product_limbs[0], product_limbs[1]);
    
    if product == Fp128::zero() {
        println!("\n❌ Product is ZERO!");
    } else if product == Fp128::one() {
        println!("\n✅ Product is ONE!");
    } else {
        println!("\n❓ Product is neither zero nor one");
    }
    
    // Let's also check what ONE looks like
    let one_limbs = unsafe {
        std::mem::transmute::<_, [u64; 2]>(Fp128::one())
    };
    println!("\nONE = [0x{:016x}, 0x{:016x}]", one_limbs[0], one_limbs[1]);
    
    // And ZERO
    let zero_limbs = unsafe {
        std::mem::transmute::<_, [u64; 2]>(Fp128::zero())
    };
    println!("ZERO = [0x{:016x}, 0x{:016x}]", zero_limbs[0], zero_limbs[1]);
}