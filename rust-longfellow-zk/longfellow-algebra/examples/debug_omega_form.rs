use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging Montgomery form conversion\n");
    
    // The value we computed in Python: 124138436495952958347847942047415585016
    // In hex: 0x5d64317630d17e980147ca53fd9ec8f8
    let omega_32_bytes = [
        0xf8, 0xc8, 0x9e, 0xfd, 0x53, 0xca, 0x47, 0x01,
        0x98, 0x7e, 0xd1, 0x30, 0x76, 0x31, 0x64, 0x5d,
    ];
    
    let omega_32 = Fp128::from_bytes_le(&omega_32_bytes).unwrap();
    println!("Loaded omega_32 = {:?}", omega_32);
    
    // Check what it looks like in non-Montgomery form
    let omega_32_regular = omega_32.from_montgomery();
    println!("omega_32 non-Montgomery = {:?}", omega_32_regular);
    
    // Check if from_montgomery is working at all
    let test_value = Fp128::from_u64(5);
    println!("\nTesting from_montgomery function:");
    println!("test_value = {:?}", test_value);
    println!("test_value.from_montgomery() = {:?}", test_value.from_montgomery());
    
    // The issue: omega_32_regular is a Nat<2>, not an Fp128
    // Let's create an Fp128 from the regular form 
    println!("\nCreating Fp128 from regular form:");
    let omega_32_from_regular = Fp128::to_montgomery(omega_32_regular);
    println!("omega_32_from_regular = {:?}", omega_32_from_regular);
    
    // Test this value as a root of unity
    println!("\nTesting omega_32_from_regular as root of unity:");
    let mut current = omega_32_from_regular;
    for i in 1..=10 {
        if current == Fp128::one() {
            println!("omega_from_regular^{} = 1 (order = {})", i, i);
            break;
        }
        println!("omega_from_regular^{} = {:?}", i, current);
        current = current * omega_32_from_regular;
    }
}