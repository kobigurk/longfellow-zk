/// Equivalence tests for random/transcript module

use longfellow_random::{Transcript, ChaChaRng};
use longfellow_algebra::Fp128;
use std::time::Instant;

#[test]
fn test_transcript_basic_operations() {
    println!("\n=== Transcript Basic Operations Test ===");
    
    let mut transcript = Transcript::new(b"test-protocol");
    
    // Add various types of data
    transcript.append_message(b"round", b"1");
    transcript.append_bytes(b"data", &[1, 2, 3, 4, 5]);
    transcript.append_field_element(b"field", &Fp128::from(42));
    
    // Get challenges
    let challenge1 = transcript.challenge_field_element(b"challenge1");
    let challenge2 = transcript.challenge_field_element(b"challenge2");
    
    // Challenges should be different
    assert_ne!(challenge1, challenge2, "Different challenges should produce different values");
    
    // Create identical transcript
    let mut transcript2 = Transcript::new(b"test-protocol");
    transcript2.append_message(b"round", b"1");
    transcript2.append_bytes(b"data", &[1, 2, 3, 4, 5]);
    transcript2.append_field_element(b"field", &Fp128::from(42));
    
    // Should produce same challenges
    let challenge1_2 = transcript2.challenge_field_element(b"challenge1");
    let challenge2_2 = transcript2.challenge_field_element(b"challenge2");
    
    assert_eq!(challenge1, challenge1_2, "Same transcript should produce same challenges");
    assert_eq!(challenge2, challenge2_2, "Same transcript should produce same challenges");
    
    println!("  ✓ Transcript determinism verified");
}

#[test]
fn test_transcript_ordering_matters() {
    println!("\n=== Transcript Ordering Test ===");
    
    // Transcript 1: A then B
    let mut t1 = Transcript::new(b"protocol");
    t1.append_message(b"msg", b"A");
    t1.append_message(b"msg", b"B");
    let c1 = t1.challenge_field_element(b"challenge");
    
    // Transcript 2: B then A
    let mut t2 = Transcript::new(b"protocol");
    t2.append_message(b"msg", b"B");
    t2.append_message(b"msg", b"A");
    let c2 = t2.challenge_field_element(b"challenge");
    
    assert_ne!(c1, c2, "Different ordering should produce different challenges");
    
    println!("  ✓ Transcript ordering sensitivity verified");
}

#[test]
fn test_transcript_domain_separation() {
    println!("\n=== Transcript Domain Separation Test ===");
    
    // Same data, different protocols
    let mut t1 = Transcript::new(b"protocol-v1");
    t1.append_message(b"data", b"hello");
    let c1 = t1.challenge_field_element(b"challenge");
    
    let mut t2 = Transcript::new(b"protocol-v2");
    t2.append_message(b"data", b"hello");
    let c2 = t2.challenge_field_element(b"challenge");
    
    assert_ne!(c1, c2, "Different protocols should produce different challenges");
    
    // Same protocol, different labels
    let mut t3 = Transcript::new(b"protocol");
    t3.append_message(b"label1", b"data");
    let c3 = t3.challenge_field_element(b"challenge");
    
    let mut t4 = Transcript::new(b"protocol");
    t4.append_message(b"label2", b"data");
    let c4 = t4.challenge_field_element(b"challenge");
    
    assert_ne!(c3, c4, "Different labels should produce different challenges");
    
    println!("  ✓ Domain separation verified");
}

#[test]
fn test_transcript_fork() {
    println!("\n=== Transcript Fork Test ===");
    
    let mut base = Transcript::new(b"protocol");
    base.append_message(b"common", b"data");
    
    // Fork into two transcripts
    let mut fork1 = base.fork(b"path1");
    let mut fork2 = base.fork(b"path2");
    
    // Add same data to forks
    fork1.append_message(b"msg", b"hello");
    fork2.append_message(b"msg", b"hello");
    
    // Should produce different challenges
    let c1 = fork1.challenge_field_element(b"challenge");
    let c2 = fork2.challenge_field_element(b"challenge");
    
    assert_ne!(c1, c2, "Forked transcripts should produce different challenges");
    
    println!("  ✓ Transcript forking verified");
}

#[test]
fn test_chacha_rng() {
    println!("\n=== ChaCha RNG Test ===");
    
    // Test with same seed
    let seed = [42u8; 32];
    let mut rng1 = ChaChaRng::from_seed(seed);
    let mut rng2 = ChaChaRng::from_seed(seed);
    
    // Should produce same sequence
    for i in 0..10 {
        let v1 = rng1.random_field_element::<Fp128>();
        let v2 = rng2.random_field_element::<Fp128>();
        assert_eq!(v1, v2, "Same seed should produce same sequence at position {}", i);
    }
    
    // Different seeds
    let seed2 = [43u8; 32];
    let mut rng3 = ChaChaRng::from_seed(seed2);
    let v3 = rng3.random_field_element::<Fp128>();
    let v1 = ChaChaRng::from_seed(seed).random_field_element::<Fp128>();
    
    assert_ne!(v1, v3, "Different seeds should produce different values");
    
    println!("  ✓ ChaCha RNG determinism verified");
}

