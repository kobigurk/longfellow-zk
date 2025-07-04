use longfellow_ligero::*;
use longfellow_algebra::{Fp128, traits::Field};
use rand::rngs::OsRng;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[test]
fn test_basic_linear_constraints() {
    // Create system with 5 variables
    let mut cs = ConstraintSystem::<Fp128>::new(5);
    
    // Add constraints:
    // w[0] + w[1] = 5
    cs.add_linear_constraint(
        vec![(0, Fp128::one()), (1, Fp128::one())],
        Fp128::from(5),
    );
    
    // 2*w[2] + 3*w[3] = 13
    cs.add_linear_constraint(
        vec![(2, Fp128::from(2)), (3, Fp128::from(3))],
        Fp128::from(13),
    );
    
    // w[0] + w[2] + w[4] = 7
    cs.add_linear_constraint(
        vec![(0, Fp128::one()), (2, Fp128::one()), (4, Fp128::one())],
        Fp128::from(7),
    );
    
    // Create witness: [2, 3, 2, 3, 3]
    let witness = vec![
        Fp128::from(2),
        Fp128::from(3),
        Fp128::from(2),
        Fp128::from(3),
        Fp128::from(3),
    ];
    
    // Verify witness satisfies constraints
    assert!(cs.is_satisfied(&witness).unwrap());
    
    // Create instance and prove
    let params = LigeroParams::security_80();
    let instance = LigeroInstance::new(params, cs).unwrap();
    
    let prover = LigeroProver::new(instance.clone()).unwrap();
    let proof = prover.prove(&witness, &mut OsRng).unwrap();
    
    // Verify proof
    let verifier = LigeroVerifier::new(instance).unwrap();
    assert!(verifier.verify(&proof).unwrap());
}

#[test]
fn test_quadratic_constraints() {
    let mut cs = ConstraintSystem::<Fp128>::new(6);
    
    // Add quadratic constraints:
    // w[0] * w[1] = w[2]
    cs.add_quadratic_constraint(0, 1, 2);
    
    // w[3] * w[3] = w[4] (squaring)
    cs.add_quadratic_constraint(3, 3, 4);
    
    // w[2] * w[3] = w[5]
    cs.add_quadratic_constraint(2, 3, 5);
    
    // Witness: [3, 4, 12, 5, 25, 60]
    let witness = vec![
        Fp128::from(3),
        Fp128::from(4),
        Fp128::from(12),
        Fp128::from(5),
        Fp128::from(25),
        Fp128::from(60),
    ];
    
    assert!(cs.is_satisfied(&witness).unwrap());
    
    let params = LigeroParams::security_80();
    let instance = LigeroInstance::new(params, cs).unwrap();
    
    let prover = LigeroProver::new(instance.clone()).unwrap();
    let proof = prover.prove(&witness, &mut OsRng).unwrap();
    
    let verifier = LigeroVerifier::new(instance).unwrap();
    assert!(verifier.verify(&proof).unwrap());
}

#[test]
fn test_mixed_constraints() {
    let mut cs = ConstraintSystem::<Fp128>::new(4);
    
    // Linear: w[0] + w[1] = w[2]
    cs.add_linear_constraint(
        vec![(0, Fp128::one()), (1, Fp128::one()), (2, -Fp128::one())],
        Fp128::zero(),
    );
    
    // Quadratic: w[0] * w[1] = w[3]
    cs.add_quadratic_constraint(0, 1, 3);
    
    // Witness that satisfies both: [2, 3, 5, 6]
    let witness = vec![
        Fp128::from(2),
        Fp128::from(3),
        Fp128::from(5),
        Fp128::from(6),
    ];
    
    assert!(cs.is_satisfied(&witness).unwrap());
    
    let params = LigeroParams::security_128();
    let instance = LigeroInstance::new(params, cs).unwrap();
    
    let prover = LigeroProver::new(instance.clone()).unwrap();
    let proof = prover.prove(&witness, &mut ChaCha20Rng::seed_from_u64(42)).unwrap();
    
    let verifier = LigeroVerifier::new(instance).unwrap();
    assert!(verifier.verify(&proof).unwrap());
}

#[test]
fn test_large_instance() {
    let num_witnesses = 10000;
    let mut cs = ConstraintSystem::<Fp128>::new(num_witnesses);
    
    // Add many constraints
    for i in 0..1000 {
        // Linear constraints
        cs.add_linear_constraint(
            vec![
                (i, Fp128::one()),
                ((i + 1) % num_witnesses, Fp128::from(2)),
                ((i + 2) % num_witnesses, -Fp128::from(3)),
            ],
            Fp128::zero(),
        );
        
        // Quadratic constraints
        if i < 500 {
            cs.add_quadratic_constraint(
                i,
                (i + 1000) % num_witnesses,
                (i + 2000) % num_witnesses,
            );
        }
    }
    
    // Create a witness that satisfies the pattern
    let mut witness = vec![Fp128::one(); num_witnesses];
    for i in 0..num_witnesses {
        witness[i] = Fp128::from((i % 100) as u64);
    }
    
    let params = LigeroParams::security_128();
    let instance = LigeroInstance::new(params, cs).unwrap();
    
    let prover = LigeroProver::new(instance.clone()).unwrap();
    let proof = prover.prove(&witness, &mut ChaCha20Rng::seed_from_u64(42)).unwrap();
    
    let verifier = LigeroVerifier::new(instance).unwrap();
    assert!(verifier.verify(&proof).unwrap());
}

