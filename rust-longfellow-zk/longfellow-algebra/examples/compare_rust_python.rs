use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Comparing Rust vs Python calculations\n");
    
    // Load the omega_32 value
    let omega_32_bytes = [
        0xf8, 0xc8, 0x9e, 0xfd, 0x53, 0xca, 0x47, 0x01,
        0x98, 0x7e, 0xd1, 0x30, 0x76, 0x31, 0x64, 0x5d,
    ];
    
    let omega = Fp128::from_bytes_le(&omega_32_bytes).unwrap();
    println!("omega = {:?}", omega);
    
    // Test small powers and compare with Python results
    let omega_2 = omega * omega;
    let omega_4 = omega_2 * omega_2;
    let omega_8 = omega_4 * omega_4;
    
    println!("\nRust results:");
    println!("omega^2 = {:?}", omega_2);
    println!("omega^4 = {:?}", omega_4);
    println!("omega^8 = {:?}", omega_8);
    
    // Python results (from previous calculation):
    // omega^2 = 244871581368291671471330542422084298591
    // omega^4 = 120136388251989084799145260554337211523  
    // omega^8 = 213852945940147393302189921459325645862
    
    // Convert Python results to Rust for comparison
    let python_omega_2_bytes = 244871581368291671471330542422084298591u128.to_le_bytes();
    let python_omega_4_bytes = 120136388251989084799145260554337211523u128.to_le_bytes();
    let python_omega_8_bytes = 213852945940147393302189921459325645862u128.to_le_bytes();
    
    let python_omega_2 = Fp128::from_bytes_le(&python_omega_2_bytes).unwrap();
    let python_omega_4 = Fp128::from_bytes_le(&python_omega_4_bytes).unwrap();
    let python_omega_8 = Fp128::from_bytes_le(&python_omega_8_bytes).unwrap();
    
    println!("\nPython results converted to Rust:");
    println!("omega^2 = {:?}", python_omega_2);
    println!("omega^4 = {:?}", python_omega_4);
    println!("omega^8 = {:?}", python_omega_8);
    
    println!("\nComparisons:");
    println!("omega^2 match? {}", omega_2 == python_omega_2);
    println!("omega^4 match? {}", omega_4 == python_omega_4);
    println!("omega^8 match? {}", omega_8 == python_omega_8);
    
    // If even small powers don't match, the issue is in basic multiplication
    // If small powers match but large ones don't, the issue is in pow()
    
    // Let's also check the raw values in regular form
    println!("\nRaw regular form values:");
    println!("Rust omega^2 regular = {:?}", omega_2.from_montgomery());
    println!("Python omega^2 regular = {:?}", python_omega_2.from_montgomery());
    
    // Test if the issue is in the loading of Python values
    println!("\nTesting Python value loading:");
    
    // Create the Python omega^2 value manually in parts
    let high_part = 244871581368291671471330542422084298591u128 >> 64;
    let low_part = 244871581368291671471330542422084298591u128 & 0xFFFFFFFFFFFFFFFF;
    
    println!("Python omega^2 = {}", 244871581368291671471330542422084298591u128);
    println!("High 64 bits = {}", high_part);
    println!("Low 64 bits = {}", low_part);
    
    // Manually construct as Nat
    let manual_nat = longfellow_algebra::nat::Nat::<2>::new([low_part as u64, high_part as u64]);
    let manual_fp128 = Fp128::to_montgomery(manual_nat);
    
    println!("Manual construction = {:?}", manual_fp128);
    println!("Matches from_bytes_le? {}", manual_fp128 == python_omega_2);
}