#[test]
fn test_transcript_rng_derivation() {
    println!("\n=== Transcript RNG Derivation Test ===");
    
    let mut transcript = Transcript::new(b"protocol");
    transcript.append_message(b"commitment", b"abcdef");
    
    // Derive RNG from transcript
    let mut rng = transcript.build_rng();
    
    // Generate some random values
    let values: Vec<Fp128> = (0..5)
        .map(|_| rng.random_field_element())
        .collect();
    
    // Verify determinism
    let mut transcript2 = Transcript::new(b"protocol");
    transcript2.append_message(b"commitment", b"abcdef");
    let mut rng2 = transcript2.build_rng();
    
    for (i, &expected) in values.iter().enumerate() {
        let actual = rng2.random_field_element::<Fp128>();
        assert_eq!(actual, expected, "RNG should be deterministic at position {}", i);
    }
    
    println!("  ✓ Transcript-derived RNG verified");
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    
    #[test]
    fn bench_transcript_operations() {
        println!("\n=== Transcript Operation Benchmarks ===");
        
        let mut transcript = Transcript::new(b"bench-protocol");
        
        // Benchmark append operations
        let start = Instant::now();
        let iterations = 10000;
        
        for i in 0..iterations {
            transcript.append_message(b"round", i.to_string().as_bytes());
        }
        
        let append_duration = start.elapsed();
        
        // Benchmark challenge generation
        let start = Instant::now();
        let challenges = 1000;
        
        for i in 0..challenges {
            let _ = transcript.challenge_field_element(&i.to_be_bytes());
        }
        
        let challenge_duration = start.elapsed();
        
        println!("  Append message: {:?} ({:.2} ns/op)", 
            append_duration,
            append_duration.as_nanos() as f64 / iterations as f64
        );
        
        println!("  Generate challenge: {:?} ({:.2} μs/op)", 
            challenge_duration,
            challenge_duration.as_micros() as f64 / challenges as f64
        );
    }
    
    #[test]
    fn bench_chacha_rng() {
        println!("\n=== ChaCha RNG Benchmarks ===");
        
        let seed = [0u8; 32];
        let mut rng = ChaChaRng::from_seed(seed);
        
        // Benchmark field element generation
        let start = Instant::now();
        let iterations = 10000;
        
        for _ in 0..iterations {
            let _ = rng.random_field_element::<Fp128>();
        }
        
        let field_duration = start.elapsed();
        
        // Benchmark byte generation
        let start = Instant::now();
        let mut bytes = vec![0u8; 1024];
        
        for _ in 0..iterations {
            rng.fill_bytes(&mut bytes);
        }
        
        let bytes_duration = start.elapsed();
        
        println!("  Random field element: {:?} ({:.2} ns/op)", 
            field_duration,
            field_duration.as_nanos() as f64 / iterations as f64
        );
        
        println!("  Random 1KB: {:?} ({:.2} MB/s)", 
            bytes_duration,
            (iterations as f64 * 1024.0) / bytes_duration.as_secs_f64() / 1_000_000.0
        );
    }
    
    #[test]
    fn bench_transcript_fork() {
        println!("\n=== Transcript Fork Benchmarks ===");
        
        let mut base = Transcript::new(b"protocol");
        
        // Add some base data
        for i in 0..100 {
            base.append_message(b"data", &i.to_be_bytes());
        }
        
        // Benchmark forking
        let start = Instant::now();
        let forks = 1000;
        
        for i in 0..forks {
            let mut fork = base.fork(&i.to_be_bytes());
            let _ = fork.challenge_field_element(b"challenge");
        }
        
        let duration = start.elapsed();
        
        println!("  Fork and challenge: {:?} ({:.2} μs/op)", 
            duration,
            duration.as_micros() as f64 / forks as f64
        );
    }
    
    #[test]
    fn bench_replay_protection() {
        println!("\n=== Replay Protection Benchmarks ===");
        
        // Simulate interactive protocol
        let mut prover = Transcript::new(b"protocol");
        let mut verifier = Transcript::new(b"protocol");
        
        let rounds = 20;
        let start = Instant::now();
        
        for round in 0..rounds {
            // Prover sends commitment
            let commitment = Fp128::from(round as u64 * 100);
            prover.append_field_element(b"commitment", &commitment);
            verifier.append_field_element(b"commitment", &commitment);
            
            // Verifier sends challenge
            let challenge = verifier.challenge_field_element(b"challenge");
            prover.append_field_element(b"v_challenge", &challenge);
            
            // Prover responds
            let response = Fp128::from(round as u64 * 200);
            prover.append_field_element(b"response", &response);
            verifier.append_field_element(b"response", &response);
        }
        
        let duration = start.elapsed();
        
        println!("  {} round protocol: {:?} ({:.2} μs/round)", 
            rounds,
            duration,
            duration.as_micros() as f64 / rounds as f64
        );
    }
}