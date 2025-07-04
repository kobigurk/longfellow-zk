/// Equivalence tests for circuits module

use longfellow_circuits::{
    StandardCircuit, CircuitBuilder, Constraint,
    gadgets::*, hash::*, comparison::*, arithmetic::*, boolean::*,
    utils::*,
};
use longfellow_algebra::Fp128;
use std::time::Instant;

#[test]
fn test_basic_circuit_operations() {
    println!("\n=== Basic Circuit Operations Test ===");
    
    let mut circuit = StandardCircuit::<Fp128>::new();
    
    // Test variable allocation
    let a = circuit.alloc_var();
    let b = circuit.alloc_var();
    let c = circuit.alloc_var();
    
    assert_eq!(a, 0, "First variable should be 0");
    assert_eq!(b, 1, "Second variable should be 1");
    assert_eq!(c, 2, "Third variable should be 2");
    
    // Test constraint addition
    circuit.add_constraint(Constraint::Linear {
        coeffs: vec![(a, Fp128::one()), (b, Fp128::one()), (c, -Fp128::one())],
        constant: Fp128::zero(),
    }).unwrap();
    
    // Test quadratic constraint
    let d = circuit.alloc_var();
    circuit.add_constraint(Constraint::Quadratic { x: a, y: b, z: d }).unwrap();
    
    println!("  ✓ Basic circuit operations verified");
}

#[test]
fn test_circuit_gadgets() {
    println!("\n=== Circuit Gadgets Test ===");
    
    let mut circuit = StandardCircuit::<Fp128>::new();
    
    // Test arithmetic gadgets
    let a = circuit.alloc_var();
    let b = circuit.alloc_var();
    
    let sum = add_gate(&mut circuit, a, b).unwrap();
    let product = mul_gate(&mut circuit, a, b).unwrap();
    let constant = const_gate(&mut circuit, Fp128::from(42)).unwrap();
    
    assert!(sum > b, "Sum should be a new variable");
    assert!(product > sum, "Product should be a new variable");
    assert!(constant > product, "Constant should be a new variable");
    
    // Test boolean gadgets
    let bool_a = circuit.alloc_var();
    let bool_b = circuit.alloc_var();
    
    let and_result = and_gate(&mut circuit, bool_a, bool_b).unwrap();
    let or_result = or_gate(&mut circuit, bool_a, bool_b).unwrap();
    let not_result = not_gate(&mut circuit, bool_a).unwrap();
    let xor_result = xor_gate(&mut circuit, bool_a, bool_b).unwrap();
    
    assert!(and_result > bool_b);
    assert!(or_result > and_result);
    assert!(not_result > or_result);
    assert!(xor_result > not_result);
    
    // Test select gadget
    let cond = circuit.alloc_var();
    let if_true = circuit.alloc_var();
    let if_false = circuit.alloc_var();
    let selected = select(&mut circuit, cond, if_true, if_false).unwrap();
    
    assert!(selected > if_false);
    
    println!("  ✓ Circuit gadgets verified");
}

#[test]
fn test_bit_operations() {
    println!("\n=== Bit Operations Test ===");
    
    let mut circuit = StandardCircuit::<Fp128>::new();
    
    // Test bit decomposition
    let value = circuit.alloc_var();
    let bits = bit_decompose(&mut circuit, value, 8).unwrap();
    
    assert_eq!(bits.len(), 8, "Should decompose into 8 bits");
    
    // Test bit packing
    let packed = bit_pack(&mut circuit, &bits).unwrap();
    assert!(packed > value, "Packed value should be a new variable");
    
    // Test bitwise operations
    let a = circuit.alloc_var();
    let b = circuit.alloc_var();
    
    let bitwise = BitwiseCircuit::new(&mut circuit);
    let and_result = bitwise.bitwise_and(a, b, 8).unwrap();
    let or_result = bitwise.bitwise_or(a, b, 8).unwrap();
    let xor_result = bitwise.bitwise_xor(a, b, 8).unwrap();
    let not_result = bitwise.bitwise_not(a, 8).unwrap();
    
    assert!(and_result > b);
    assert!(or_result > and_result);
    assert!(xor_result > or_result);
    assert!(not_result > xor_result);
    
    println!("  ✓ Bit operations verified");
}

