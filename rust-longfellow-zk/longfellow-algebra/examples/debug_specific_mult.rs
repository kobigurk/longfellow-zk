use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging specific (-1) * (-1) multiplication step by step\n");
    
    let one = Fp128::one();
    let minus_one = -one;
    
    // Get the regular form values to compare with Python
    let one_regular = one.from_montgomery();
    let minus_one_regular = minus_one.from_montgomery();
    
    println!("Regular form values:");
    let one_val = (one_regular.limbs[1] as u128) << 64 | one_regular.limbs[0] as u128;
    let minus_one_val = (minus_one_regular.limbs[1] as u128) << 64 | minus_one_regular.limbs[0] as u128;
    
    println!("1 = {}", one_val);
    println!("-1 = {}", minus_one_val);
    
    // Check these match our expected values
    let expected_p = 340282042402384805036647824275747635201u128;
    let expected_minus_one = expected_p - 1;
    
    println!("\nVerification:");
    println!("1 correct: {}", one_val == 1);
    println!("-1 correct: {}", minus_one_val == expected_minus_one);
    
    // Now let's manually compute what the Montgomery forms should be
    // We know R = 324518553658426726783156020576255
    let r = 324518553658426726783156020576255u128;
    
    // -1 in Montgomery form should be (p-1) * R mod p
    // From Python: minus_one_montgomery = 340281717883831146609921041119727058946
    let expected_minus_one_mont = 340281717883831146609921041119727058946u128;
    
    println!("\nExpected Montgomery forms:");
    println!("1 in Montgomery form = R = {}", r);
    println!("-1 in Montgomery form = {}", expected_minus_one_mont);
    
    // The product of two Montgomery form numbers should be:
    // (-1_mont) * (-1_mont) = expected_minus_one_mont * expected_minus_one_mont
    let product = (expected_minus_one_mont as u128).wrapping_mul(expected_minus_one_mont as u128);
    println!("\nWide multiplication result:");
    println!("(-1_mont) * (-1_mont) = {}", product);
    println!("Product hex = 0x{:064x}", product);
    
    // The expected product from Python is too large for u128, so we'll skip this comparison
    println!("Expected product from Python is too large for u128 representation");
    
    // But wait - we can't directly compare these because the Rust multiplication might overflow
    // Let's actually compute (-1) * (-1) and see what we get
    let result = minus_one * minus_one;
    let result_regular = result.from_montgomery();
    let result_val = (result_regular.limbs[1] as u128) << 64 | result_regular.limbs[0] as u128;
    
    println!("\nActual result:");
    println!("(-1) * (-1) in regular form = {}", result_val);
    println!("Expected: 1");
    println!("Correct: {}", result_val == 1);
    
    if result_val != 1 {
        println!("\n❌ Montgomery multiplication is producing wrong result!");
        println!("This confirms the Montgomery REDC implementation has a bug.");
        
        // Let's also test a few other small multiplications to see the pattern
        println!("\nTesting other small multiplications:");
        
        let two = Fp128::from_u64(2);
        let three = Fp128::from_u64(3);
        let four = Fp128::from_u64(4);
        
        let two_times_three = two * three;
        let two_times_three_val = (two_times_three.from_montgomery().limbs[1] as u128) << 64 | 
                                  two_times_three.from_montgomery().limbs[0] as u128;
        
        let four_times_four = four * four;
        let four_times_four_val = (four_times_four.from_montgomery().limbs[1] as u128) << 64 | 
                                  four_times_four.from_montgomery().limbs[0] as u128;
        
        println!("2 * 3 = {} (expected 6)", two_times_three_val);
        println!("4 * 4 = {} (expected 16)", four_times_four_val);
        
        if two_times_three_val == 6 && four_times_four_val == 16 {
            println!("✓ Small multiplications work, issue is specific to large values or edge cases");
        } else {
            println!("❌ Small multiplications also broken");
        }
    } else {
        println!("✅ Montgomery multiplication is working correctly!");
    }
}