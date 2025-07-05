use longfellow_algebra::nat::Nat;
use longfellow_algebra::Fp128;

fn main() {
    println!("Testing conditional subtraction\n");
    
    // The modulus
    let modulus = Fp128::MODULUS;
    println!("Modulus = {:?}", modulus);
    
    // Test case: a value that should have modulus subtracted
    // After Montgomery reduction of 1*1, we get 0xfffff00000000001
    let mut test1 = Nat::<2>::new([0x0000000000000001, 0xfffff00000000000]);
    println!("\nTest 1: {:?}", test1);
    
    // Check if >= modulus
    if test1 >= modulus {
        println!("✅ test1 >= modulus (should subtract)");
    } else {
        println!("❌ test1 < modulus (won't subtract)");
    }
    
    // Try subtraction
    let borrow = test1.sub_with_borrow(&modulus);
    println!("After subtraction: {:?}", test1);
    println!("Borrow: {}", borrow);
    
    if borrow == 0 {
        println!("✅ No borrow (subtraction was valid)");
    } else {
        println!("❌ Borrow occurred (subtraction was invalid)");
    }
    
    // Expected result after subtraction: 0x0000000000000001
    let expected = Nat::<2>::from_u64(1);
    if test1 == expected {
        println!("✅ Result equals 1");
    } else {
        println!("❌ Result does not equal 1");
    }
    
    // Let's also test the comparison directly
    println!("\n\nDirect comparison test:");
    let a = Nat::<2>::new([0x0000000000000001, 0xfffff00000000000]);
    let b = Nat::<2>::new([0x0000000000000001, 0xfffff00000000000]);
    
    if a == b {
        println!("✅ Values are equal");
    } else {
        println!("❌ Values are not equal");
    }
    
    // The issue might be in the conditional subtraction logic
    // in montgomery_reduce_wide
}