#[test]
fn test_comparison_circuits() {
    println!("\n=== Comparison Circuits Test ===");
    
    let mut circuit = StandardCircuit::<Fp128>::new();
    
    // Test range proof
    let value = circuit.alloc_var();
    let mut range_proof = RangeProofCircuit::new(&mut circuit);
    range_proof.prove_range(value, 16).unwrap();
    
    // Test comparisons
    let a = circuit.alloc_var();
    let b = circuit.alloc_var();
    
    let is_less = less_than(&mut circuit, a, b, 8).unwrap();
    let is_equal = is_equal(&mut circuit, a, b).unwrap();
    
    assert!(is_less > b);
    assert!(is_equal > is_less);
    
    // Test membership proof
    let element = circuit.alloc_var();
    let set = vec![
        circuit.alloc_var(),
        circuit.alloc_var(),
        circuit.alloc_var(),
    ];
    
    let mut membership = MembershipCircuit::new(&mut circuit);
    membership.prove_membership(element, &set).unwrap();
    
    println!("  ✓ Comparison circuits verified");
}

#[test]
fn test_arithmetic_circuits() {
    println!("\n=== Arithmetic Circuits Test ===");
    
    let mut circuit = StandardCircuit::<Fp128>::new();
    
    // Test polynomial evaluation
    let coeffs = vec![
        circuit.alloc_var(), // a0
        circuit.alloc_var(), // a1
        circuit.alloc_var(), // a2
    ];
    let x = circuit.alloc_var();
    
    let mut poly = PolynomialCircuit::new(&mut circuit);
    let result = poly.evaluate(&coeffs, x).unwrap();
    assert!(result > x);
    
    // Test vector operations
    let vec_a = vec![
        circuit.alloc_var(),
        circuit.alloc_var(),
        circuit.alloc_var(),
    ];
    let vec_b = vec![
        circuit.alloc_var(),
        circuit.alloc_var(),
        circuit.alloc_var(),
    ];
    
    let mut vec_circuit = VectorCircuit::new(&mut circuit);
    let dot_prod = vec_circuit.dot_product(&vec_a, &vec_b).unwrap();
    let vec_sum = vec_circuit.add(&vec_a, &vec_b).unwrap();
    
    assert!(dot_prod > vec_b[2]);
    assert_eq!(vec_sum.len(), 3);
    
    // Test fixed-point arithmetic
    let mut fixed = FixedPointCircuit::new(&mut circuit, 16);
    let fp_a = circuit.alloc_var();
    let fp_b = circuit.alloc_var();
    
    let fp_sum = fixed.add(fp_a, fp_b).unwrap();
    let fp_prod = fixed.mul(fp_a, fp_b).unwrap();
    
    assert!(fp_sum > fp_b);
    assert!(fp_prod > fp_sum);
    
    println!("  ✓ Arithmetic circuits verified");
}

#[test]
fn test_hash_circuits() {
    println!("\n=== Hash Circuits Test ===");
    
    let mut circuit = StandardCircuit::<Fp128>::new();
    
    // Test Poseidon hash
    let inputs = vec![
        circuit.alloc_var(),
        circuit.alloc_var(),
        circuit.alloc_var(),
    ];
    
    let mut poseidon = PoseidonCircuit::new(&mut circuit, 8, 3);
    let hash = poseidon.hash(&inputs).unwrap();
    assert!(hash > inputs[2]);
    
    // Test SHA-256 circuit (simplified)
    let message_bits = circuit.alloc_vars(256); // 32 bytes
    let mut sha256 = Sha256Circuit::new(&mut circuit);
    let sha_hash = sha256.hash(&message_bits).unwrap();
    assert_eq!(sha_hash.len(), 256); // 256 bits output
    
    // Test SHA-3 circuit (simplified)
    let mut sha3 = Sha3Circuit::new(&mut circuit);
    let sha3_hash = sha3.hash(&message_bits, 256).unwrap();
    assert_eq!(sha3_hash.len(), 256);
    
    println!("  ✓ Hash circuits verified");
}

