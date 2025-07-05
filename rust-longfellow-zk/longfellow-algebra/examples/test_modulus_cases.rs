use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing edge cases with modulus\n");
    
    // Test if p - 1 works correctly
    // In the field, p ≡ 0, so p - 1 ≡ -1
    let p_bytes = vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 255, 255];
    let p_minus_1_bytes = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 255, 255];
    
    let p_val = Fp128::from_bytes_le(&p_bytes);
    let p_minus_1_val = Fp128::from_bytes_le(&p_minus_1_bytes);
    
    if let (Ok(p), Ok(p_minus_1)) = (p_val, p_minus_1_val) {
        println!("p as field element = {:?}", p);
        println!("p - 1 as field element = {:?}", p_minus_1);
        
        // p should equal 0 in the field
        if p == Fp128::zero() {
            println!("✅ p ≡ 0 (mod p)");
        } else {
            println!("❌ p !≡ 0 (mod p)");
        }
        
        // p - 1 should equal -1
        let minus_one = -Fp128::one();
        if p_minus_1 == minus_one {
            println!("✅ p - 1 ≡ -1 (mod p)");
        } else {
            println!("❌ p - 1 !≡ -1 (mod p)");
            println!("Expected -1 = {:?}", minus_one);
        }
    }
    
    // Now let's test if the issue with 6 is related to a specific pattern
    println!("\n\nTesting multiples of 6:");
    
    let six = Fp128::from_u64(6);
    for i in 1u64..10 {
        let multiple = Fp128::from_u64(i) * six;
        let inv = multiple.invert();
        
        if let Some(inv_val) = inv {
            let prod = multiple * inv_val;
            if prod == Fp128::one() {
                println!("✅ {}*6 * ({}*6)^(-1) = 1", i, i);
            } else {
                println!("❌ {}*6 * ({}*6)^(-1) != 1", i, i);
            }
        } else {
            println!("! {}*6 has no inverse", i);
        }
    }
}