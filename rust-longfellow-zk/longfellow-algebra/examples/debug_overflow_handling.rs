use longfellow_algebra::{Fp128, Field};
use longfellow_algebra::nat::Nat;

fn main() {
    println!("Debugging overflow handling\n");
    
    // Simulate the overflow case
    // After Montgomery reduction, we have extended = [0, 0, 0, 0, 1]
    // After shifting right by 2: result = [0, 0] with overflow bit = 1
    
    let mut result = Nat::<2>::new([0, 0]);
    let modulus = Fp128::MODULUS;
    let r = Fp128::R;
    
    println!("Initial result: {:?}", result);
    println!("Modulus: {:?}", modulus);
    println!("R: {:?}", r);
    
    // The overflow handling code does:
    // 1. result.sub_with_borrow(&modulus)
    // 2. result.add_with_carry(&R)
    
    println!("\nStep 1: result - modulus");
    let borrow = result.sub_with_borrow(&modulus);
    println!("After subtraction: {:?}", result);
    println!("Borrow: {}", borrow);
    
    println!("\nStep 2: result + R");
    let carry = result.add_with_carry(&r);
    println!("After addition: {:?}", result);
    println!("Carry: {}", carry);
    
    // What should we get?
    // We want 2^128 mod p = R
    // But we're getting something else
    
    println!("\nExpected: R = {:?}", r);
    println!("Got: {:?}", result);
    println!("Are they equal? {}", result == r);
    
    // Let's check what 2 in Montgomery form looks like
    let two_mont = Fp128::from_u64(2);
    let two_mont_limbs = unsafe {
        std::mem::transmute::<_, [u64; 2]>(two_mont)
    };
    println!("\n2 in Montgomery form: [0x{:016x}, 0x{:016x}]",
             two_mont_limbs[0], two_mont_limbs[1]);
}