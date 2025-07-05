use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing -1 * -1 multiplication\n");
    
    let one = Fp128::one();
    let minus_one = -one;
    
    println!("1 = {:?}", one);
    println!("-1 = {:?}", minus_one);
    
    // Test (-1) * (-1)
    let minus_one_squared = minus_one * minus_one;
    println!("(-1) * (-1) = {:?}", minus_one_squared);
    
    if minus_one_squared == one {
        println!("✓ (-1) * (-1) = 1 (correct)");
    } else {
        println!("✗ (-1) * (-1) ≠ 1 (multiplication bug!)");
        
        // Check if the result is 0
        let zero = Fp128::zero();
        if minus_one_squared == zero {
            println!("  Result is 0, which suggests a serious multiplication bug");
        }
        
        // Test the values in regular form
        let minus_one_regular = minus_one.from_montgomery();
        let minus_one_squared_regular = minus_one_squared.from_montgomery();
        let one_regular = one.from_montgomery();
        
        println!("\nRegular form values:");
        println!("-1 regular = {:?}", minus_one_regular);
        println!("(-1)^2 regular = {:?}", minus_one_squared_regular);
        println!("1 regular = {:?}", one_regular);
        
        // Convert to u128 for easier debugging
        let minus_one_val = (minus_one_regular.limbs[1] as u128) << 64 | minus_one_regular.limbs[0] as u128;
        let result_val = (minus_one_squared_regular.limbs[1] as u128) << 64 | minus_one_squared_regular.limbs[0] as u128;
        let one_val = (one_regular.limbs[1] as u128) << 64 | one_regular.limbs[0] as u128;
        
        println!("\nAs u128:");
        println!("-1 = {}", minus_one_val);
        println!("(-1)^2 = {}", result_val);
        println!("1 = {}", one_val);
        
        // Expected: p-1 * p-1 = (p-1)^2 mod p = p^2 - 2p + 1 mod p = 1 mod p
        let p = 340282042402384805036647824275747635201u128;
        println!("p = {}", p);
        println!("p-1 = {}", p-1);
        
        if minus_one_val == p - 1 {
            println!("✓ -1 has correct value");
        } else {
            println!("✗ -1 has incorrect value");
        }
    }
    
    // Test some other multiplications
    println!("\nTesting other multiplications:");
    let two = Fp128::from_u64(2);
    let three = Fp128::from_u64(3);
    let six = two * three;
    let expected_six = Fp128::from_u64(6);
    
    println!("2 * 3 = {:?}", six);
    println!("Expected 6 = {:?}", expected_six);
    
    if six == expected_six {
        println!("✓ 2 * 3 = 6 (basic multiplication works)");
    } else {
        println!("✗ 2 * 3 ≠ 6 (basic multiplication broken)");
    }
}