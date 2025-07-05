use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging ONE value\n");
    
    let one = Fp128::one();
    println!("one = {:?}", one);
    
    // Get the internal value using to_bytes_le (which calls from_montgomery)
    let one_bytes = one.to_bytes_le();
    println!("one.to_bytes_le() = {:02x?}", one_bytes);
    
    // Get the raw Montgomery form 
    let one_regular = one.from_montgomery();
    println!("one.from_montgomery() = {:?}", one_regular);
    
    // Test what we expect
    use longfellow_algebra::field::fp_generic::FieldReduction;
    use longfellow_algebra::field::fp128::Fp128Reduce;
    
    println!("\nExpected values:");
    println!("R (what ONE should contain) = {:?}", Fp128Reduce::R);
    
    // Test if the issue is in from_montgomery
    // Let's manually check if ONE actually contains R
    
    // Create another value and see what happens
    let two = Fp128::from_u64(2);
    println!("\ntwo = {:?}", two);
    let two_regular = two.from_montgomery();
    println!("two.from_montgomery() = {:?}", two_regular);
    
    // Check if 2 * ONE = 2 (which would be true if ONE actually represents 1)
    let two_times_one = two * one;
    println!("\ntwo * one = {:?}", two_times_one);
    
    if two_times_one == two {
        println!("✓ one acts like multiplicative identity");
    } else {
        println!("✗ one doesn't act like multiplicative identity");
    }
    
    // Check 1 + 1 = 2
    let one_plus_one = one + one;
    println!("\none + one = {:?}", one_plus_one);
    
    if one_plus_one == two {
        println!("✓ one + one = two");
    } else {
        println!("✗ one + one ≠ two");
    }
}