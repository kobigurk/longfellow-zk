use longfellow_algebra::Fp128;
use longfellow_algebra::field::fp_generic::FieldReduction;
use longfellow_algebra::field::fp128::Fp128Reduce;

fn main() {
    println!("Debugging R^2 multiplication\n");
    
    // Create 1 in regular form, then wrap it as Montgomery (but don't convert)
    let one_regular = longfellow_algebra::nat::Nat::<2>::from_u64(1);
    println!("1 regular = {:?}", one_regular);
    
    // Create an Fp128 from the regular value WITHOUT conversion using to_montgomery
    // But we want to bypass the multiplication step, so let's create it directly
    let one_fake_montgomery = unsafe {
        std::mem::transmute::<longfellow_algebra::nat::Nat<2>, Fp128>(one_regular)
    };
    println!("1 as fake Montgomery = {:?}", one_fake_montgomery);
    
    // Create R^2 as an Fp128 WITHOUT conversion  
    let r2_fake_montgomery = unsafe {
        std::mem::transmute::<longfellow_algebra::nat::Nat<2>, Fp128>(Fp128Reduce::R2)
    };
    println!("R^2 as fake Montgomery = {:?}", r2_fake_montgomery);
    
    // Now multiply 1 * R^2 (this should be the Montgomery conversion)
    let result = one_fake_montgomery * r2_fake_montgomery;
    println!("1 * R^2 = {:?}", result);
    
    // This should be the Montgomery form of 1, which is R
    let expected_r = unsafe {
        std::mem::transmute::<longfellow_algebra::nat::Nat<2>, Fp128>(Fp128Reduce::R)
    };
    println!("Expected (R) = {:?}", expected_r);
    
    // Compare the actual values
    let result_from_montgomery = result.from_montgomery();
    let expected_from_montgomery = expected_r.from_montgomery();
    println!("result from_montgomery = {:?}", result_from_montgomery);
    println!("expected from_montgomery = {:?}", expected_from_montgomery);
    
    // Also test 5 * R^2
    println!("\nTesting 5 * R^2:");
    let five_regular = longfellow_algebra::nat::Nat::<2>::from_u64(5);
    let five_fake_montgomery = unsafe {
        std::mem::transmute::<longfellow_algebra::nat::Nat<2>, Fp128>(five_regular)
    };
    let five_times_r2 = five_fake_montgomery * r2_fake_montgomery;
    
    println!("5 * R^2 = {:?}", five_times_r2);
    
    // This should be 5 * R mod p
    // We can compute this by: 5 * R = 5 * 2^108 - 5 (since R = 2^108 - 1)
    // But let's see what we actually get
}