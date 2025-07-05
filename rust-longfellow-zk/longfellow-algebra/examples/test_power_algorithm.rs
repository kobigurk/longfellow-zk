use longfellow_algebra::{Fp128, Field};

// Reference implementation using repeated squaring
fn reference_pow(base: &Fp128, mut exp: u64) -> Fp128 {
    if exp == 0 {
        return Fp128::one();
    }
    
    let mut result = Fp128::one();
    let mut current_base = *base;
    
    while exp > 0 {
        if exp & 1 == 1 {
            result *= &current_base;
        }
        current_base = current_base.square();
        exp >>= 1;
    }
    
    result
}

// Alternative implementation using binary method
fn binary_pow(base: &Fp128, exp: u64) -> Fp128 {
    if exp == 0 {
        return Fp128::one();
    }
    if exp == 1 {
        return *base;
    }
    
    let half = binary_pow(base, exp / 2);
    let half_squared = half * half;
    
    if exp % 2 == 0 {
        half_squared
    } else {
        half_squared * base
    }
}

fn main() {
    println!("Testing different power algorithms\n");
    
    let omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_bytes).unwrap();
    
    // Test small powers first
    println!("Testing small powers:");
    for exp in [2, 4, 8, 16, 32] {
        let rust_result = omega.pow(&[exp]);
        let ref_result = reference_pow(&omega, exp);
        let bin_result = binary_pow(&omega, exp);
        
        println!("omega^{}: Rust={:?}", exp, rust_result);
        println!("        Ref={:?}", ref_result);
        println!("        Bin={:?}", bin_result);
        
        if rust_result == ref_result && ref_result == bin_result {
            println!("        ✓ All match");
        } else {
            println!("        ✗ MISMATCH!");
        }
        println!();
    }
    
    // Test progressively larger powers to find divergence point
    println!("Testing larger powers:");
    let large_exponents = [64, 128, 256, 512, 1024, 2048, 4096];
    
    for &exp in &large_exponents {
        let rust_result = omega.pow(&[exp]);
        let ref_result = reference_pow(&omega, exp);
        
        println!("omega^{}: Match = {}", exp, rust_result == ref_result);
        
        if rust_result != ref_result {
            println!("  ✗ DIVERGENCE FOUND at omega^{}", exp);
            println!("  Rust: {:?}", rust_result);
            println!("  Ref:  {:?}", ref_result);
            break;
        }
    }
    
    // Test the problematic 2^31 power specifically
    println!("\nTesting omega^(2^31):");
    let exp_31 = 1u64 << 31;
    println!("2^31 = {}", exp_31);
    
    let rust_result = omega.pow(&[exp_31]);
    let ref_result = reference_pow(&omega, exp_31);
    
    println!("Rust omega^(2^31) = {:?}", rust_result);
    println!("Ref  omega^(2^31) = {:?}", ref_result);
    
    if rust_result == ref_result {
        println!("✓ Both algorithms give same result");
        
        // But is it the mathematically correct result?
        let minus_one = -Fp128::one();
        if rust_result == minus_one {
            println!("✓ Result equals -1 (mathematically correct!)");
        } else {
            println!("✗ Result ≠ -1 (both algorithms have same bug)");
            println!("Expected -1 = {:?}", minus_one);
        }
    } else {
        println!("✗ Algorithms give different results");
    }
}