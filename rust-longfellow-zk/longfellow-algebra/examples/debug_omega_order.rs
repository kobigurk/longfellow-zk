use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Finding the actual order of omega_32\n");
    
    // Load the omega_32 bytes
    let omega_32_bytes = [
        0xf8, 0xc8, 0x9e, 0xfd, 0x53, 0xca, 0x47, 0x01,
        0x98, 0x7e, 0xd1, 0x30, 0x76, 0x31, 0x64, 0x5d,
    ];
    
    let omega_32 = Fp128::from_bytes_le(&omega_32_bytes).unwrap();
    println!("omega_32 = {:?}", omega_32);
    
    // Check if omega_32^(2^32) is actually 1
    println!("\nTesting specific powers of 2:");
    
    // We saw that omega_32^(2^32) ≠ 1, so let's continue checking higher powers
    let mut current = omega_32;
    
    // Fast computation of high powers by repeated squaring  
    for k in 32..=50 {
        let exponent = 1u64 << k; // 2^k
        if k <= 40 { // Only for reasonable exponent sizes
            current = current.square(); // This gives us omega_32^(2^k)
            println!("omega_32^(2^{}) = {:?}", k, current);
            
            if current == Fp128::one() {
                println!("✓ Found: omega_32 has order 2^{}", k);
                return;
            }
        } else {
            break;
        }
    }
    
    // Let's check if the value has order related to the field structure
    // For p = 2^128 - 2^108 + 1, we have p - 1 = 2^108 * (2^20 - 1)
    // The maximum possible order for any element is p - 1
    
    println!("\nChecking if it's related to the field structure:");
    println!("p - 1 = 2^108 * (2^20 - 1)");
    println!("2^20 - 1 = {}", (1u64 << 20) - 1); // = 1048575
    
    // Let's test if omega_32^(2^20 - 1) gives us something meaningful
    let test_exp = (1u64 << 20) - 1; // 2^20 - 1
    let test_result = omega_32.pow(&[test_exp]);
    println!("omega_32^(2^20 - 1) = omega_32^{} = {:?}", test_exp, test_result);
    
    // And test omega_32^(2^108)
    // This is too large to compute directly, but let's see what happens with smaller powers
    
    // Actually, let's check if there's an error in our computation by testing
    // a simpler root of unity. Let's compute a primitive square root of unity (-1)
    println!("\nTesting -1 as a square root of unity:");
    let minus_one = -Fp128::one();
    let minus_one_squared = minus_one.square();
    println!("-1 = {:?}", minus_one);
    println!("(-1)^2 = {:?}", minus_one_squared);
    
    if minus_one_squared == Fp128::one() {
        println!("✓ -1 is a square root of unity");
    } else {
        println!("✗ -1 is NOT a square root of unity - this is a serious problem!");
    }
    
    // Let's also test if our omega value has ANY reasonable order
    println!("\nTesting if omega has order dividing (p-1):");
    
    // Test if omega^(p-1) = 1 (Fermat's little theorem)
    // We can't compute p-1 exactly, but we can test smaller divisors
    
    // Let's try omega^(2^16) for a reasonable test
    let test_power = omega_32.pow(&[1u64 << 16]);
    println!("omega_32^(2^16) = {:?}", test_power);
    
    // Let's try omega^(2^20)
    let test_power_20 = omega_32.pow(&[1u64 << 20]);
    println!("omega_32^(2^20) = {:?}", test_power_20);
}