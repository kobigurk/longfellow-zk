use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing fixed Montgomery reduction\n");
    
    // Test that the fixed montgomery_reduce_wide actually works
    
    // The key test: omega^k where k is large
    let omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_bytes).unwrap();
    println!("omega = {:?}", omega);
    
    // Test small powers first
    let omega2 = omega * omega;
    let omega4 = omega2 * omega2;
    let omega8 = omega4 * omega4;
    
    println!("omega^2 = {:?}", omega2);
    println!("omega^4 = {:?}", omega4);
    println!("omega^8 = {:?}", omega8);
    
    // Convert to regular form to see actual values
    let omega_reg = omega.from_montgomery();
    let omega2_reg = omega2.from_montgomery();
    let omega4_reg = omega4.from_montgomery();
    let omega8_reg = omega8.from_montgomery();
    
    println!("\nRegular forms:");
    println!("omega = {:?}", omega_reg);
    println!("omega^2 = {:?}", omega2_reg);
    println!("omega^4 = {:?}", omega4_reg);
    println!("omega^8 = {:?}", omega8_reg);
    
    // Test if Montgomery form is working by checking if 
    // from_montgomery gives different values than the stored values
    if omega != Fp128::to_montgomery(omega_reg) {
        println!("\n✓ Montgomery form is working!");
        println!("  Stored value != regular form");
    } else {
        println!("\n✗ Montgomery form is still broken");
        println!("  Stored value == regular form (should be different)");
    }
    
    // Test ONE constant specifically
    let one = Fp128::one();
    let one_reg = one.from_montgomery();
    
    println!("\nONE constant:");
    println!("ONE = {:?}", one);
    println!("ONE.from_montgomery() = {:?}", one_reg);
    
    // ONE should be stored as R, not 1
    if one_reg == longfellow_algebra::nat::Nat::<2>::from_u64(1) {
        println!("✓ ONE.from_montgomery() gives 1");
        
        // But ONE itself should be R, not 1
        let expected_r = longfellow_algebra::nat::Nat::<2>::new([0xFFFFFFFFFFFFFFFF, 0x00000FFFFFFFFFFF]);
        let expected_r_fp = Fp128::to_montgomery(expected_r);
        if one == expected_r_fp {
            println!("✓ ONE is stored as R (Montgomery form of 1)");
        } else {
            println!("✗ ONE is not stored as R");
        }
    } else {
        println!("✗ ONE.from_montgomery() does not give 1");
    }
    
    // Test the to_montgomery function
    println!("\nTesting to_montgomery:");
    let regular_one = longfellow_algebra::nat::Nat::<2>::from_u64(1);
    let montgomery_one = Fp128::to_montgomery(regular_one);
    
    println!("to_montgomery(1) = {:?}", montgomery_one);
    
    if montgomery_one == one {
        println!("✓ to_montgomery(1) == ONE");
    } else {
        println!("✗ to_montgomery(1) != ONE");
    }
}