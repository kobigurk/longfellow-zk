use longfellow_algebra::{Fp128, Field};
use longfellow_algebra::nat::Nat;

fn main() {
    println!("Debugging Montgomery form ONE...\n");
    
    // Get the actual internal representation
    let one = Fp128::one();
    
    // ONE should be R in Montgomery form
    println!("Expected R  = {:?}", Fp128::R);
    
    // Let's manually trace from_montgomery for R
    println!("\nManual trace of from_montgomery(R):");
    
    let r_value = Fp128::R;
    let modulus = Fp128::MODULUS;
    let inv = Fp128::INV;
    
    println!("Input: R = {:?}", r_value);
    println!("Modulus = {:?}", modulus);
    println!("INV = 0x{:016x}", inv);
    
    // Create 2N-limb array
    let mut t = vec![0u64; 4];
    t[0] = r_value.limbs[0];
    t[1] = r_value.limbs[1];
    println!("\nInitial t = [{:016x}, {:016x}, {:016x}, {:016x}]", t[0], t[1], t[2], t[3]);
    
    // First iteration (i=0)
    let k0 = t[0].wrapping_mul(inv);
    println!("\ni=0: k = 0x{:016x} * 0x{:016x} = 0x{:016x}", t[0], inv, k0);
    
    // We should have k0 * modulus.limbs[0] + t[0] = 0 (mod 2^64)
    let (lo0, hi0) = longfellow_algebra::nat::mul_wide(k0, modulus.limbs[0]);
    println!("k * modulus[0] = 0x{:016x} * 0x{:016x} = (0x{:016x}, 0x{:016x})", k0, modulus.limbs[0], lo0, hi0);
    
    let sum0 = t[0].wrapping_add(lo0);
    println!("t[0] + lo = 0x{:016x} + 0x{:016x} = 0x{:016x}", t[0], lo0, sum0);
    
    // Direct calculation check
    // R = 0x00000fffffffffff
    // p = 0xfffff000000000000000000000000001
    // R * R^(-1) mod p should be 1
    
    // For Fp128, p = 2^128 - 2^108 + 1
    // R = 2^108 - 1 (this is 2^128 mod p)
    
    println!("\n2^108 - 1 = 0x{:032x}", (1u128 << 108) - 1);
}