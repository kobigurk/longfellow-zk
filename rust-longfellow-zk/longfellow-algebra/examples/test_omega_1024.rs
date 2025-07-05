use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing omega^1024 specifically\n");
    
    let omega_bytes = [
        0x36, 0xd8, 0x3d, 0x41, 0x23, 0x6f, 0xff, 0x42,
        0xb4, 0x8e, 0xd3, 0x48, 0x9f, 0x83, 0x19, 0x7c,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_bytes).unwrap();
    
    // Test omega^1024 using pow function
    let omega_1024_pow = omega.pow(&[1024]);
    let omega_1024_regular = omega_1024_pow.from_montgomery();
    let rust_1024 = (omega_1024_regular.limbs[1] as u128) << 64 | omega_1024_regular.limbs[0] as u128;
    
    println!("Rust omega^1024 (pow) = 0x{:032x}", rust_1024);
    
    // Test omega^1024 by repeated squaring manually
    let mut current = omega;
    for i in 0..10 {  // 2^10 = 1024
        current = current * current;
        let current_regular = current.from_montgomery();
        let current_val = (current_regular.limbs[1] as u128) << 64 | current_regular.limbs[0] as u128;
        println!("omega^{} = 0x{:032x}", 1 << (i+1), current_val);
    }
    
    let rust_1024_manual = (current.from_montgomery().limbs[1] as u128) << 64 | current.from_montgomery().limbs[0] as u128;
    
    println!("\nRust omega^1024 (manual) = 0x{:032x}", rust_1024_manual);
    println!("Python omega^1024 = 0x23a31048cd28281a2910c406aba20719");
    
    println!("\nComparison:");
    println!("pow vs manual: {}", rust_1024 == rust_1024_manual);
    println!("Rust vs Python: {}", rust_1024 == 0x23a31048cd28281a2910c406aba20719u128);
}