use longfellow_algebra::{Fp128, traits::Field};
use longfellow_ligero::{ConstraintSystem, LigeroProver, LigeroVerifier, LigeroParams, LigeroInstance};
use longfellow_core::Result;
use rand::thread_rng;

fn main() -> Result<()> {
    println!("Testing simple Ligero proof with Fp128\n");
    
    // Create a simple constraint system
    // Let's prove knowledge of x such that x^2 = 4
    // We'll have 3 witnesses: x=2, x_squared=4, one=1
    let mut cs = ConstraintSystem::<Fp128>::new(3);
    
    // Witness indices
    let x = 0;
    let x_squared = 1;
    let one = 2;
    
    // Add constraint: x * x = x_squared
    cs.add_quadratic_constraint(x, x, x_squared);
    
    // Add constraint: x_squared - 4 = 0
    // This is a linear constraint: 1*x_squared + (-4)*one = 0
    let four = Fp128::from_u64(4);
    cs.add_linear_constraint(
        vec![(x_squared, Fp128::one()), (one, -four)],
        Fp128::zero()
    );
    
    // Create witness (x = 2, x_squared = 4, one = 1)
    let two = Fp128::from_u64(2);
    let witness = vec![two, four, Fp128::one()];
    
    // Check if witness satisfies constraints
    let satisfied = cs.is_satisfied(&witness)?;
    println!("Witness satisfies constraints: {}", satisfied);
    
    // Create Ligero instance with 80-bit security
    let params = LigeroParams::security_80();
    let prover_instance = LigeroInstance::new(params.clone(), cs.clone())?;
    let verifier_instance = LigeroInstance::new(params, cs)?;
    
    // Create prover
    let prover = LigeroProver::new(prover_instance)?;
    
    // Generate proof
    println!("\nGenerating proof...");
    let mut rng = thread_rng();
    let proof = prover.prove(&witness, &mut rng)?;
    println!("Proof generated successfully!");
    
    // Create verifier  
    let verifier = LigeroVerifier::new(verifier_instance)?;
    
    // Verify proof
    println!("\nVerifying proof...");
    let valid = verifier.verify(&proof)?;
    println!("Proof valid: {}", valid);
    
    Ok(())
}