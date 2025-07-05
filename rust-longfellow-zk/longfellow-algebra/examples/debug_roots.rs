use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging roots of unity\n");
    
    // Get 2nd root of unity
    if let Some(omega_2) = Fp128::get_root_of_unity(2) {
        println!("omega_2 = {:?}", omega_2);
        
        let omega_2_squared = omega_2.square();
        println!("omega_2² = {:?}", omega_2_squared);
        println!("Expected: {:?}", Fp128::one());
        
        if omega_2_squared == Fp128::one() {
            println!("✓ omega_2² = 1");
        } else {
            println!("✗ omega_2² ≠ 1");
            
            // Debug more
            let omega_2_cubed = omega_2_squared * omega_2;
            let omega_2_fourth = omega_2_squared.square();
            
            println!("\nomega_2³ = {:?}", omega_2_cubed);
            println!("omega_2⁴ = {:?}", omega_2_fourth);
        }
        
        // Check if it's -1
        let minus_one = -Fp128::one();
        println!("\n-1 = {:?}", minus_one);
        if omega_2 == minus_one {
            println!("omega_2 is -1");
        }
    } else {
        println!("No 2nd root of unity found");
    }
}