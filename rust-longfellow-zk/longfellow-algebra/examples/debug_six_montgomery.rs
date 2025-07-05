use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging 6 in Montgomery form\n");
    
    // Let's trace how 6 is converted to Montgomery form
    let six = Fp128::from_u64(6);
    println!("6 = {:?}", six);
    
    // from_u64 calls to_montgomery which multiplies by R^2 and reduces
    // So 6 in Montgomery form should be 6R mod p
    
    // Let's check if this is correct
    // Expected: 6R mod p = 0x00005ffffffffffffffffffffffffffa
    
    // But the debug output shows 6, not 6R
    // This suggests the Debug implementation is calling from_montgomery
    
    // Let's check by doing arithmetic
    let one = Fp128::one();
    let six_times_one = six * one;
    
    println!("\n6 * 1 = {:?}", six_times_one);
    
    if six_times_one == six {
        println!("✅ 6 * 1 = 6");
    } else {
        println!("❌ 6 * 1 != 6");
    }
    
    // Let's also check 1 * 1
    let one_squared = one * one;
    println!("\n1 * 1 = {:?}", one_squared);
    
    if one_squared == one {
        println!("✅ 1 * 1 = 1");
    } else {
        println!("❌ 1 * 1 != 1");
    }
    
    // Now let's manually check what 6 * 6^(-1) should be
    // We know 6^(-1) in Montgomery form is 0x7ffffaaaaaaaaaaaaaaaaaaaaaaaaaab
    // So we need to multiply 6R * (6^(-1))R and reduce
    
    // Actually, let's just verify that our arithmetic is self-consistent
    let two = Fp128::from_u64(2);
    let three = Fp128::from_u64(3);
    let two_times_three = two * three;
    
    println!("\n2 * 3 = {:?}", two_times_three);
    if two_times_three == six {
        println!("✅ 2 * 3 = 6");
    } else {
        println!("❌ 2 * 3 != 6");
        println!("Expected: {:?}", six);
    }
    
    // Check division
    let six_div_two = six * two.invert().unwrap();
    println!("\n6 / 2 = {:?}", six_div_two);
    if six_div_two == three {
        println!("✅ 6 / 2 = 3");
    } else {
        println!("❌ 6 / 2 != 3");
    }
    
    let six_div_three = six * three.invert().unwrap();
    println!("\n6 / 3 = {:?}", six_div_three);
    if six_div_three == two {
        println!("✅ 6 / 3 = 2");
    } else {
        println!("❌ 6 / 3 != 2");
    }
}