#[test]
fn test_soundness_invalid_witness() {
    let mut cs = ConstraintSystem::<Fp128>::new(3);
    
    // w[0] + w[1] = w[2]
    cs.add_linear_constraint(
        vec![(0, Fp128::one()), (1, Fp128::one()), (2, -Fp128::one())],
        Fp128::zero(),
    );
    
    // Valid witness would be [2, 3, 5]
    // Invalid witness: [2, 3, 6]
    let invalid_witness = vec![
        Fp128::from(2),
        Fp128::from(3),
        Fp128::from(6),
    ];
    
    assert!(!cs.is_satisfied(&invalid_witness).unwrap());
    
    let params = LigeroParams::security_80();
    let instance = LigeroInstance::new(params, cs).unwrap();
    
    let prover = LigeroProver::new(instance).unwrap();
    
    // Should fail to create proof with invalid witness
    assert!(prover.prove(&invalid_witness, &mut OsRng).is_err());
}

#[test]
fn test_proof_tampering() {
    let mut cs = ConstraintSystem::<Fp128>::new(3);
    cs.add_linear_constraint(
        vec![(0, Fp128::one()), (1, Fp128::one()), (2, -Fp128::one())],
        Fp128::zero(),
    );
    
    let witness = vec![Fp128::from(2), Fp128::from(3), Fp128::from(5)];
    
    let params = LigeroParams::security_80();
    let instance = LigeroInstance::new(params, cs).unwrap();
    
    let prover = LigeroProver::new(instance.clone()).unwrap();
    let mut proof = prover.prove(&witness, &mut OsRng).unwrap();
    
    // Tamper with the proof
    if !proof.linear_responses.is_empty() {
        proof.linear_responses[0] += Fp128::one();
    }
    
    let verifier = LigeroVerifier::new(instance).unwrap();
    assert!(!verifier.verify(&proof).unwrap());
}

#[test]
fn test_different_security_levels() {
    let mut cs = ConstraintSystem::<Fp128>::new(10);
    cs.add_linear_constraint(
        vec![(0, Fp128::one()), (1, Fp128::one())],
        Fp128::from(5),
    );
    
    let witness = vec![
        Fp128::from(2),
        Fp128::from(3),
        Fp128::zero(),
        Fp128::zero(),
        Fp128::zero(),
        Fp128::zero(),
        Fp128::zero(),
        Fp128::zero(),
        Fp128::zero(),
        Fp128::zero(),
    ];
    
    for &security_bits in &[80, 128, 256] {
        let params = LigeroParams::new(security_bits).unwrap();
        let instance = LigeroInstance::new(params, cs.clone()).unwrap();
        
        let prover = LigeroProver::new(instance.clone()).unwrap();
        let proof = prover.prove(&witness, &mut ChaCha20Rng::seed_from_u64(42)).unwrap();
        
        let verifier = LigeroVerifier::new(instance).unwrap();
        assert!(verifier.verify(&proof).unwrap());
        
        // Higher security should have more column openings
        match security_bits {
            80 => assert_eq!(proof.column_openings.len(), 80),
            128 => assert_eq!(proof.column_openings.len(), 189),
            256 => assert_eq!(proof.column_openings.len(), 400),
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_zero_knowledge() {
    // Create two different witnesses that satisfy the same constraints
    let mut cs = ConstraintSystem::<Fp128>::new(3);
    
    // w[0] + w[1] = 10
    cs.add_linear_constraint(
        vec![(0, Fp128::one()), (1, Fp128::one())],
        Fp128::from(10),
    );
    
    let witness1 = vec![Fp128::from(3), Fp128::from(7), Fp128::zero()];
    let witness2 = vec![Fp128::from(4), Fp128::from(6), Fp128::zero()];
    
    assert!(cs.is_satisfied(&witness1).unwrap());
    assert!(cs.is_satisfied(&witness2).unwrap());
    
    let params = LigeroParams::security_128();
    let instance = LigeroInstance::new(params, cs).unwrap();
    
    let prover = LigeroProver::new(instance.clone()).unwrap();
    
    // Generate proofs with different randomness
    let proof1 = prover.prove(&witness1, &mut ChaCha20Rng::seed_from_u64(1)).unwrap();
    let proof2 = prover.prove(&witness2, &mut ChaCha20Rng::seed_from_u64(2)).unwrap();
    
    // Both proofs should verify
    let verifier = LigeroVerifier::new(instance).unwrap();
    assert!(verifier.verify(&proof1).unwrap());
    assert!(verifier.verify(&proof2).unwrap());
    
    // Proofs should be different (due to randomness)
    assert_ne!(proof1.column_roots, proof2.column_roots);
}