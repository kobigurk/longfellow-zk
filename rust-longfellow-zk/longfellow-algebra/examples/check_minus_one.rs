use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Checking -1 in Fp128\n");
    
    let minus_one = -Fp128::one();
    println!("-1 = {:?}", minus_one);
    
    // Get the raw limbs
    let minus_one_limbs = unsafe {
        std::mem::transmute::<_, [u64; 2]>(minus_one)
    };
    println!("-1 limbs = [0x{:016x}, 0x{:016x}]", minus_one_limbs[0], minus_one_limbs[1]);
    
    // Square it
    let minus_one_squared = minus_one.square();
    println!("\n(-1)² = {:?}", minus_one_squared);
    
    if minus_one_squared == Fp128::one() {
        println!("✓ (-1)² = 1");
    } else {
        println!("✗ (-1)² ≠ 1");
    }
    
    // Check the non-Montgomery form
    let minus_one_regular = minus_one.from_montgomery();
    println!("\n-1 in regular form = {:?}", minus_one_regular);
    
    // For comparison, what is p-1?
    let p = Fp128::MODULUS;
    println!("\np = {:?}", p);
    println!("p - 1 should be -1 in regular form");
}