#[test]
fn test_boolean_circuits() {
    println!("\n=== Boolean Circuits Test ===");
    
    let mut circuit = StandardCircuit::<Fp128>::new();
    
    // Test boolean formula evaluation
    let a = circuit.alloc_var();
    let b = circuit.alloc_var();
    let c = circuit.alloc_var();
    
    let mut bool_circuit = BooleanFormulaCircuit::new(&mut circuit);
    
    // Test CNF: (a OR b) AND (NOT a OR c)
    let cnf = vec![
        vec![(a, true), (b, true)],
        vec![(a, false), (c, true)],
    ];
    let cnf_result = bool_circuit.evaluate_cnf(&cnf).unwrap();
    assert!(cnf_result > c);
    
    // Test DNF: (a AND b) OR (NOT a AND c)
    let dnf = vec![
        vec![(a, true), (b, true)],
        vec![(a, false), (c, true)],
    ];
    let dnf_result = bool_circuit.evaluate_dnf(&dnf).unwrap();
    assert!(dnf_result > cnf_result);
    
    // Test lookup table
    let index = circuit.alloc_var();
    let table = vec![
        Fp128::from(10),
        Fp128::from(20),
        Fp128::from(30),
        Fp128::from(40),
    ];
    
    let mut lookup = LookupTableCircuit::new(&mut circuit);
    let lookup_result = lookup.lookup(index, &table).unwrap();
    assert!(lookup_result > index);
    
    // Test multiplexer
    let sel = circuit.alloc_var();
    let in0 = circuit.alloc_var();
    let in1 = circuit.alloc_var();
    
    let mut mux = MuxCircuit::new(&mut circuit);
    let mux_result = mux.mux2(sel, in0, in1).unwrap();
    assert!(mux_result > in1);
    
    println!("  ✓ Boolean circuits verified");
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    
    #[test]
    fn bench_constraint_generation() {
        println!("\n=== Constraint Generation Benchmarks ===");
        
        let constraint_counts = vec![100, 1000, 10000];
        
        for count in constraint_counts {
            let mut circuit = StandardCircuit::<Fp128>::new();
            
            // Pre-allocate variables
            let vars: Vec<usize> = (0..count).map(|_| circuit.alloc_var()).collect();
            
            let start = Instant::now();
            
            // Add linear constraints
            for i in 0..count/2 {
                circuit.add_constraint(Constraint::Linear {
                    coeffs: vec![
                        (vars[i], Fp128::one()),
                        (vars[(i + 1) % count], Fp128::one()),
                        (vars[(i + 2) % count], -Fp128::one()),
                    ],
                    constant: Fp128::zero(),
                }).unwrap();
            }
            
            // Add quadratic constraints
            for i in 0..count/2 {
                let z = circuit.alloc_var();
                circuit.add_constraint(Constraint::Quadratic {
                    x: vars[i],
                    y: vars[(i + 1) % count],
                    z,
                }).unwrap();
            }
            
            let duration = start.elapsed();
            
            println!("  {} constraints: {:?} ({:.2} μs/constraint)",
                count,
                duration,
                duration.as_micros() as f64 / count as f64
            );
        }
    }
    
    #[test]
    fn bench_gadget_operations() {
        println!("\n=== Gadget Operation Benchmarks ===");
        
        let mut circuit = StandardCircuit::<Fp128>::new();
        
        // Pre-allocate variables
        let vars: Vec<usize> = (0..1000).map(|_| circuit.alloc_var()).collect();
        
        // Benchmark arithmetic gadgets
        let start = Instant::now();
        let iterations = 1000;
        
        for i in 0..iterations {
            let _ = add_gate(&mut circuit, vars[i % 100], vars[(i + 1) % 100]).unwrap();
        }
        
        let add_duration = start.elapsed();
        
        // Benchmark multiplication gadgets
        let start = Instant::now();
        
        for i in 0..iterations {
            let _ = mul_gate(&mut circuit, vars[i % 100], vars[(i + 1) % 100]).unwrap();
        }
        
        let mul_duration = start.elapsed();
        
        // Benchmark boolean gadgets
        let start = Instant::now();
        
        for i in 0..iterations {
            let _ = and_gate(&mut circuit, vars[i % 100], vars[(i + 1) % 100]).unwrap();
        }
        
        let and_duration = start.elapsed();
        
        println!("  Addition gates: {:?} ({:.2} μs/op)",
            add_duration,
            add_duration.as_micros() as f64 / iterations as f64
        );
        
        println!("  Multiplication gates: {:?} ({:.2} μs/op)",
            mul_duration,
            mul_duration.as_micros() as f64 / iterations as f64
        );
        
        println!("  AND gates: {:?} ({:.2} μs/op)",
            and_duration,
            and_duration.as_micros() as f64 / iterations as f64
        );
    }
    
    #[test]
    fn bench_complex_circuits() {
        println!("\n=== Complex Circuit Benchmarks ===");
        
        // Benchmark hash circuit
        let mut circuit = StandardCircuit::<Fp128>::new();
        let inputs: Vec<usize> = (0..10).map(|_| circuit.alloc_var()).collect();
        
        let start = Instant::now();
        let iterations = 100;
        
        for _ in 0..iterations {
            let mut poseidon = PoseidonCircuit::new(&mut circuit, 8, 3);
            let _ = poseidon.hash(&inputs[..3]).unwrap();
        }
        
        let hash_duration = start.elapsed();
        
        // Benchmark range proof
        let mut circuit = StandardCircuit::<Fp128>::new();
        let values: Vec<usize> = (0..100).map(|_| circuit.alloc_var()).collect();
        
        let start = Instant::now();
        
        for i in 0..iterations {
            let mut range_proof = RangeProofCircuit::new(&mut circuit);
            range_proof.prove_range(values[i % 100], 32).unwrap();
        }
        
        let range_duration = start.elapsed();
        
        // Benchmark polynomial evaluation
        let mut circuit = StandardCircuit::<Fp128>::new();
        let coeffs: Vec<usize> = (0..10).map(|_| circuit.alloc_var()).collect();
        let x_values: Vec<usize> = (0..100).map(|_| circuit.alloc_var()).collect();
        
        let start = Instant::now();
        
        for i in 0..iterations {
            let mut poly = PolynomialCircuit::new(&mut circuit);
            let _ = poly.evaluate(&coeffs, x_values[i % 100]).unwrap();
        }
        
        let poly_duration = start.elapsed();
        
        println!("  Poseidon hash (3 inputs): {:?} ({:.2} μs/op)",
            hash_duration,
            hash_duration.as_micros() as f64 / iterations as f64
        );
        
        println!("  Range proof (32 bits): {:?} ({:.2} μs/op)",
            range_duration,
            range_duration.as_micros() as f64 / iterations as f64
        );
        
        println!("  Polynomial eval (degree 9): {:?} ({:.2} μs/op)",
            poly_duration,
            poly_duration.as_micros() as f64 / iterations as f64
        );
    }
    
    #[test]
    fn bench_circuit_sizes() {
        println!("\n=== Circuit Size Benchmarks ===");
        
        // Measure circuit sizes for different operations
        let operations = vec![
            ("8-bit comparison", |c: &mut StandardCircuit<Fp128>| {
                let a = c.alloc_var();
                let b = c.alloc_var();
                let _ = less_than(c, a, b, 8).unwrap();
            }),
            ("16-bit multiplication", |c: &mut StandardCircuit<Fp128>| {
                let a = c.alloc_var();
                let b = c.alloc_var();
                let mut arith = IntegerArithmeticCircuit::new(c);
                let m = c.alloc_var();
                let _ = arith.mod_mul(a, b, m).unwrap();
            }),
            ("32-bit range proof", |c: &mut StandardCircuit<Fp128>| {
                let value = c.alloc_var();
                let mut range = RangeProofCircuit::new(c);
                range.prove_range(value, 32).unwrap();
            }),
        ];
        
        for (name, op) in operations {
            let mut circuit = StandardCircuit::<Fp128>::new();
            let initial_vars = circuit.num_vars();
            
            op(&mut circuit);
            
            let total_vars = circuit.num_vars() - initial_vars;
            
            println!("  {}: {} variables", name, total_vars);
        }
    }
}