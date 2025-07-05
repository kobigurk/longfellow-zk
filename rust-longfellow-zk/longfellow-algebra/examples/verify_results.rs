use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Verifying multiplication results\n");
    
    // Test cases that fail
    let test_cases = vec![
        (6, "6^(-1)"),
        (-1, "-1"),
    ];
    
    for (val, name) in test_cases {
        let a = if val > 0 {
            Fp128::from_u64(val as u64)
        } else {
            -Fp128::one()
        };
        
        let b = if val > 0 {
            a.invert().unwrap()
        } else {
            a  // For -1, we square it
        };
        
        let product = a * b;
        let expected = Fp128::one();
        
        println!("{} * {} = {:?}", name, name, product);
        println!("Expected: {:?}", expected);
        
        if product == expected {
            println!("✅ Correct\n");
        } else {
            println!("❌ Wrong");
            
            // Check if it's zero
            if product == Fp128::zero() {
                println!("Got zero instead of one\n");
            }
        }
    }
    
    // Let's also test some values that work
    println!("Testing values that work:");
    for i in vec![1, 2, 3, 4, 5, 7, 8, 9] {
        let a = Fp128::from_u64(i);
        let a_inv = a.invert().unwrap();
        let prod = a * a_inv;
        
        if prod == Fp128::one() {
            println!("✅ {} * {}^(-1) = 1", i, i);
        } else {
            println!("❌ {} * {}^(-1) != 1", i, i);
        }
    }
    
    // Pattern: 6 and -1 fail, others work
    // What do 6 and -1 have in common?
    // 6 * 6^(-1) should give 1, but gives 0
    // (-1) * (-1) should give 1, but gives 0
    
    // This suggests that certain products are giving p instead of 1
    // And p ≡ 0 in the field
}