use longfellow_sumcheck::prover::Prover;
use longfellow_sumcheck::circuit::{Layer, GateType};
use longfellow_sumcheck::SumcheckOptions;
use longfellow_sumcheck::transcript::SumcheckTranscript;
use longfellow_arrays::dense::Dense;
use longfellow_algebra::Fp128;
use rand::rngs::OsRng;

fn main() {
    println!("Testing sumcheck implementation...");
    
    // Create a simple layer: output = input[0] + input[1]
    let mut layer = Layer::<Fp128>::new(0, 1, 0); // 1 output, 2 inputs
    layer.add_gate(0, 0, 1, GateType::Add(Fp128::one())).unwrap();
    
    // Create wire values
    let wires = Dense::from_vec(1, 2, vec![Fp128::from_u64(3), Fp128::from_u64(5)]).unwrap();
    let prover = Prover::new(wires, 1, SumcheckOptions::default());
    
    // Expected claim: 3 + 5 = 8
    let claim = Fp128::from_u64(8);
    let mut transcript = SumcheckTranscript::new(b"test");
    
    match prover.prove_layer(&layer, claim, &mut transcript, &mut OsRng) {
        Ok(proof) => {
            println!("Proof generated successfully!");
            println!("Hand polys: {:?}", proof.hand_polys.len());
            println!("Wire claims: {:?}", proof.wire_claims.len());
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}