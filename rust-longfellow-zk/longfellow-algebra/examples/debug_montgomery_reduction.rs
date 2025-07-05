use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging Montgomery reduction for (-1) * (-1)");
    
    // Let's manually trace through the Montgomery reduction for (-1) * (-1)
    let one = Fp128::one();
    let minus_one = -one;
    
    // Get the Montgomery form values
    let minus_one_mont = minus_one.value;
    println!("Minus one Montgomery form: {:?}", minus_one_mont);
    
    // Manually compute the wide multiplication
    let (wide_result, _) = minus_one_mont.mul_wide(&minus_one_mont);
    println!("Wide multiplication result: {:?}", wide_result);
    
    // Let's trace what happens in Montgomery reduction
    let mut extended = wide_result.to_vec();
    println!("Extended array before reduction: {:?}", extended);
    
    // Apply the Montgomery reduction and see what we get
    println!("Applying Montgomery reduction...");
    
    // Since we can't call the private method directly, let's do the multiplication
    // and check the result
    let result = minus_one * minus_one;
    println!("Final result: {:?}", result);
    
    // Let's also test with Python to see what the expected result should be
    println!("\nExpected computation:");
    println!("p = 2^128 - 2^108 + 1 = {}", 340282042402384805036647824275747635201u128);
    println!("p-1 = {}", 340282042402384805036647824275747635200u128);
    
    // The regular form multiplication should be: (p-1) * (p-1) mod p
    let p = 340282042402384805036647824275747635201u128;
    let p_minus_1 = p - 1;
    
    // This is too large to compute directly in u128, so let's check what the result should be
    // (p-1)^2 = p^2 - 2p + 1
    // (p-1)^2 mod p = (-2p + 1) mod p = 1 mod p
    println!("Expected result: 1");
    
    // Let's check if our Montgomery constants are correct
    let fp128_r = Fp128::R;
    let fp128_r2 = Fp128::R2;
    let fp128_inv = Fp128::INV;
    
    println!("\nMontgomery constants:");
    println!("R = {:?}", fp128_r);
    println!("R2 = {:?}", fp128_r2);
    println!("INV = 0x{:016x}", fp128_inv);
    
    // Let's also check what -1 in regular form should be
    let minus_one_regular = minus_one.from_montgomery();
    println!("\nRegular form -1: {:?}", minus_one_regular);
    
    // Check if this equals p-1
    let expected_minus_one = (p_minus_1 & ((1u128 << 64) - 1)) as u64;
    let expected_minus_one_high = (p_minus_1 >> 64) as u64;
    
    println!("Expected -1 regular: low=0x{:016x}, high=0x{:016x}", expected_minus_one, expected_minus_one_high);
    
    if minus_one_regular.limbs[0] == expected_minus_one && minus_one_regular.limbs[1] == expected_minus_one_high {
        println!("✓ -1 regular form is correct");
    } else {
        println!("✗ -1 regular form is incorrect");
    }
}