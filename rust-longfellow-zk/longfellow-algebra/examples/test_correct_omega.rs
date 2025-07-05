use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing the mathematically correct omega value\n");
    
    // Test the new omega_32 value
    let omega_32 = Fp128::get_root_of_unity(1 << 32).unwrap();
    println!("omega_32 = {:?}", omega_32);
    
    // Test if omega_32^(2^31) = -1
    let omega_2_31 = omega_32.pow(&[1u64 << 31]);
    let minus_one = -Fp128::one();
    
    println!("omega_32^(2^31) = {:?}", omega_2_31);
    println!("-1 = {:?}", minus_one);
    
    if omega_2_31 == minus_one {
        println!("✓ omega_32^(2^31) = -1 (correct!)");
    } else {
        println!("✗ omega_32^(2^31) ≠ -1 (still wrong)");
    }
    
    // Test if (omega_32^(2^31))^2 = 1
    let omega_2_31_squared = omega_2_31 * omega_2_31;
    let one = Fp128::one();
    
    println!("(omega_32^(2^31))^2 = {:?}", omega_2_31_squared);
    println!("1 = {:?}", one);
    
    if omega_2_31_squared == one {
        println!("✓ (omega_32^(2^31))^2 = 1 (correct!)");
    } else {
        println!("✗ (omega_32^(2^31))^2 ≠ 1 (still wrong)");
    }
    
    // Test smaller roots of unity
    println!("\nTesting smaller roots of unity:");
    
    let omega_2 = Fp128::get_root_of_unity(2).unwrap();
    println!("omega_2 = {:?}", omega_2);
    println!("omega_2^2 = {:?}", omega_2 * omega_2);
    
    if omega_2 * omega_2 == one {
        println!("✓ omega_2^2 = 1");
    } else {
        println!("✗ omega_2^2 ≠ 1");
    }
    
    if omega_2 == minus_one {
        println!("✓ omega_2 = -1 (correct for 2nd root of unity)");
    } else {
        println!("✗ omega_2 ≠ -1");
    }
    
    // Test omega_4
    let omega_4 = Fp128::get_root_of_unity(4).unwrap();
    println!("\nomega_4 = {:?}", omega_4);
    println!("omega_4^2 = {:?}", omega_4 * omega_4);
    println!("omega_4^4 = {:?}", omega_4.pow(&[4]));
    
    if omega_4.pow(&[4]) == one {
        println!("✓ omega_4^4 = 1");
    } else {
        println!("✗ omega_4^4 ≠ 1");
    }
    
    if omega_4 * omega_4 == minus_one {
        println!("✓ omega_4^2 = -1 (correct for 4th root of unity)");
    } else {
        println!("✗ omega_4^2 ≠ -1");
    }
}