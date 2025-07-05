use longfellow_algebra::{Fp128, Field};

fn main() {
    println!("Debugging raw memory of ONE\n");
    
    let one = Fp128::one();
    
    // Get the memory representation directly
    let one_ptr = &one as *const Fp128 as *const u8;
    let memory_slice = unsafe { std::slice::from_raw_parts(one_ptr, std::mem::size_of::<Fp128>()) };
    
    println!("Raw memory of one: {:02x?}", memory_slice);
    
    // Also check what the R constant looks like in memory
    use longfellow_algebra::field::fp_generic::FieldReduction;
    use longfellow_algebra::field::fp128::Fp128Reduce;
    
    let r_value = Fp128Reduce::R;
    let r_ptr = &r_value as *const _ as *const u8;
    let r_memory_slice = unsafe { std::slice::from_raw_parts(r_ptr, std::mem::size_of_val(&r_value)) };
    
    println!("Raw memory of R: {:02x?}", r_memory_slice);
    
    // Let's also create an Fp128 that should definitely contain R
    // by using transmute
    let manual_one: Fp128 = unsafe {
        std::mem::transmute(r_value)
    };
    
    println!("\nManually constructed Fp128 with R:");
    println!("manual_one = {:?}", manual_one);
    
    let manual_ptr = &manual_one as *const _ as *const u8;
    let manual_memory = unsafe { std::slice::from_raw_parts(manual_ptr, std::mem::size_of_val(&manual_one)) };
    println!("Raw memory of manual_one: {:02x?}", manual_memory);
    
    // Test if manual_one acts like ONE
    let two = Fp128::from_u64(2);
    let manual_times_two = manual_one * two;
    println!("\nmanual_one * two = {:?}", manual_times_two);
    
    // This should equal two if manual_one is the correct Montgomery representation of 1
}