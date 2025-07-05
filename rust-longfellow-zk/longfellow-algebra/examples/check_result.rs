use longfellow_algebra::nat::Nat;

fn main() {
    // Result from trace_mont
    let mut result = Nat::<2>::new([0xffffe00001000000, 0x00000fffff000000]);
    println!("Result from from_montgomery(R): {:?}", result);
    
    // Modulus
    let modulus = Nat::<2>::new([0x0000000000000001, 0xfffff00000000000]);
    println!("Modulus: {:?}", modulus);
    
    // Compare
    if result >= modulus {
        println!("\nResult >= modulus, needs reduction");
        let borrow = result.sub_with_borrow(&modulus);
        println!("After subtraction: {:?} (borrow={})", result, borrow);
    } else {
        println!("\nResult < modulus, no reduction needed");
    }
    
    // Convert to bytes
    let bytes = result.to_bytes_le();
    println!("\nAs bytes: {:?}", bytes);
    
    // Expected: should be 1
    println!("Expected: [1]");
}