use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing p-1 in the field\n");
    
    // p - 1 should be -1 in the field
    let minus_one = -Fp128::one();
    println!("-1 = {:?}", minus_one);
    
    // Let's check what -1 * -1 gives us
    let minus_one_squared = minus_one * minus_one;
    println!("(-1) * (-1) = {:?}", minus_one_squared);
    
    if minus_one_squared == Fp128::one() {
        println!("✅ (-1)^2 = 1");
    } else {
        println!("❌ (-1)^2 != 1");
    }
    
    // Now let's check 6 * ((p-1)/6)
    // Since p ≡ 1 (mod 6), we have p = 6k + 1
    // So k = (p-1)/6
    // And 6 * k = p - 1 ≡ -1 (mod p)
    
    let six = Fp128::from_u64(6);
    let k = minus_one * six.invert().unwrap(); // k = -1/6 = (p-1)/6
    
    println!("\nk = (p-1)/6 = {:?}", k);
    
    let six_times_k = six * k;
    println!("6 * k = {:?}", six_times_k);
    
    if six_times_k == minus_one {
        println!("✅ 6 * (p-1)/6 = p-1 ≡ -1");
    } else {
        println!("❌ 6 * (p-1)/6 != -1");
    }
    
    // The issue might be that when we compute 6 * 6^(-1),
    // we're somehow getting p instead of 1
    // And p ≡ 0 in the field
    
    // Let's verify this
    println!("\n\nChecking if issue is p vs 1:");
    
    // Create a value that's p more than it should be
    let one_plus_p = Fp128::from_u64(1); // This should just be 1
    println!("1 = {:?}", one_plus_p);
    
    // If we had p + 1, it would reduce to 1
    // But if we have p, it reduces to 0
    
    // Let's check zero
    let zero = Fp128::zero();
    println!("0 = {:?}", zero);
    
    // The pattern we see with 6 * 6^(-1) = 0 suggests
    // the multiplication is producing p, not 1
}