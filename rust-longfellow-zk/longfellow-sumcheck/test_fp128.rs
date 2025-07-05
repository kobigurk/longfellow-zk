use longfellow_algebra::field::fp128::Fp128;
use longfellow_algebra::traits::Field;

fn main() {
    // Test 1: Check Fp128::one() representation
    let one = Fp128::one();
    println!("Fp128::one() internal value: {:?}", one);
    println!("Fp128::one() as bytes: {:02x?}", one.to_bytes_le());
    
    // Test 2: Check arithmetic with from_u64
    let w0 = Fp128::from_u64(5);
    let w1 = Fp128::from_u64(7);
    let w2 = Fp128::from_u64(12);
    
    println!("\nw0 = Fp128::from_u64(5): {:?}", w0);
    println!("w0 as bytes: {:02x?}", w0.to_bytes_le());
    
    println!("\nw1 = Fp128::from_u64(7): {:?}", w1);
    println!("w1 as bytes: {:02x?}", w1.to_bytes_le());
    
    println!("\nw2 = Fp128::from_u64(12): {:?}", w2);
    println!("w2 as bytes: {:02x?}", w2.to_bytes_le());
    
    let sum = w0 + w1;
    println!("\nw0 + w1 = {:?}", sum);
    println!("w0 + w1 as bytes: {:02x?}", sum.to_bytes_le());
    
    println!("\nDoes w0 + w1 == w2? {}", sum == w2);
    
    // Test 3: Check modulus and R values
    println!("\nModulus: 0x{:032x}{:016x}", Fp128::MODULUS.limbs[1], Fp128::MODULUS.limbs[0]);
    println!("R (Montgomery constant): 0x{:016x}{:016x}", Fp128::R.limbs[1], Fp128::R.limbs[0]);
    println!("R2: 0x{:016x}{:016x}", Fp128::R2.limbs[1], Fp128::R2.limbs[0]);
    
    // Test 4: Direct arithmetic verification
    let five = Fp128::from_u64(5);
    let seven = Fp128::from_u64(7);
    let twelve = Fp128::from_u64(12);
    let thirty_five = Fp128::from_u64(35);
    
    println!("\nDirect arithmetic tests:");
    println!("5 + 7 = {:?}", five + seven);
    println!("5 + 7 == 12? {}", five + seven == twelve);
    println!("5 * 7 = {:?}", five * seven);
    println!("5 * 7 == 35? {}", five * seven == thirty_five);
}