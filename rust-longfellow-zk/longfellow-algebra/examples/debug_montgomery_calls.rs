use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Testing if montgomery_reduce_wide is called\n");
    
    let one = Fp128::from_u64(1);
    println!("Created one = {:?}", one);
    
    // This should call to_montgomery which should call mul_assign which should call montgomery_reduce_wide
    let regular_one = longfellow_algebra::nat::Nat::<2>::from_u64(1);
    let montgomery_one = Fp128::to_montgomery(regular_one);
    println!("to_montgomery(1) = {:?}", montgomery_one);
}