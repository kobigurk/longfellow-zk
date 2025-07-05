use longfellow_algebra::{Fp128, traits::Field};
use longfellow_ligero::{ConstraintSystem, LigeroProver, LigeroVerifier, LigeroParams, LigeroInstance};
use longfellow_core::Result;
use rand::thread_rng;

fn main() -> Result<()> {
    println!("Testing Ligero proof with larger constraint system\n");
    
    // Create a larger constraint system
    // We'll create a system that proves knowledge of values a, b, c such that:
    // a * b = c
    // a + b = 10
    // c = 21
    // Solution: a = 3, b = 7, c = 21
    
    const NUM_VARS: usize = 64; // Make it large enough for FFT
    let mut cs = ConstraintSystem::<Fp128>::new(NUM_VARS);
    
    // Variable indices
    let a = 0;
    let b = 1;
    let c = 2;
    let one = 3;
    
    // Add constraint: a * b = c
    cs.add_quadratic_constraint(a, b, c);
    
    // Add constraint: a + b - 10 = 0
    // Linear constraint: 1*a + 1*b + (-10)*one = 0
    let ten = Fp128::from_u64(10);
    cs.add_linear_constraint(
        vec![(a, Fp128::one()), (b, Fp128::one()), (one, -ten)],
        Fp128::zero()
    );
    
    // Add constraint: c - 21 = 0
    // Linear constraint: 1*c + (-21)*one = 0
    let twenty_one = Fp128::from_u64(21);
    cs.add_linear_constraint(
        vec![(c, Fp128::one()), (one, -twenty_one)],
        Fp128::zero()
    );
    
    // Create witness
    let mut witness = vec![Fp128::zero(); NUM_VARS];
    witness[a] = Fp128::from_u64(3);
    witness[b] = Fp128::from_u64(7);
    witness[c] = Fp128::from_u64(21);
    witness[one] = Fp128::one();
    
    // Fill rest with random values (or zeros)
    for i in 4..NUM_VARS {
        witness[i] = Fp128::from_u64(i as u64);
    }
    
    // Check if witness satisfies constraints
    let satisfied = cs.is_satisfied(&witness)?;
    println!("Witness satisfies constraints: {}", satisfied);
    
    // Create Ligero instance with 80-bit security
    let params = LigeroParams::security_80();
    println!("\nLigero parameters:");
    println!("  Block size: {}", params.block_size);
    println!("  Extension factor: {}", params.extension_factor);
    println!("  Witnesses: {}", NUM_VARS);
    
    let prover_instance = LigeroInstance::new(params.clone(), cs.clone())?;
    let verifier_instance = LigeroInstance::new(params, cs)?;
    
    // Create prover
    let prover = LigeroProver::new(prover_instance)?;
    
    // Generate proof
    println!("\nGenerating proof...");
    let mut rng = thread_rng();
    let proof = prover.prove(&witness, &mut rng)?;
    println!("Proof generated successfully!");
    println!("  Column roots: {}", proof.column_roots.len());
    println!("  LDT responses: {}", proof.ldt_responses.len());
    
    // Create verifier  
    let verifier = LigeroVerifier::new(verifier_instance)?;
    
    // Verify proof
    println!("\nVerifying proof...");
    let valid = verifier.verify(&proof)?;
    println!("Proof valid: {}", valid);
    
    Ok(())
}