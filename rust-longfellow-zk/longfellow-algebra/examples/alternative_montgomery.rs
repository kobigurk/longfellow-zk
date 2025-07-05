use longfellow_algebra::{Fp128, Field};
use longfellow_algebra::nat::{Nat, Limb, mul_wide, add_with_carry};

// Alternative Montgomery multiplication implementation
fn alt_montgomery_mul(a: &Nat<2>, b: &Nat<2>) -> Nat<2> {
    // Constants for Fp128
    let p = Nat::<2>::new([0xfffff000000000000000000000000001u128 as u64, (0xfffff000000000000000000000000001u128 >> 64) as u64]);
    let inv = 0xFFFFFFFFFFFFFFFF; // INV constant
    
    // Textbook Montgomery multiplication algorithm
    // Input: a, b in Montgomery form
    // Output: a * b * R^(-1) mod p in Montgomery form
    
    // Step 1: Compute the full product a * b
    let mut result = [0u64; 4]; // 4 limbs for 2*N result
    
    // School multiplication
    for i in 0..2 {
        let mut carry = 0u64;
        for j in 0..2 {
            let (lo, hi) = mul_wide(a.limbs[i], b.limbs[j]);
            let (sum1, c1) = add_with_carry(result[i + j], lo, carry);
            result[i + j] = sum1;
            carry = hi.wrapping_add(c1);
        }
        result[i + 2] = result[i + 2].wrapping_add(carry);
    }
    
    // Step 2: Montgomery reduction (REDC)
    for i in 0..2 {
        let m = result[i].wrapping_mul(inv);
        
        let mut carry = 0u64;
        for j in 0..2 {
            let (lo, hi) = mul_wide(m, p.limbs[j]);
            let (sum1, c1) = add_with_carry(result[i + j], lo, carry);
            result[i + j] = sum1;
            carry = hi.wrapping_add(c1);
        }
        
        // Propagate carry through remaining limbs
        let mut k = i + 2;
        while k < 4 && carry > 0 {
            let (sum, c) = add_with_carry(result[k], 0, carry);
            result[k] = sum;
            carry = c;
            k += 1;
        }
    }
    
    // Extract the result from upper half
    let mut final_result = Nat::<2>::new([result[2], result[3]]);
    
    // Conditional subtraction
    if final_result >= p {
        final_result.sub_with_borrow(&p);
    }
    
    final_result
}

fn main() {
    println!("Testing alternative Montgomery multiplication\n");
    
    let omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_bytes).unwrap();
    
    // Test a simple multiplication first
    let two = Fp128::from_u64(2);
    let three = Fp128::from_u64(3);
    let six_std = two * three;
    
    // Test with alternative algorithm
    let two_nat = two.from_montgomery();
    let three_nat = three.from_montgomery();
    
    // Convert to Montgomery form manually
    let r2 = Nat::<2>::new([0xfffeffffefffff01u64, 0x000fdffffeffffef]);
    let two_mont = alt_montgomery_mul(&two_nat, &r2);
    let three_mont = alt_montgomery_mul(&three_nat, &r2);
    let six_alt_mont = alt_montgomery_mul(&two_mont, &three_mont);
    let six_alt = alt_montgomery_mul(&six_alt_mont, &Nat::<2>::new([1, 0])); // Multiply by 1 to get regular form
    
    println!("Standard 2 * 3 = {:?}", six_std);
    println!("Alternative 2 * 3 regular form = {:?}", six_alt);
    
    // Now test with omega
    println!("\nTesting omega multiplication:");
    
    let omega_regular = omega.from_montgomery();
    let omega_squared_std = omega * omega;
    
    // Test with alternative Montgomery
    let omega_mont = alt_montgomery_mul(&omega_regular, &r2); // Convert to Montgomery
    let omega_squared_alt_mont = alt_montgomery_mul(&omega_mont, &omega_mont);
    
    println!("Standard omega^2 = {:?}", omega_squared_std);
    println!("Alternative omega^2 Montgomery = {:?}", omega_squared_alt_mont);
    
    // Compare regular forms
    let omega_squared_std_regular = omega_squared_std.from_montgomery();
    
    if omega_squared_std_regular == omega_squared_alt_mont {
        println!("✓ Both algorithms give same result for omega^2");
    } else {
        println!("✗ Algorithms differ for omega^2");
        println!("  This suggests a bug in one of the implementations");
    }
}