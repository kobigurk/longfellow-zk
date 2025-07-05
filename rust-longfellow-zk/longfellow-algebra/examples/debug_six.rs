use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging inversion of 6\n");
    
    let six = Fp128::from_u64(6);
    let six_inv = six.invert().unwrap();
    let product = six * six_inv;
    let one = Fp128::one();
    
    println!("6 = {:?}", six);
    println!("6^(-1) = {:?}", six_inv);
    println!("6 * 6^(-1) = {:?}", product);
    println!("Expected 1 = {:?}", one);
    
    if product == one {
        println!("\n✅ 6 * 6^(-1) = 1");
    } else {
        println!("\n❌ 6 * 6^(-1) != 1");
        
        // Let's check what we got
        let prod_bytes = product.to_bytes_le();
        println!("\nProduct bytes: {:?}", prod_bytes);
        
        // Also test if 6 has some special property
        // 6 = 2 * 3, so let's test those
        let two = Fp128::from_u64(2);
        let three = Fp128::from_u64(3);
        let two_inv = two.invert().unwrap();
        let three_inv = three.invert().unwrap();
        
        println!("\nTesting components:");
        let prod2 = two * two_inv;
        if prod2 == one {
            println!("✅ 2 * 2^(-1) = 1");
        } else {
            println!("❌ 2 * 2^(-1) != 1");
        }
        
        let prod3 = three * three_inv;
        if prod3 == one {
            println!("✅ 3 * 3^(-1) = 1");
        } else {
            println!("❌ 3 * 3^(-1) != 1");
        }
        
        // Check if 6^(-1) = 2^(-1) * 3^(-1)
        let expected_six_inv = two_inv * three_inv;
        if six_inv == expected_six_inv {
            println!("\n✅ 6^(-1) = 2^(-1) * 3^(-1)");
        } else {
            println!("\n❌ 6^(-1) != 2^(-1) * 3^(-1)");
            println!("Computed 6^(-1) = {:?}", six_inv);
            println!("2^(-1) * 3^(-1) = {:?}", expected_six_inv);
        }
    }
    
    // Let's also test a range around 6
    println!("\n\nTesting inversions around 6:");
    for i in 4u64..9 {
        let a = Fp128::from_u64(i);
        let a_inv = a.invert().unwrap();
        let prod = a * a_inv;
        if prod == one {
            println!("✅ {} * {}^(-1) = 1", i, i);
        } else {
            println!("❌ {} * {}^(-1) != 1", i, i);
        }
    }
}