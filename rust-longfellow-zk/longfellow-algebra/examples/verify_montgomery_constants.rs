use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Verifying Montgomery constants for Fp128\n");
    
    // Get the constants
    let p = Fp128::MODULUS;
    let r = Fp128::R;
    let r2 = Fp128::R2;
    let inv = Fp128::INV;
    
    println!("p (modulus) = {:?}", p);
    println!("R = {:?}", r);
    println!("R^2 = {:?}", r2);
    println!("INV = 0x{:016x}", inv);
    
    // Convert to u128 for easier comparison
    let p_val = (p.limbs[1] as u128) << 64 | p.limbs[0] as u128;
    let r_val = (r.limbs[1] as u128) << 64 | r.limbs[0] as u128;
    let r2_val = (r2.limbs[1] as u128) << 64 | r2.limbs[0] as u128;
    
    println!("\nAs u128:");
    println!("p = {}", p_val);
    println!("R = {}", r_val);
    println!("R^2 = {}", r2_val);
    
    // Expected values from Python
    let expected_p = 340282042402384805036647824275747635201u128;
    let expected_r = 324518553658426726783156020576255u128;
    let expected_r2 = 82427712319755083633389905903615745u128;
    let expected_inv = 18446744073709551615u64;
    
    println!("\nExpected values:");
    println!("p = {}", expected_p);
    println!("R = {}", expected_r);
    println!("R^2 = {}", expected_r2);
    println!("INV = {}", expected_inv);
    
    println!("\nVerification:");
    println!("p matches: {}", p_val == expected_p);
    println!("R matches: {}", r_val == expected_r);
    println!("R^2 matches: {}", r2_val == expected_r2);
    println!("INV matches: {}", inv == expected_inv);
    
    // Test conversion to Montgomery form
    println!("\nTesting conversion to Montgomery form:");
    let one = Fp128::one();
    let minus_one = -one;
    
    println!("1 in Montgomery form: {:?}", one);
    println!("-1 in Montgomery form: {:?}", minus_one);
    
    // Convert back to regular form
    let one_regular = one.from_montgomery();
    let minus_one_regular = minus_one.from_montgomery();
    
    println!("1 in regular form: {:?}", one_regular);
    println!("-1 in regular form: {:?}", minus_one_regular);
    
    // Check if they match expected values
    let one_regular_val = (one_regular.limbs[1] as u128) << 64 | one_regular.limbs[0] as u128;
    let minus_one_regular_val = (minus_one_regular.limbs[1] as u128) << 64 | minus_one_regular.limbs[0] as u128;
    
    println!("\nRegular form values:");
    println!("1 = {}", one_regular_val);
    println!("-1 = {}", minus_one_regular_val);
    
    println!("\nExpected regular form values:");
    println!("1 = 1");
    println!("-1 = {}", expected_p - 1);
    
    println!("\nRegular form verification:");
    println!("1 correct: {}", one_regular_val == 1);
    println!("-1 correct: {}", minus_one_regular_val == expected_p - 1);
    
    // Test that 1 in Montgomery form is R
    // We can't access the private field directly, so we'll infer it from the fact that
    // 1 * R = R in Montgomery form, and we know R from the constants
    println!("\nMontgomery form verification:");
    println!("1 in Montgomery form should be R = {}", expected_r);
    println!("We can verify by checking that 1 converts back to 1 correctly");
}