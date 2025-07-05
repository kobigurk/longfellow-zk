use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing simple squaring\n");
    
    // Test -1
    let minus_one = -Fp128::one();
    let minus_one_squared = minus_one * minus_one;
    println!("(-1) * (-1) = {:?}", minus_one_squared);
    println!("Expected: {:?}", Fp128::one());
    println!("Correct? {}\n", minus_one_squared == Fp128::one());
    
    // Test a simple value
    let three = Fp128::from_u64(3);
    let nine = three * three;
    let nine_expected = Fp128::from_u64(9);
    println!("3 * 3 = {:?}", nine);
    println!("Expected: {:?}", nine_expected);
    println!("Correct? {}\n", nine == nine_expected);
    
    // Test 2
    let two = Fp128::from_u64(2);
    let four = two * two;
    let four_expected = Fp128::from_u64(4);
    println!("2 * 2 = {:?}", four);
    println!("Expected: {:?}", four_expected);
    println!("Correct? {}", four == four_expected);
}